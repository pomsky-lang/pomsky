//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::{Alternation, RuleExt};

impl RuleExt for Alternation {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        Ok(Regex::Alternation(RegexAlternation {
            parts: self
                .rules
                .iter()
                .map(|rule| rule.compile(options, state))
                .collect::<Result<_, _>>()?,
        }))
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexAlternation {
    pub(crate) parts: Vec<Regex>,
}

impl RegexAlternation {
    pub(crate) fn new(parts: Vec<Regex>) -> Self {
        Self { parts }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        for rule in &self.parts {
            rule.codegen(buf, flavor);
            buf.push('|');
        }
        if !self.parts.is_empty() {
            let _ = buf.pop();
        }
    }
}
