//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use pomsky_syntax::Span;

use crate::{
    compile::CompileResult,
    diagnose::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

/// The `Grapheme` expression, matching a
/// [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Grapheme {}

impl Grapheme {
    pub(crate) fn compile(&self, options: CompileOptions) -> CompileResult {
        if matches!(options.flavor, RegexFlavor::Pcre | RegexFlavor::Java | RegexFlavor::Ruby) {
            Ok(Regex::Grapheme)
        } else {
            Err(CompileErrorKind::Unsupported(Feature::Grapheme, options.flavor).at(Span::empty()))
        }
    }
}
