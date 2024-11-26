use std::{mem, ops::Add};

use pomsky_syntax::exprs::RepetitionKind;

use crate::{exprs::group::RegexGroupKind, unicode_set::UnicodeSet};

use super::{Regex, RegexCharSet};

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

impl Regex {
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
                for part in &mut a.parts {
                    part.optimize();
                }

                let mut i = 0;
                while i < a.parts.len() - 1 {
                    let (p1, p2) = a.parts.split_at_mut(i + 1);
                    let lhs = &mut p1[i];
                    let rhs = &mut p2[0];

                    if lhs.is_single_char() && rhs.is_single_char() {
                        match (lhs, rhs) {
                            (Regex::Literal(lit1), Regex::Literal(lit2)) => {
                                let mut set = UnicodeSet::new();
                                set.add_char(lit1.chars().next().unwrap());
                                set.add_char(lit2.chars().next().unwrap());
                                a.parts[i] = Regex::CharSet(RegexCharSet::new(set));
                                a.parts.remove(i + 1);
                            }
                            (Regex::Literal(lit), Regex::CharSet(set))
                            | (Regex::CharSet(set), Regex::Literal(lit))
                                if !set.negative =>
                            {
                                let mut set = std::mem::take(set);
                                set.set.add_char(lit.chars().next().unwrap());
                                a.parts[i] = Regex::CharSet(set);
                                a.parts.remove(i + 1);
                            }
                            (Regex::CharSet(set1), Regex::CharSet(set2))
                                if !set1.negative && !set2.negative =>
                            {
                                for range in set2.set.ranges() {
                                    set1.set.add_set_range(range);
                                }
                                for prop in set2.set.props() {
                                    set1.set.add_prop(prop);
                                }
                                a.parts.remove(i + 1);
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    } else {
                        i += 1;
                    }
                }

                if a.parts.len() == 1 {
                    *self = a.parts.pop().unwrap();
                    return self.optimize();
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
            Regex::CharSet(_)
            | Regex::Grapheme
            | Regex::Dot
            | Regex::Boundary(_)
            | Regex::Reference(_)
            | Regex::Recursion => Count::One,
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
        ) => Some(RepetitionKind { lower_bound: mul_repetitions(l1, l2)?, upper_bound: None }),

        (
            RepetitionKind { lower_bound: l1, upper_bound: Some(l1_1) },
            RepetitionKind { lower_bound: l2, upper_bound: Some(l2_1) },
        ) if l1 == l1_1 && l2 == l2_1 => {
            let repetition = mul_repetitions(l1, l2)?;
            Some(RepetitionKind { lower_bound: repetition, upper_bound: Some(repetition) })
        }

        (
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(u1) },
            RepetitionKind { lower_bound: 0 | 1, upper_bound: Some(u2) },
        ) => {
            let lower_bound = inner.lower_bound.min(outer.lower_bound);
            Some(RepetitionKind { lower_bound, upper_bound: Some(mul_repetitions(u1, u2)?) })
        }

        _ => None,
    }
}

fn mul_repetitions(a: u32, b: u32) -> Option<u32> {
    let res = a.saturating_mul(b);
    if res > u16::MAX as u32 {
        // some regex flavors don't support repetitions greater than 2^16
        None
    } else {
        Some(res)
    }
}
