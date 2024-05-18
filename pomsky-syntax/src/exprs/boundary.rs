//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use crate::Span;

/// A [word boundary](https://www.regular-expressions.info/wordboundaries.html) or
/// [anchor](https://www.regular-expressions.info/anchors.html), which we combine under the term
/// _boundary_.
///
/// All boundaries use a variation of the `%` sigil, so they are easy to
/// remember.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Boundary {
    pub kind: BoundaryKind,
    pub unicode_aware: bool,
    pub span: Span,
}

impl Boundary {
    pub fn new(kind: BoundaryKind, unicode_aware: bool, span: Span) -> Self {
        Boundary { kind, unicode_aware, span }
    }

    pub fn kind(&self) -> BoundaryKind {
        self.kind
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        match self.kind {
            BoundaryKind::Start => buf.push('^'),
            BoundaryKind::End => buf.push('$'),
            BoundaryKind::Word => buf.push('%'),
            BoundaryKind::NotWord => buf.push_str("!%"),
            BoundaryKind::WordStart => buf.push_str("<"),
            BoundaryKind::WordEnd => buf.push_str(">"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum BoundaryKind {
    /// `Start`, the start of the string (or start of line in single-line mode)
    Start,
    /// `End`, the end of the string (or end of line in single-line mode)
    End,
    /// `%`, a word boundary
    Word,
    /// `!%`, not a word boundary
    NotWord,
    /// `<` the beginning of a word
    WordStart,
    /// `>` the end of a word
    WordEnd,
}
