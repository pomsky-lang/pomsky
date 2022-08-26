use pomsky_syntax::exprs::Variable;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileErrorKind,
    options::CompileOptions,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Variable<'i> {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        let rule = state
            .variables
            .iter()
            .enumerate()
            .rev()
            .find(|&(i, &(name, _))| name == self.name && !state.current_vars.contains(&i));

        if let Some((i, &(_, rule))) = rule {
            state.current_vars.insert(i);
            let res = rule.compile(options, state)?;
            state.current_vars.remove(&i);
            Ok(res)
        } else {
            let recursive_rule = state.variables.iter().rev().find(|&&(name, _)| name == self.name);
            if recursive_rule.is_some() {
                Err(CompileErrorKind::RecursiveVariable.at(self.span))
            } else {
                Err(CompileErrorKind::UnknownVariable {
                    found: self.name.into(),
                    #[cfg(feature = "suggestions")]
                    similar: pomsky_syntax::find_suggestion(
                        self.name,
                        state.variables.iter().map(|&(var, _)| var),
                    ),
                }
                .at(self.span))
            }
        }
    }
}
