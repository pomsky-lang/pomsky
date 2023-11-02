use pomsky_syntax::exprs::Recursion;

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Recursion {
    fn compile<'c>(
        &'c self,
        _options: CompileOptions,
        _: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        Ok(Regex::Recursion)
    }
}

pub(crate) fn codegen(buf: &mut String, _flavor: RegexFlavor) {
    buf.push_str("(?R)")
}
