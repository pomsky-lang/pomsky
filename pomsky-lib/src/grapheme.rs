//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use crate::{
    compile::CompileResult,
    error::{CompileErrorKind, Feature, ParseError},
    features::PomskyFeatures,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    regex::Regex,
    span::Span,
};

/// The `Grapheme` expression, matching a
/// [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Grapheme;

impl Grapheme {
    pub(crate) fn compile(&self, options: CompileOptions) -> CompileResult<'static> {
        if matches!(options.flavor, RegexFlavor::Pcre | RegexFlavor::Java | RegexFlavor::Ruby) {
            Ok(Regex::Grapheme)
        } else {
            Err(CompileErrorKind::Unsupported(Feature::Grapheme, options.flavor).at(Span::empty()))
        }
    }

    pub(crate) fn validate(&self, options: &ParseOptions) -> Result<(), ParseError> {
        options.allowed_features.require(PomskyFeatures::GRAPHEME, Span::empty())?;
        Ok(())
    }
}
