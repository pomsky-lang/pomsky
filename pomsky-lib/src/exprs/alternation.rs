//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::{Alternation, RuleExt};

impl<'i> RuleExt<'i> for Alternation<'i> {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
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
pub(crate) struct RegexAlternation<'i> {
    pub(crate) parts: Vec<Regex<'i>>,
}

impl<'i> RegexAlternation<'i> {
    pub(crate) fn new(parts: Vec<Regex<'i>>) -> Self {
        Self { parts }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        for rule in &self.parts {
            rule.codegen(buf, flavor);
            buf.push('|');
        }
        let _ = buf.pop();
    }
}
