//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use crate::{
    compile::CompileResult,
    options::CompileOptions,
    regex::{Regex, RegexShorthand},
};

use super::char_class::{RegexCharSet, RegexCharSetItem};

/// The `Codepoint` expression, matching an arbitrary Unicode code point.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Codepoint {}

impl Codepoint {
    pub(crate) fn compile(&self, _options: CompileOptions) -> CompileResult {
        Ok(Regex::CharSet(RegexCharSet::new(vec![
            RegexCharSetItem::Shorthand(RegexShorthand::Space),
            RegexCharSetItem::Shorthand(RegexShorthand::NotSpace),
        ])))
    }
}
