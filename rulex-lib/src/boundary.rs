//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use crate::{compile::CompileResult, regex::Regex, span::Span};

/// A [word boundary](https://www.regular-expressions.info/wordboundaries.html) or
/// [anchor](https://www.regular-expressions.info/anchors.html), which we combine under the term
/// _boundary_.
///
/// All boundaries use a variation of the `%` sigil, so they are easy to remember.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Boundary {
    kind: BoundaryKind,
    pub(crate) span: Span,
}

impl Boundary {
    pub(crate) fn new(kind: BoundaryKind, span: Span) -> Self {
        Boundary { kind, span }
    }
}

impl Boundary {
    pub(crate) fn compile(&self) -> CompileResult<'static> {
        Ok(Regex::Boundary(self.kind))
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundaryKind {
    /// `<%`, the start of the string (or start of line in single-line mode)
    Start,
    /// `%`, a word boundary
    Word,
    /// `!%`, not a word boundary
    NotWord,
    /// `%>`, the end of the string (or end of line in single-line mode)
    End,
}

impl BoundaryKind {
    pub(crate) fn codegen(&self, buf: &mut String) {
        match self {
            BoundaryKind::Start => buf.push('^'),
            BoundaryKind::Word => buf.push_str("\\b"),
            BoundaryKind::NotWord => buf.push_str("\\B"),
            BoundaryKind::End => buf.push('$'),
        }
    }
}
