use std::{borrow::Cow, collections::HashMap};

use pomsky_syntax::exprs::{Quantifier, Repetition, RepetitionKind};

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Repetition<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        let content = self.rule.compile(options, state)?;

        let quantifier = match self.quantifier {
            Quantifier::Greedy => RegexQuantifier::Greedy,
            Quantifier::Lazy => RegexQuantifier::Lazy,
            Quantifier::Default => state.default_quantifier,
        };

        Ok(Regex::Repetition(Box::new(RegexRepetition { content, kind: self.kind, quantifier })))
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        self.rule.validate(options)
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexRepetition<'i> {
    pub(crate) content: Regex<'i>,
    pub(crate) kind: RepetitionKind,
    pub(crate) quantifier: RegexQuantifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

        if let Regex::Literal(Cow::Borrowed("")) = self.content {
            return;
        }

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
