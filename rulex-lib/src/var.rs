use crate::{
    compile::{CompileResult, CompileState},
    error::CompileErrorKind,
    options::CompileOptions,
    span::Span,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Variable<'i> {
    name: &'i str,
    pub(crate) span: Span,
}

impl<'i> Variable<'i> {
    pub(crate) fn new(name: &'i str, span: Span) -> Self {
        Variable { name, span }
    }

    pub(crate) fn compile<'c>(
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
            let res = rule.comp(options, state)?;
            state.current_vars.remove(&i);
            Ok(res)
        } else {
            let recursive_rule = state.variables.iter().rev().find(|&&(name, _)| name == self.name);
            if recursive_rule.is_some() {
                Err(CompileErrorKind::RecursiveVariable.at(self.span))
            } else {
                Err(CompileErrorKind::UnknownVariable.at(self.span))
            }
        }
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for Variable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variable({})", self.name)
    }
}
