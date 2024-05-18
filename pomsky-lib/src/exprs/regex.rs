use pomsky_syntax::exprs::Regex as RegexLiteral;

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
    regex::Regex,
};

use super::RuleExt;

impl RuleExt for RegexLiteral {
    fn compile(&self, _: CompileOptions, _: &mut CompileState<'_>) -> CompileResult {
        Ok(Regex::Unescaped(self.content.clone()))
    }
}
