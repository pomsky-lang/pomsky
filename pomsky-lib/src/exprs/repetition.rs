use pomsky_syntax::exprs::{Quantifier, Repetition, RepetitionKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::Compile;

impl Compile for Repetition {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        let content = self.rule.compile(options, state)?;

        if options.flavor == RegexFlavor::Ruby && content.is_assertion() {
            return Err(CompileErrorKind::Unsupported(Feature::RepeatedAssertion, options.flavor)
                .at(self.span));
        }

        let quantifier = match self.quantifier {
            Quantifier::Greedy | Quantifier::DefaultGreedy => RegexQuantifier::Greedy,
            Quantifier::Lazy | Quantifier::DefaultLazy => RegexQuantifier::Lazy,
        };

        Ok(Regex::Repetition(Box::new(RegexRepetition { content, kind: self.kind, quantifier })))
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexRepetition {
    pub(crate) content: Regex,
    pub(crate) kind: RepetitionKind,
    pub(crate) quantifier: RegexQuantifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegexQuantifier {
    Greedy,
    Lazy,
}

impl RegexRepetition {
    pub(crate) fn new(content: Regex, kind: RepetitionKind, quantifier: RegexQuantifier) -> Self {
        Self { content, kind, quantifier }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        use std::fmt::Write;

        if let Regex::Literal(l) = &self.content {
            if l.is_empty() {
                return;
            }
        }

        if self.content.needs_parens_before_repetition(flavor) {
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
