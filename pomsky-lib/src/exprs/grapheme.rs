//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use pomsky_syntax::Span;

use crate::{
    compile::CompileResult,
    diagnose::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

/// The `Grapheme` expression, matching a
/// [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Grapheme {}

impl Grapheme {
    pub(crate) fn compile(&self, options: CompileOptions) -> CompileResult<'static> {
        if matches!(options.flavor, RegexFlavor::Pcre | RegexFlavor::Java | RegexFlavor::Ruby) {
            Ok(Regex::Grapheme)
        } else {
            Err(CompileErrorKind::Unsupported(Feature::Grapheme, options.flavor).at(Span::empty()))
        }
    }

    pub(crate) fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        options.allowed_features.require(PomskyFeatures::GRAPHEME, Span::empty())?;
        Ok(())
    }
}
