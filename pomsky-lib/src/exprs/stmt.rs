use std::collections::HashMap;

use pomsky_syntax::exprs::{BooleanSetting, Stmt, StmtExpr};

use crate::{
    compile::{CompileResult, CompileState, ValidationState},
    diagnose::{CompileError, CompileErrorKind},
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
            Stmt::Enable(..) | Stmt::Disable(..) => {
                let prev_quantifier = state.default_quantifier;
                let prev_ascii = state.ascii_only;
                match &self.stmt {
                    Stmt::Enable(BooleanSetting::Lazy, _) => {
                        state.default_quantifier = RegexQuantifier::Lazy;
                    }
                    Stmt::Disable(BooleanSetting::Lazy, _) => {
                        state.default_quantifier = RegexQuantifier::Greedy;
                    }
                    Stmt::Enable(BooleanSetting::Unicode, _) => {
                        state.ascii_only = false;
                    }
                    Stmt::Disable(BooleanSetting::Unicode, _) => {
                        state.ascii_only = true;
                    }
                    Stmt::Let(_) | Stmt::Test(_) => unreachable!(),
                }
                let res = self.rule.compile(options, state)?;
                state.default_quantifier = prev_quantifier;
                state.ascii_only = prev_ascii;
                Ok(res)
            }
            Stmt::Let(r#let) => {
                state.variables.push((r#let.name, &r#let.rule));
                let res = self.rule.compile(options, state)?;
                state.variables.pop();
                Ok(res)
            }
            Stmt::Test(_) => self.rule.compile(options, state),
        }
    }

    fn validate(
        &self,
        options: &CompileOptions,
        state: &mut ValidationState,
    ) -> Result<(), CompileError> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy, span) => {
                options.allowed_features.require(PomskyFeatures::LAZY_MODE, *span)?;
            }
            Stmt::Disable(BooleanSetting::Unicode, span) => {
                options.allowed_features.require(PomskyFeatures::ASCII_MODE, *span)?;
            }
            Stmt::Enable(..) | Stmt::Disable(..) => {}
            Stmt::Let(l) => {
                options.allowed_features.require(PomskyFeatures::VARIABLES, l.name_span)?;
                l.rule.validate(options, &mut state.layer_down())?;
            }
            Stmt::Test(t) => {
                if !state.is_top_layer {
                    return Err(CompileErrorKind::NestedTest.at(t.span));
                }
            }
        }

        self.rule.validate(options, state)
    }
}
