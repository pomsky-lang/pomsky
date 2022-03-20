//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use crate::{
    compile::{Compile, CompileResult, CompileState, Transform, TransformState},
    options::CompileOptions,
    span::Span,
};

/// A [word boundary](https://www.regular-expressions.info/wordboundaries.html) or
/// [anchor](https://www.regular-expressions.info/anchors.html), which we combine under the term
/// _boundary_.
///
/// All boundaries use a variation of the `%` sigil, so they are easy to remember.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Boundary {
    kind: BoundaryKind,
    pub(crate) span: Span,
}

impl Boundary {
    pub(crate) fn new(kind: BoundaryKind, span: Span) -> Self {
        Boundary { kind, span }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind {
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
        match self.kind {
            BoundaryKind::Start => buf.push('^'),
            BoundaryKind::Word => buf.push_str("\\b"),
            BoundaryKind::NotWord => buf.push_str("\\B"),
            BoundaryKind::End => buf.push('$'),
        }
        Ok(())
    }
}

impl Transform for Boundary {
    fn transform(&mut self, _: CompileOptions, _: &mut TransformState) -> CompileResult {
        Ok(())
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            BoundaryKind::Start => write!(f, "<%"),
            BoundaryKind::Word => write!(f, "%"),
            BoundaryKind::NotWord => write!(f, "!%"),
            BoundaryKind::End => write!(f, "%>"),
        }
    }
}
