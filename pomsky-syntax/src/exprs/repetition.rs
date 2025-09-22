use crate::{Span, error::RepetitionError};

use super::Rule;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Repetition {
    pub rule: Rule,
    pub kind: RepetitionKind,
    pub quantifier: Quantifier,
    pub span: Span,
}

impl Repetition {
    pub(crate) fn new(
        rule: Rule,
        kind: RepetitionKind,
        quantifier: Quantifier,
        span: Span,
    ) -> Self {
        Repetition { rule, kind, quantifier, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        self.rule.pretty_print(buf, true);
        match self.kind {
            RepetitionKind { lower_bound, upper_bound: None } => {
                buf.push('{');
                buf.write_fmt(lower_bound);
                buf.push(',');
                buf.push('}');
            }
            RepetitionKind { lower_bound, upper_bound: Some(upper_bound) }
                if lower_bound == upper_bound =>
            {
                buf.push('{');
                buf.write_fmt(lower_bound);
                buf.push('}');
            }
            RepetitionKind { lower_bound, upper_bound: Some(upper_bound) } => {
                buf.push('{');
                buf.write_fmt(lower_bound);
                buf.push(',');
                buf.write_fmt(upper_bound);
                buf.push('}');
            }
        }
        match self.quantifier {
            Quantifier::Greedy => buf.push_str(" greedy"),
            Quantifier::Lazy => buf.push_str(" lazy"),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Quantifier {
    Greedy,
    Lazy,
    DefaultGreedy,
    DefaultLazy,
}

/// A repetition in its most canonical form, `{x,y}`.
///
/// For example:
///
///  * `'x'?` is equivalent to `'x'{0,1}`
///  * `'x'+` is equivalent to `'x'{1,}`
///  * `'x'*` is equivalent to `'x'{0,}`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RepetitionKind {
    /// The lower bound, e.g. `{4,}`
    pub lower_bound: u32,

    /// The upper bound, e.g. `{0,7}`. `None` means infinity.
    pub upper_bound: Option<u32>,
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for RepetitionKind {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let lower = u.int_in_range(0u8..=40)?;
        if u.arbitrary()? {
            let upper = u.int_in_range(lower..=lower + 40)?;
            Ok(RepetitionKind { lower_bound: lower as u32, upper_bound: Some(upper as u32) })
        } else {
            Ok(RepetitionKind { lower_bound: lower as u32, upper_bound: None })
        }
    }
}

impl RepetitionKind {
    pub(crate) fn zero_inf() -> Self {
        RepetitionKind { lower_bound: 0, upper_bound: None }
    }

    pub(crate) fn one_inf() -> Self {
        RepetitionKind { lower_bound: 1, upper_bound: None }
    }

    pub(crate) fn zero_one() -> Self {
        RepetitionKind { lower_bound: 0, upper_bound: Some(1) }
    }

    pub(crate) fn fixed(n: u32) -> Self {
        RepetitionKind { lower_bound: n, upper_bound: Some(n) }
    }
}

impl TryFrom<(u32, Option<u32>)> for RepetitionKind {
    type Error = RepetitionError;

    fn try_from((lower_bound, upper_bound): (u32, Option<u32>)) -> Result<Self, Self::Error> {
        if lower_bound > upper_bound.unwrap_or(u32::MAX) {
            return Err(RepetitionError::NotAscending);
        }

        Ok(RepetitionKind { lower_bound, upper_bound })
    }
}
