use crate::{error::RepetitionError, Span};

use super::Rule;

#[derive(Clone)]
pub struct Repetition<'i> {
    pub rule: Rule<'i>,
    pub kind: RepetitionKind,
    pub quantifier: Quantifier,
    pub span: Span,
}

impl<'i> Repetition<'i> {
    pub(crate) fn new(
        rule: Rule<'i>,
        kind: RepetitionKind,
        quantifier: Quantifier,
        span: Span,
    ) -> Self {
        Repetition { rule, kind, quantifier, span }
    }
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Repetition<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Repetition").field(&self.rule).finish()?;
        match self.kind {
            RepetitionKind { lower_bound, upper_bound: None } => {
                write!(f, "{{{lower_bound}, inf}}")
            }
            RepetitionKind { lower_bound, upper_bound: Some(upper_bound) } => {
                write!(f, "{{{lower_bound}, {upper_bound}}}")
            }
        }?;
        match self.quantifier {
            Quantifier::Greedy => write!(f, " greedy")?,
            Quantifier::Lazy => write!(f, " lazy")?,
            Quantifier::Default => {}
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "pretty-print", derive(Debug))]
pub enum Quantifier {
    Greedy,
    Lazy,
    Default,
}

/// A repetition in its most canonical form, `{x,y}`.
///
/// For example:
///
///  * `'x'?` is equivalent to `'x'{0,1}`
///  * `'x'+` is equivalent to `'x'{1,}`
///  * `'x'*` is equivalent to `'x'{0,}`
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "pretty-print", derive(Debug))]
pub struct RepetitionKind {
    /// The lower bound, e.g. `{4,}`
    pub lower_bound: u32,

    /// The upper bound, e.g. `{0,7}`. `None` means infinity.
    pub upper_bound: Option<u32>,
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
