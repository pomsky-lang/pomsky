use std::{mem, ops::Add};

use pomsky_syntax::exprs::RepetitionKind;

use crate::exprs::alternation::RegexAlternation;
use crate::exprs::group::{RegexGroup, RegexGroupKind};
use crate::exprs::repetition::{RegexQuantifier, RegexRepetition};
use crate::unicode_set::UnicodeSet;

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
                if let Some(Regex::Literal(l)) = a.parts.first()
                    && l.is_empty()
                {
                    a.parts.remove(0);
                    let parts = mem::take(&mut a.parts);
                    *self = Regex::Repetition(Box::new(RegexRepetition::new(
                        Regex::Alternation(RegexAlternation { parts }),
                        RepetitionKind { lower_bound: 0, upper_bound: Some(1) },
                        RegexQuantifier::Lazy,
                    )));
                    return self.optimize();
                }
                if let Some(Regex::Literal(l)) = a.parts.last()
                    && l.is_empty()
                {
                    a.parts.pop();
                    let parts = mem::take(&mut a.parts);
                    *self = Regex::Repetition(Box::new(RegexRepetition::new(
                        Regex::Alternation(RegexAlternation { parts }),
                        RepetitionKind { lower_bound: 0, upper_bound: Some(1) },
                        RegexQuantifier::Greedy,
                    )));
                    return self.optimize();
                }

                for part in &mut a.parts {
                    part.optimize();
                }

                let mut merged = false;

                reduce_many_mut(&mut a.parts, |lhs, rhs| {
                    if lhs.is_single_char() && rhs.is_single_char() {
                        match (&mut *lhs, rhs) {
                            (Regex::Literal(lit1), Regex::Literal(lit2)) => {
                                if lit1 == lit2 {
                                    return true;
                                }
                                let mut set = UnicodeSet::new();
                                set.add_char(lit1.chars().next().unwrap());
                                set.add_char(lit2.chars().next().unwrap());
                                *lhs = Regex::CharSet(RegexCharSet::new(set));
                                true
                            }
                            (Regex::Literal(lit), Regex::CharSet(char_set))
                            | (Regex::CharSet(char_set), Regex::Literal(lit))
                                if !char_set.negative =>
                            {
                                let mut char_set = std::mem::take(char_set);
                                char_set.set.add_char(lit.chars().next().unwrap());
                                *lhs = Regex::CharSet(char_set);
                                true
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
                                true
                            }
                            _ => false,
                        }
                    } else if merge_common_prefix(lhs, rhs) {
                        merged = true;
                        true
                    } else {
                        false
                    }
                });

                if merged {
                    for part in &mut a.parts {
                        part.optimize();
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
            | Regex::CompoundCharSet(_)
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

/// Merge adjacent elements in the Vec using the `reducer`, which processes two elements at a time.
///
/// When the reducer returns `true`, this indicates that they were merged into the first element
/// in-place, so the second one needs to be removed.
fn reduce_many_mut<T>(slice: &mut Vec<T>, mut reducer: impl FnMut(&mut T, &mut T) -> bool) {
    let mut i = 0;
    while i < slice.len() - 1 {
        let (p1, p2) = slice.split_at_mut(i + 1);
        let lhs = &mut p1[i];
        let rhs = &mut p2[0];

        let res = reducer(lhs, rhs);
        if res {
            slice.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

fn merge_common_prefix(lhs: &mut Regex, rhs: &mut Regex) -> bool {
    let prefix1 = prefix(lhs);
    let prefix2 = prefix(rhs);

    if let (Some(prefix1), Some(prefix2)) = (prefix1, prefix2)
        && prefix1 == prefix2
    {
        let prefix = match prefix1 {
            Prefix::Dot => Regex::Dot,
            Prefix::Char(c) => Regex::Literal(c.to_string()),
            Prefix::CharSet(char_set) => Regex::CharSet(char_set.clone()),
        };

        remove_prefix(lhs);
        remove_prefix(rhs);

        let group = if let Regex::Alternation(alt) = lhs {
            alt.parts.push(mem::take(rhs));
            vec![prefix, mem::take(lhs)]
        } else {
            let alts = vec![mem::take(lhs), mem::take(rhs)];
            vec![prefix, Regex::Alternation(RegexAlternation::new(alts))]
        };
        *lhs = Regex::Group(RegexGroup::new(group, RegexGroupKind::Normal));

        true
    } else {
        false
    }
}

#[derive(PartialEq, Eq)]
enum Prefix<'a> {
    Dot,
    Char(char),
    CharSet(&'a RegexCharSet),
}

fn prefix(regex: &Regex) -> Option<Prefix<'_>> {
    match regex {
        Regex::Literal(lit) => lit.chars().next().map(Prefix::Char),
        Regex::CharSet(char_set) => Some(Prefix::CharSet(char_set)),
        Regex::Dot => Some(Prefix::Dot),
        Regex::Group(group) if group.kind == RegexGroupKind::Normal => {
            group.parts.first().and_then(prefix)
        }
        _ => None,
    }
}

fn remove_prefix(regex: &mut Regex) {
    match regex {
        Regex::Literal(lit) => {
            let len = lit.chars().next().unwrap().len_utf8();
            lit.drain(0..len);
        }
        Regex::CharSet(_) | Regex::Dot => {
            *regex = Regex::Literal(String::new());
        }
        Regex::Group(group) => {
            if let Some(part) = group.parts.first_mut() {
                remove_prefix(part);
            }
            if let Some(Regex::Literal(s)) = group.parts.first()
                && s.is_empty()
            {
                group.parts.remove(0);
                if group.parts.len() == 1 {
                    *regex = group.parts.pop().unwrap();
                }
            }
        }
        _ => {}
    }
}
