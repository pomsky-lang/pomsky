//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use crate::{compile::CompileResult, options::CompileOptions, regex::Regex};

/// The dot, matching anything except line breaks
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Dot {}

impl Dot {
    pub(crate) fn compile(&self, _: CompileOptions) -> CompileResult<'static> {
        Ok(Regex::Dot)
    }
}
