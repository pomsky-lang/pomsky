//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use crate::{
    compile::CompileResult,
    error::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    span::Span,
};

/// The `Grapheme` expression, matching a
/// [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub struct Grapheme {
    pub(crate) span: Span,
}

impl Grapheme {
    pub(crate) fn compile(&self, options: CompileOptions) -> CompileResult<'static> {
        if options.flavor == RegexFlavor::JavaScript {
            return Err(
                CompileErrorKind::Unsupported(Feature::Grapheme, options.flavor).at(self.span)
            );
        }
        Ok(Regex::Grapheme)
    }
}
