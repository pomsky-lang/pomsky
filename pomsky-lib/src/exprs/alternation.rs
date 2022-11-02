//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::{Alternation, RuleExt};

impl<'i> RuleExt<'i> for Alternation<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        for rule in &self.rules {
            rule.get_capturing_groups(count, map, within_variable)?;
        }
        Ok(())
    }

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

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        for rule in &self.rules {
            rule.validate(options)?;
        }
        Ok(())
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
