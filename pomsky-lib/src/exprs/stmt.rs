use pomsky_syntax::exprs::{BooleanSetting, Stmt, StmtExpr};

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
};

use super::{repetition::RegexQuantifier, RuleExt};

impl<'i> RuleExt<'i> for StmtExpr<'i> {
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
}
