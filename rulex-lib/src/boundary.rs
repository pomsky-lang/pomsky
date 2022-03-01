//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use crate::{
    compile::{Compile, CompileResult, CompileState},
    options::CompileOptions,
};

/// A [word boundary](https://www.regular-expressions.info/wordboundaries.html) or
/// [anchor](https://www.regular-expressions.info/anchors.html), which we combine under the term
/// _boundary_.
///
/// All boundaries use a variation of the `%` sigil, so they are easy to remember.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    /// `<%`, the start of the string (or start of line in single-line mode)
    Start,
    /// `%`, a word boundary
    Word,
    /// `!%`, not a word boundary
    NotWord,
    /// `%>`, the end of the string (or end of line in single-line mode)
    End,
}

impl Compile for Boundary {
    fn comp(
        &self,
        _options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        match self {
            Boundary::Start => buf.push('^'),
            Boundary::Word => buf.push_str("\\b"),
            Boundary::NotWord => buf.push_str("\\B"),
            Boundary::End => buf.push('$'),
        }
        Ok(())
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Start => write!(f, "<%"),
            Self::Word => write!(f, "%"),
            Self::NotWord => write!(f, "!%"),
            Self::End => write!(f, "%>"),
        }
    }
}
