use pomsky_syntax::exprs::Recursion;

use crate::{
    compile::{CompileResult, CompileState, ValidationState},
    diagnose::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Recursion {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        _: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match options.flavor {
            RegexFlavor::Pcre | RegexFlavor::Ruby => Ok(Regex::Recursion),
            _ => {
                Err(CompileErrorKind::Unsupported(Feature::Recursion, options.flavor).at(self.span))
            }
        }
    }

    fn validate(
        &self,
        options: &CompileOptions,
        _: &mut ValidationState,
    ) -> Result<(), CompileError> {
        options.allowed_features.require(PomskyFeatures::RECURSION, self.span)
    }
}

pub(crate) fn codegen(buf: &mut String, _flavor: RegexFlavor) {
    buf.push_str("(?R)")
}
