use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    span::Span,
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Repetition<'i> {
    rule: Rulex<'i>,
    kind: RepetitionKind,
    quantifier: Quantifier,
    pub(crate) span: Span,
}

impl<'i> Repetition<'i> {
    pub(crate) fn new(
        rule: Rulex<'i>,
        kind: RepetitionKind,
        quantifier: Quantifier,
        span: Span,
    ) -> Self {
        Repetition { rule, kind, quantifier, span }
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
    ) -> Result<(), CompileError> {
        self.rule.get_capturing_groups(count, map)
    }

    pub(crate) fn compile(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
    ) -> CompileResult<'i> {
        Ok(Regex::Repetition(Box::new(RegexRepetition {
            content: self.rule.comp(options, state)?,
            kind: self.kind,
            quantifier: self.quantifier,
        })))
    }
}

#[cfg(feature = "dbg")]
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
        if let Quantifier::Greedy = self.quantifier {
            write!(f, " greedy")?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "dbg", derive(Debug))]
#[non_exhaustive]
pub enum Quantifier {
    Greedy,
    Lazy,
}

/// A repetition in its most canonical form, `{x,y}`.
///
/// For example:
///
///  * `'x'?` is equivalent to `'x'{0,1}`
///  * `'x'+` is equivalent to `'x'{1,}`
///  * `'x'*` is equivalent to `'x'{0,}`
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub struct RepetitionKind {
    /// The lower bound, e.g. `{4,}`
    lower_bound: u32,

    /// The upper bound, e.g. `{0,7}`. `None` means infinity.
    upper_bound: Option<u32>,
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

        Ok(RepetitionKind { lower_bound, upper_bound })
    }
}

pub(crate) struct RegexRepetition<'i> {
    content: Regex<'i>,
    kind: RepetitionKind,
    quantifier: Quantifier,
}

impl<'i> RegexRepetition<'i> {
    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        use std::fmt::Write;

        if self.content.needs_parens_before_repetition() {
            buf.push_str("(?:");
            self.content.codegen(buf, flavor);
            buf.push(')');
        } else {
            self.content.codegen(buf, flavor);
        }

        let omit_lazy = match self.kind {
            RepetitionKind { lower_bound: 1, upper_bound: Some(1) } => return,
            RepetitionKind { lower_bound: 0, upper_bound: Some(1) } => {
                buf.push('?');
                false
            }
            RepetitionKind { lower_bound: 0, upper_bound: None } => {
                buf.push('*');
                false
            }
            RepetitionKind { lower_bound: 1, upper_bound: None } => {
                buf.push('+');
                false
            }
            RepetitionKind { lower_bound, upper_bound: None } => {
                write!(buf, "{{{lower_bound},}}").unwrap();
                false
            }
            RepetitionKind { lower_bound, upper_bound: Some(upper_bound) }
                if lower_bound == upper_bound =>
            {
                write!(buf, "{{{lower_bound}}}").unwrap();
                true
            }
            RepetitionKind { lower_bound: 0, upper_bound: Some(upper_bound) } => {
                write!(buf, "{{0,{upper_bound}}}").unwrap();
                false
            }
            RepetitionKind { lower_bound, upper_bound: Some(upper_bound) } => {
                write!(buf, "{{{lower_bound},{upper_bound}}}").unwrap();
                false
            }
        };

        if let Quantifier::Lazy = self.quantifier {
            if !omit_lazy {
                buf.push('?');
            }
        }
    }
}
