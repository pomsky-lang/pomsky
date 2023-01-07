use std::collections::HashMap;

use pomsky_syntax::exprs::{BooleanSetting, Stmt, StmtExpr};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::CompileError,
    features::PomskyFeatures,
    options::CompileOptions,
};

use super::{repetition::RegexQuantifier, RuleExt};

impl<'i> RuleExt<'i> for StmtExpr<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        if let Stmt::Let(l) = &self.stmt {
            l.rule.get_capturing_groups(count, map, true)?;
        }
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Lazy;
                let res = self.rule.compile(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
            Stmt::Disable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Greedy;
                let res = self.rule.compile(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
            Stmt::Let(r#let) => {
                state.variables.push((r#let.name, &r#let.rule));
                let res = self.rule.compile(options, state)?;
                state.variables.pop();
                Ok(res)
            }
        }
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => {
                options.allowed_features.require(PomskyFeatures::LAZY_MODE, self.span)?;
            }
            Stmt::Disable(_) => {}
            Stmt::Let(l) => {
                options.allowed_features.require(PomskyFeatures::VARIABLES, l.name_span)?;
                l.rule.validate(options)?;
            }
        }

        self.rule.validate(options)
    }
}
