use pomsky_syntax::exprs::Recursion;

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl RuleExt for Recursion {
    fn compile(&self, _options: CompileOptions, _: &mut CompileState<'_>) -> CompileResult {
        Ok(Regex::Recursion)
    }
}

pub(crate) fn codegen(buf: &mut String, _flavor: RegexFlavor) {
    buf.push_str("\\g<0>")
}
