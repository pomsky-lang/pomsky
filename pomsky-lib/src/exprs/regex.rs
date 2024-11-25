use pomsky_syntax::exprs::Regex as RegexLiteral;

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
    regex::Regex,
};

use super::Compile;

impl Compile for RegexLiteral {
    fn compile(&self, _: CompileOptions, _: &mut CompileState<'_>) -> CompileResult {
        Ok(Regex::Unescaped(self.content.clone()))
    }
}
