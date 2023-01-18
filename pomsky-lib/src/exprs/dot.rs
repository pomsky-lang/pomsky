//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use pomsky_syntax::Span;

use crate::{
    compile::CompileResult, diagnose::CompileError, features::PomskyFeatures,
    options::CompileOptions, regex::Regex,
};

/// The dot, matching anything except line breaks
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Dot {}

impl Dot {
    pub(crate) fn compile(&self, _: CompileOptions) -> CompileResult<'static> {
        Ok(Regex::Dot)
    }

    pub(crate) fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        options.allowed_features.require(PomskyFeatures::DOT, Span::empty())?;
        Ok(())
    }
}
