use std::cmp::Ordering;

use crate::{
    alternation::RegexAlternation,
    char_class::{RegexCharClass, RegexClassItem},
    compile::CompileResult,
    error::CompileErrorKind,
    group::RegexGroup,
    regex::Regex,
    repetition::{Quantifier, RegexRepetition, RepetitionKind},
    span::Span,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Range {
    start: Vec<u8>,
    end: Vec<u8>,
    radix: u8,
    pub(crate) span: Span,
}

impl Range {
    pub(crate) fn new(start: Vec<u8>, end: Vec<u8>, radix: u8, span: Span) -> Self {
        Range { start, end, radix, span }
    }

    pub(crate) fn comp(&self) -> CompileResult<'static> {
        match range(&self.start, &self.end, 0, self.radix) {
            Ok(rule) => Ok(rule.to_regex()),
            Err(Error) => {
                Err(CompileErrorKind::Other("Expanding the range yielded an unexpected error")
                    .at(self.span))
            }
        }
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn hex(n: u8) -> char {
            match n {
                0..=9 => (n + b'0') as char,
                _ => (n + (b'A' - 10)) as char,
            }
        }

        write!(
            f,
            "Range (base {}): {}-{}",
            self.radix,
            self.start.iter().map(|&n| hex(n)).collect::<String>(),
            self.end.iter().map(|&n| hex(n)).collect::<String>(),
        )
    }
}

/// This generates a set of rules that exactly match a string containing a number in a certain
/// range.
///
/// For example, `range(&[1,2,0], &[2,0,0], 0, 10)` matches "120", "125", "150",
/// "199", "200", but not "119" or "201".
///
/// The generated regex is always optimal in terms of search performance. However, it might be
/// somewhat bigger than a regex optimized for size instead of performance.
///
/// This algorithm has been extensively fuzzed, so you can trust its correctness even in rare
/// edge cases. The fuzzer generates all possible ranges and validates them by matching a large
/// number of test strings against them using the `regex` crate. It starts with smaller ranges
/// and tries larger and larger ranges with all permutations (0-0, 0-1, 1-1, 0-2, 1-2,
/// 2-2, 0-3, 1-3, 2-3, 3-3, ...). Run the fuzzer with `cargo test --test it -- --fuzz-ranges`.
///
/// ## How it works
///
/// Lower and upper bound of the range are passed to this function as slices containing individual
/// digits.
///
/// We always look only at the first digit of each bound; these digits are called `ax` (from lower
/// bound) and `bx` (from upper bound). For simplicity, we assume that the radix is 10 (decimal).
/// For example:
///
/// ```no_test
/// a = [4]
/// b = [7, 0, 5]
/// ```
///
/// This means we need a regex that matches a number between 10 and 799. By looking at the first
/// digit, we can deduce:
///
/// - The number can't start with 0 (leading zeros aren't allowed)
/// - The number can start with 1, 2 or 3, but must be followed 1 or 2 more digit in that case
/// - The number can be 4, 5 or 6, and can be followed by 0, 1 or 2 more digits
/// - If the number starts with 7, it can be followed by
///     - nothing
///     - a zero, and possibly a third digit that is at most 5
///     - a digit greater than zero, if there is no third digit.
/// - If the number starts with 8 or 9, it can be followed by at most 1 more digit.
///
/// This is implemented recursively. We always remove the first digit from the slices. We then
/// create a number of alternatives, each starting with a different digit or range of digits:
///
/// 1. `0 ..= ax-1`
/// 2. `ax`
/// 3. `ax+1 ..= bx-1`
/// 4. `bx`
/// 5. `bx+1 ..= 9`
///
/// If `ax` and `bx` are identical, 3. and 4. are omitted; if they're consecutive numbers, 3.
/// is omitted. If `ax` is 0 or `bx` is 9, 1. or 5. is omitted, respectively. If `ax` is bigger
/// than `bx`, the alternatives are a bit different, and this is important later:
///
/// 1. `0 ..= bx-1`
/// 2. `bx`
/// 3. `bx+1 ..= ax-1`
/// 4. `ax`
/// 5. `ax+1 ..= 9`
///
/// There is one more special case: The first digit in a number can't be 0, unless the range's
/// lower bound is 0. So we check if we are currently looking at the first digit, and if that is
/// the case, the first character class omits 0. If the lower bound is 0, then an alternative
/// containing _only_ 0 is added _once_.
///
/// Now, for each of the above alternatives, we add two things: A character class matching the first
/// digit, and _something_ matching the remaining digits. That _something_ is calculated by
/// recursively calling the `range` function on the remaining digits. To make sure that this doesn't
/// recurse for infinity, we must detect terminal calls (calls that stop recursing):
///
/// - If both slices are empty, we are done.
///
/// - If both slices contain exactly 1 digit, we simply add a character class matching a digit in
///   that range.
///
/// - If the first slice is empty but not the second one, we apply a trick: We add a 0 to the lower
///   bound and try again. Also, the returned sub-expression is made optional.
///     - For example, `range([4], [400])` at some point adds an alternative starting with `4` and
///       calls `range([], [0, 0])` recursively. We want this to match the empty string, any single
///       digit, or two zeros, because a "4" matching the range 4-400 can be followed by nothing,
///       any single digit or two zeros.
///   If we just added a 0 to the lower bound, that would mean that the 4 MUST be followed by
///   at least one more digit. We don't want that, so we make the expression following the 4
///   optional.
///
/// - If the second slice is empty but not the first, this is an error that should NEVER happen.
///   The parser validates the input so that the upper bound can't be smaller/shorter than
///   the lower bound.
///
/// Now, about the alternatives: This part is quite interesting. To recap, the alternatives are
/// either this:
///
/// 1. `0 ..= ax-1`
/// 2. `ax`
/// 3. `ax+1 ..= bx-1`
/// 4. `bx`
/// 5. `bx+1 ..= 9`
///
/// or this, if `bx > ax`:
///
/// 1. `0 ..= bx-1`
/// 2. `bx`
/// 3. `bx+1 ..= ax-1`
/// 4. `ax`
/// 5. `ax+1 ..= 9`
///
/// Alternative 1 and 5 are the same, if we substitute `ax` and `bx` with `min(ax, bx)` in 1. and
/// with `max(ax, bx)` in step 5:
///
/// ```no_test
/// 1. [1-(min - 1)] [0-9]{la + 1, lb}  (first digit)
/// 1. [0-(min - 1)] [0-9]{la + 1, lb}  (not first digit)
/// 5. [(max + 1)-9] [0-9]{al, bl - 1}
/// ```
///
/// (`la` and `lb` are the lengths of the remaining digits in the lower and upper bound,
/// respectively).
///
/// What is the deal with the added or subtracted 1's? If we have a lower bound such as 533, the
/// number must be at least 3 digits long, because the lower bound is three digits long. However,
/// if the first digit is less than 5, it must be at least 4 digits long to be greater than 533.
/// With the upper bound, it's the exact opposite: For example, with an upper bound of 6111, the
/// number can be at most 3 digits if it starts with 7, 8 or 9.
///
/// I'm not going to explain the remaining alternatives (2 through 4), since you can understand
/// them by reading the code.
///
/// The last step is to optimize the alternatives to be as compact as possible. This is achieved
/// by simplifying and merging alternatives if possible. For example,
///
/// ```no_test
/// [0-4] [5-9] | 5 [5-9]
/// ```
///
/// This can be merged into `[0-5] [5-9]`. The rules are like addition and multiplication, where
/// alternation (with `|`) is equivalent to `+` and concatenation is equivalent to `*`.
/// This means we can use the distributive law: `a * x + b * x = (a + b) * x`. Note that we only
/// do this if the first character class of each alternation are consecutive; for example,
/// we merge `[0-4]` and `5`, but not `[0-4]` and `[6-9]`. This would be possible in theory, but
/// would be computationally more expensive, since the second part of each alternation must be
/// checked for equality.
///
/// The next optimization is to replace concatenation of equal elements with repetition. In other
/// words, we replace `a + a` with `a * 2`, and `a + (a * 2)` with `a * 3`. This is important,
/// because when we check whether two expressions are equal, it only works if they have the exact
/// same structure: `[0-9][0-9]` is not considered equal to `[0-9]{2}`. So this optimization also
/// serves as a _normalization_, to ensure that equal alternatives can be merged.
fn range(a: &[u8], b: &[u8], level: usize, radix: u8) -> Result<Rule, Error> {
    let hi_digit = radix - 1;
    let lo_digit = if level == 0 { 1 } else { 0 };

    debug_assert!(a.len() <= b.len() && (a.len() < b.len() || a <= b));

    Ok(match (a.split_first(), b.split_first()) {
        (None, None) => Rule::Empty,
        (Some(_), None) => return Err(Error),
        (None, Some(_)) => range(&[0], b, level + 1, radix)?.optional(),
        (Some((&ax, [])), Some((&bx, []))) => Rule::class(ax, bx),
        (Some((&ax, a_rest)), Some((&bx, b_rest))) => {
            let (min, max) = (u8::min(ax, bx), u8::max(ax, bx));
            let mut alternatives = vec![];

            if min > lo_digit && a_rest.len() < b_rest.len() {
                // 1.
                alternatives.push(vec![
                    Rule::class(lo_digit, min - 1),
                    Rule::class(0, hi_digit).repeat(a_rest.len() + 1, b_rest.len()),
                ]);
            }

            match ax.cmp(&bx) {
                // ax == bx:
                Ordering::Equal => {
                    // 2.
                    alternatives
                        .push(vec![Rule::class(ax, bx), range(a_rest, b_rest, level + 1, radix)?]);
                }
                // ax < bx:
                Ordering::Less => {
                    if level == 0 && ax == 0 && a_rest.is_empty() {
                        // add zero once
                        alternatives.push(vec![Rule::class(0, 0)]);
                    } else {
                        // 2.
                        alternatives.push(vec![
                            Rule::class(min, min),
                            range(a_rest, &vec![hi_digit; b_rest.len()], level + 1, radix)?,
                        ]);
                    }
                    if max - min >= 2 {
                        // 3.
                        alternatives.push(vec![
                            Rule::class(min + 1, max - 1),
                            Rule::class(0, hi_digit).repeat(a_rest.len(), b_rest.len()),
                        ]);
                    }
                    // 4.
                    alternatives.push(vec![
                        Rule::class(max, max),
                        range(&vec![0; a_rest.len()], b_rest, level + 1, radix)?,
                    ]);
                }
                // ax > bx:
                Ordering::Greater => {
                    // 2.
                    alternatives.push(vec![
                        Rule::class(min, min),
                        range(&vec![0; a.len()], b_rest, level + 1, radix)?,
                    ]);
                    if max - min >= 2 && a_rest.len() + 2 <= b_rest.len() {
                        // 3.
                        alternatives.push(vec![
                            Rule::class(min + 1, max - 1),
                            Rule::class(0, hi_digit).repeat(a_rest.len() + 1, b_rest.len() - 1),
                        ]);
                    }
                    // 4.
                    alternatives.push(vec![
                        Rule::class(max, max),
                        range(a_rest, &vec![hi_digit; b_rest.len() - 1], level + 1, radix)?,
                    ]);
                }
            }

            if max < hi_digit && a_rest.len() < b_rest.len() {
                // 5.
                alternatives.push(vec![
                    Rule::class(max + 1, hi_digit),
                    Rule::class(0, hi_digit).repeat(a_rest.len(), b_rest.len() - 1),
                ]);
            }

            merge_and_optimize_alternatives(alternatives)
        }
    })
}

fn merge_and_optimize_alternatives(alternatives: Vec<Vec<Rule>>) -> Rule {
    let mut alternatives =
        alternatives.into_iter().fold(vec![], |mut acc: Vec<Vec<Rule>>, mut rules| {
            if let [this1, this2] = rules.as_slice() {
                if this1 == this2 {
                    let rule = rules.pop().unwrap();
                    rules[0] = rule.repeat(2, 2);
                } else if let Rule::Repeat(r) = this2 {
                    if r.rule == *this1 {
                        let (min, max) = (r.min, r.max);
                        let _ = rules.pop();
                        let rule = rules.pop().unwrap();
                        rules.push(rule.repeat(min + 1, max + 1));
                    }
                } else if *this2 == Rule::Empty {
                    rules.pop();
                }
            }

            match acc.last_mut() {
                Some(last) => {
                    if let [Rule::Class(prev_class), prev] = last.as_mut_slice() {
                        if let [Rule::Class(this_class), ref mut this2] = rules.as_mut_slice() {
                            if prev == this2 {
                                debug_assert!(prev_class.end + 1 == this_class.start);
                                prev_class.end = this_class.end;

                                if let [last1, last2] = last.as_slice() {
                                    if last1 == last2 {
                                        let rule = last.pop().unwrap();
                                        last[0] = rule.repeat(2, 2);
                                    } else if let Rule::Repeat(r) = last2 {
                                        if r.rule == *last1 {
                                            let (min, max) = (r.min, r.max);
                                            let _ = last.pop();
                                            let rule = last.pop().unwrap();
                                            last.push(rule.repeat(min + 1, max + 1));
                                        }
                                    }
                                }

                                return acc;
                            }
                        }
                    }
                }
                None => {}
            }

            acc.push(rules);
            acc
        });

    if alternatives.len() == 1 && alternatives[0].len() == 1 {
        alternatives.pop().unwrap().pop().unwrap()
    } else {
        Rule::alt(alternatives)
    }
}

struct Error;

#[derive(PartialEq, Eq)]
enum Rule {
    Empty,
    Class(Class),
    Repeat(Box<Repeat>),
    Alt(Alt),
}

// #[cfg(FALSE)]
impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Class(Class { start, end }) => write!(f, "[{start}-{end}]"),
            Self::Repeat(r) => {
                if f.alternate() {
                    write!(f, "{:#?}{{{}, {}}}", r.rule, r.min, r.max)
                } else {
                    write!(f, "{:?}{{{}, {}}}", r.rule, r.min, r.max)
                }
            }
            Self::Alt(arg0) => {
                let mut d = f.debug_tuple("Alt");
                let mut d = &mut d;
                for item in &arg0.0 {
                    if item.iter().map(|n| n.debug_size()).sum::<u64>() < 5 {
                        d = d.field(&format_args!(
                            "{}",
                            item.iter().map(|n| format!("{n:?} ")).collect::<String>().trim()
                        ))
                    } else {
                        d = d.field(item);
                    }
                }
                d.finish()
            }
        }
    }
}

impl Rule {
    fn class(start: u8, end: u8) -> Self {
        Rule::Class(Class { start, end })
    }

    fn repeat(self, min: usize, max: usize) -> Self {
        debug_assert!(min <= max);

        if max == 0 {
            Rule::Empty
        } else if max == 1 && min == 1 {
            self
        } else {
            Rule::Repeat(Box::new(Repeat { rule: self, min, max }))
        }
    }

    fn alt(alts: Vec<Vec<Rule>>) -> Self {
        Rule::Alt(Alt(alts))
    }

    fn optional(self) -> Rule {
        match self {
            Rule::Repeat(mut repeat) if repeat.min <= 1 => {
                repeat.min = 0;
                Rule::Repeat(repeat)
            }
            rule => Rule::repeat(rule, 0, 1),
        }
    }

    fn to_regex(&self) -> Regex<'static> {
        match self {
            Rule::Empty => Regex::Literal(""),
            Rule::Class(c) => c.to_regex(),
            Rule::Repeat(r) => r.to_regex(),
            Rule::Alt(a) => a.to_regex(),
        }
    }

    // #[cfg(FALSE)]
    fn debug_size(&self) -> u64 {
        match self {
            Rule::Empty => 1,
            Rule::Class(_) => 1,
            Rule::Repeat(r) => 1 + r.rule.debug_size(),
            Rule::Alt(_) => 10,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Class {
    start: u8,
    end: u8,
}

#[derive(PartialEq, Eq)]
struct Repeat {
    rule: Rule,
    min: usize,
    max: usize,
}

impl Repeat {
    fn to_regex(&self) -> Regex<'static> {
        Regex::Repetition(Box::new(RegexRepetition::new(
            self.rule.to_regex(),
            RepetitionKind::try_from((self.min as u32, Some(self.max as u32))).unwrap(),
            Quantifier::Greedy,
        )))
    }
}

#[derive(PartialEq, Eq)]
struct Alt(Vec<Vec<Rule>>);

impl Alt {
    fn to_regex(&self) -> Regex<'static> {
        Regex::Alternation(RegexAlternation::new(
            self.0
                .iter()
                .map(|v| {
                    Regex::Group(RegexGroup::new(v.iter().map(|r| r.to_regex()).collect(), None))
                })
                .collect(),
        ))
    }
}

impl Class {
    fn to_regex(self) -> Regex<'static> {
        let (a, b) = (self.start, self.end);

        Regex::CharClass(RegexCharClass::new(match (a, b, a == b) {
            (0..=9, _, true) => return Regex::Char((a + b'0') as char),
            (0..=9, 0..=9, _) => {
                vec![RegexClassItem::range_unchecked((a + b'0') as char, (b + b'0') as char)]
            }
            (10.., _, true) => vec![
                RegexClassItem::Char((a + b'a' - 10) as char),
                RegexClassItem::Char((a + b'A' - 10) as char),
            ],
            (10.., 10.., _) => vec![
                RegexClassItem::range_unchecked((a + b'a' - 10) as char, (b + b'a' - 10) as char),
                RegexClassItem::range_unchecked((a + b'A' - 10) as char, (b + b'A' - 10) as char),
            ],
            (9, 10, _) => vec![
                RegexClassItem::Char('9'),
                RegexClassItem::Char('a'),
                RegexClassItem::Char('A'),
            ],
            (_, 10, _) => vec![
                RegexClassItem::range_unchecked((a + b'0') as char, '9'),
                RegexClassItem::Char('a'),
                RegexClassItem::Char('A'),
            ],
            (9, _, _) => vec![
                RegexClassItem::Char('9'),
                RegexClassItem::range_unchecked('a', (b + b'a' - 10) as char),
                RegexClassItem::range_unchecked('A', (b + b'A' - 10) as char),
            ],
            _ => vec![
                RegexClassItem::range_unchecked((a + b'0') as char, '9'),
                RegexClassItem::range_unchecked('a', (b + b'a' - 10) as char),
                RegexClassItem::range_unchecked('A', (b + b'A' - 10) as char),
            ],
        }))
    }
}
