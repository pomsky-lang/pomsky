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
            Stmt::Enable(_) | Stmt::Disable(_) => {
                let prev_quantifier = state.default_quantifier;
                let prev_ascii = state.ascii_only;
                match &self.stmt {
                    Stmt::Enable(BooleanSetting::Lazy) => {
                        state.default_quantifier = RegexQuantifier::Lazy;
                    }
                    Stmt::Disable(BooleanSetting::Lazy) => {
                        state.default_quantifier = RegexQuantifier::Greedy;
                    }
                    Stmt::Enable(BooleanSetting::Unicode) => {
                        state.ascii_only = false;
                    }
                    Stmt::Disable(BooleanSetting::Unicode) => {
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

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => {
                options.allowed_features.require(PomskyFeatures::LAZY_MODE, self.span)?;
            }
            Stmt::Disable(BooleanSetting::Unicode) => {
                options.allowed_features.require(PomskyFeatures::ASCII_MODE, self.span)?;
            }
            Stmt::Enable(_) | Stmt::Disable(_) | Stmt::Test(_) => {}
            Stmt::Let(l) => {
                options.allowed_features.require(PomskyFeatures::VARIABLES, l.name_span)?;
                l.rule.validate(options)?;
            }
        }

        self.rule.validate(options)
    }
}
