use pomsky_syntax::exprs::Regex as RegexLiteral;

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for RegexLiteral<'i> {
    fn compile<'c>(&'c self, _: CompileOptions, _: &mut CompileState<'c, 'i>) -> CompileResult<'i> {
        Ok(Regex::Unescaped(self.content.clone()))
    }
}
