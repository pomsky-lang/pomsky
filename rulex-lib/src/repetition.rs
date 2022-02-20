use crate::Rulex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repetition<'i> {
    rule: Rulex<'i>,
    kind: RepetitionKind,
    greedy: Greedy,
}

impl<'i> Repetition<'i> {
    pub fn new(rule: Rulex<'i>, kind: RepetitionKind, greedy: Greedy) -> Self {
        Repetition { rule, kind, greedy }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Greedy {
    Yes,
    No,
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
    lower_bound: u32,

    /// The upper bound, e.g. `{,7}`. `None` means infinity.
    upper_bound: Option<u32>,
}

impl RepetitionKind {
    pub fn zero_inf() -> Self {
        RepetitionKind {
            lower_bound: 0,
            upper_bound: None,
        }
    }

    pub fn one_inf() -> Self {
        RepetitionKind {
            lower_bound: 1,
            upper_bound: None,
        }
    }

    pub fn zero_one() -> Self {
        RepetitionKind {
            lower_bound: 0,
            upper_bound: Some(1),
        }
    }

    pub fn fixed(n: u32) -> Self {
        RepetitionKind {
            lower_bound: n,
            upper_bound: Some(n),
        }
    }

    pub fn get_range(&self) -> (u32, Option<u32>) {
        (self.lower_bound, self.upper_bound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RepetitionError {
    #[error("Lower bound can't be greater than the upper bound")]
    NotAscending,
}

impl TryFrom<(u32, Option<u32>)> for RepetitionKind {
    type Error = RepetitionError;

    fn try_from((lower_bound, upper_bound): (u32, Option<u32>)) -> Result<Self, Self::Error> {
        if lower_bound > upper_bound.unwrap_or(u32::MAX) {
            return Err(RepetitionError::NotAscending);
        }

        Ok(RepetitionKind {
            lower_bound,
            upper_bound,
        })
    }
}
