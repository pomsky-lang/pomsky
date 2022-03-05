//! Contains the [`Grapheme`] type, which matches a
//! [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).

use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
};

/// The `X` expression, matching a
/// [Unicode grapheme](https://www.regular-expressions.info/unicode.html#grapheme).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub struct Grapheme;

impl Compile for Grapheme {
    fn comp(
        &self,
        options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        if options.flavor == RegexFlavor::JavaScript {
            return Err(CompileError::Unsupported(Feature::Grapheme, options.flavor));
        }
        buf.push_str("\\X");
        Ok(())
    }
}
