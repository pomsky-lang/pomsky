use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    group::{RegexCapture, RegexGroup},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    rule::Rule,
    span::Span,
};

#[derive(Clone)]
pub(crate) struct Repetition<'i> {
    rule: Rule<'i>,
    kind: RepetitionKind,
    quantifier: Quantifier,
    pub(crate) span: Span,
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

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    pub(crate) fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        let mut content = self.rule.comp(options, state)?;

        if let RepetitionKind { lower_bound: 0, upper_bound: Some(1) } = self.kind {
            if let Rule::Repetition(_) = &self.rule {
                content =
                    Regex::Group(RegexGroup::new(vec![content], RegexCapture::NoneWithParens));
            }
        }

        let quantifier = match self.quantifier {
            Quantifier::Greedy => RegexQuantifier::Greedy,
            Quantifier::Lazy => RegexQuantifier::Lazy,
            Quantifier::Default => state.default_quantifier,
        };

        Ok(Regex::Repetition(Box::new(RegexRepetition { content, kind: self.kind, quantifier })))
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
        match self.quantifier {
            Quantifier::Greedy => write!(f, " greedy")?,
            Quantifier::Lazy => write!(f, " lazy")?,
            Quantifier::Default => {}
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "dbg", derive(Debug))]
#[non_exhaustive]
pub(crate) enum Quantifier {
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
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RepetitionKind {
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
    #[error("Unexpected `?` following a repetition")]
    QuestionMarkAfterRepetition,
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
    quantifier: RegexQuantifier,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RegexQuantifier {
    Greedy,
    Lazy,
}

impl<'i> RegexRepetition<'i> {
    pub(crate) fn new(
        content: Regex<'i>,
        kind: RepetitionKind,
        quantifier: RegexQuantifier,
    ) -> Self {
        Self { content, kind, quantifier }
    }

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

        if let RegexQuantifier::Lazy = self.quantifier {
            if !omit_lazy {
                buf.push('?');
            }
        }
    }
}
