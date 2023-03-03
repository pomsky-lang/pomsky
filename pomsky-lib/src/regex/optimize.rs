use std::{mem, ops::Add};

use pomsky_syntax::exprs::RepetitionKind;

use crate::exprs::group::RegexGroupKind;

use super::Regex;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Count {
    Zero,
    One,
    Many,
}

impl Add for Count {
    type Output = Count;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Count::Zero, Count::Zero) => Count::Zero,
            (Count::Zero, Count::One) | (Count::One, Count::Zero) => Count::One,
            _ => Count::Many,
        }
    }
}

impl<'i> Regex<'i> {
    pub(crate) fn optimize(&mut self) -> Count {
        match self {
            Regex::Literal(l) => {
                let l: &str = &*l;
                if l.is_empty() {
                    // indicates that the parent should remove it
                    Count::Zero
                } else if l.chars().nth(1).is_none() {
                    Count::One
                } else {
                    Count::Many
                }
            }
            Regex::Group(g) => {
                let mut count = Count::Zero;
                g.parts.retain_mut(|part| {
                    let add = part.optimize();
                    count = count + add;
                    add != Count::Zero
                });

                if g.parts.len() == 1
                    && g.kind == RegexGroupKind::Normal
                    && !matches!(&g.parts[0], Regex::Unescaped(_))
                {
                    // don't remove group if it is wrapping raw regex
                    *self = g.parts.pop().unwrap();
                    return count;
                }

                if matches!(g.kind, RegexGroupKind::Capture | RegexGroupKind::NamedCapture(_)) {
                    Count::One
                } else if g.parts.is_empty() {
                    // indicates that the parent should remove it
                    Count::Zero
                } else {
                    count
                }
            }
            Regex::Alternation(a) => {
                if a.parts.len() == 1 {
                    *self = a.parts.pop().unwrap();
                    return self.optimize();
                }

                for part in &mut a.parts {
                    part.optimize();
                }

                Count::One
            }
            Regex::Repetition(r) => {
                if r.kind.lower_bound == 1 && r.kind.upper_bound == Some(1) {
                    *self = mem::take(&mut r.content);
                    return self.optimize();
                }

                match r.content.optimize() {
                    Count::Zero => {
                        // indicates that the parent should remove it
                        return Count::Zero;
                    }
                    Count::One => match &mut r.content {
                        Regex::Repetition(inner) if inner.quantifier == r.quantifier => {
                            if let Some(kind) = reduce_repetitions(r.kind, inner.kind) {
                                inner.kind = kind;
                                *self = mem::take(&mut r.content);
                            }
                        }
                        _ => {}
                    },
                    Count::Many => {}
                }

                Count::One
            }
            Regex::Lookaround(l) => {
                l.content.optimize();
                Count::One
            }
            Regex::Unescaped(_) => Count::Many,
            Regex::Char(_)
            | Regex::CharSet(_)
            | Regex::Grapheme
            | Regex::Dot
            | Regex::Boundary(_)
            | Regex::Reference(_) => Count::One,
        }
    }
}

fn reduce_repetitions(outer: RepetitionKind, inner: RepetitionKind) -> Option<RepetitionKind> {
    match (outer, inner) {
        (
            RepetitionKind { lower_bound: 0 | 1, upper_bound: _ },
            RepetitionKind { lower_bound: 0 | 1, upper_bound: None },
        )
        | (
            RepetitionKind { lower_bound: 0 | 1, upper_bound: None },
            RepetitionKind { lower_bound: 0 | 1, upper_bound: _ },
        ) => {
            let lower_bound = inner.lower_bound.min(outer.lower_bound);
            Some(RepetitionKind { lower_bound, upper_bound: None })
        }

        (
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(n) },
            RepetitionKind { lower_bound: 0, upper_bound: Some(1) },
        )
        | (
            RepetitionKind { lower_bound: 0, upper_bound: Some(1) },
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(n) },
        ) => Some(RepetitionKind { lower_bound: 0, upper_bound: Some(n) }),

        (
            RepetitionKind { lower_bound: l1, upper_bound: None },
            RepetitionKind { lower_bound: l2, upper_bound: None },
        ) => Some(RepetitionKind { lower_bound: l1.saturating_mul(l2), upper_bound: None }),

        (
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(u1) },
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(u2) },
        ) => {
            let lower_bound = inner.lower_bound.min(outer.lower_bound);
            Some(RepetitionKind { lower_bound, upper_bound: Some(u1.saturating_mul(u2)) })
        }

        _ => None,
    }
}
