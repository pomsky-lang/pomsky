use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    options::CompileOptions,
    repetition::RegexQuantifier,
    span::Span,
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Modified<'i> {
    modifier: Modifier,
    rule: Rulex<'i>,
    pub(crate) span: Span,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Modifier {
    Enable(BooleanSetting),
    Disable(BooleanSetting),
}

#[derive(Clone, PartialEq, Eq)]
pub enum BooleanSetting {
    Lazy,
}

impl<'i> Modified<'i> {
    pub(crate) fn new(modifier: Modifier, rule: Rulex<'i>, span: Span) -> Self {
        Self { modifier, rule, span }
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
        match self.modifier {
            Modifier::Enable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Lazy;
                let res = self.rule.comp(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
            Modifier::Disable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Greedy;
                let res = self.rule.comp(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
        }
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for Modified<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Modified")
            .field(match self.modifier {
                Modifier::Enable(BooleanSetting::Lazy) => &"enable lazy",
                Modifier::Disable(BooleanSetting::Lazy) => &"disable lazy",
            })
            .field(&self.rule)
            .finish()
    }
}
