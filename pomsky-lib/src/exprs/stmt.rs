use pomsky_syntax::exprs::{Stmt, StmtExpr};

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for StmtExpr<'i> {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match &self.stmt {
            Stmt::Enable(..) | Stmt::Disable(..) => self.rule.compile(options, state),
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
