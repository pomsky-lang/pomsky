//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use crate::{error::ParseErrorKind, Span};

/// A [word boundary](https://www.regular-expressions.info/wordboundaries.html) or
/// [anchor](https://www.regular-expressions.info/anchors.html), which we combine under the term
/// _boundary_.
///
/// All boundaries use a variation of the `%` sigil, so they are easy to
/// remember.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Boundary {
    pub kind: BoundaryKind,
    pub span: Span,
}

impl Boundary {
    pub fn new(kind: BoundaryKind, span: Span) -> Self {
        Boundary { kind, span }
    }

    pub fn kind(&self) -> BoundaryKind {
        self.kind
    }

    pub(crate) fn negate(&mut self) -> Result<(), ParseErrorKind> {
        match self.kind {
            BoundaryKind::Start | BoundaryKind::End => Err(ParseErrorKind::UnallowedNot),
            BoundaryKind::NotWord => Err(ParseErrorKind::UnallowedDoubleNot),
            BoundaryKind::Word => {
                self.kind = BoundaryKind::NotWord;
                Ok(())
            }
        }
    }
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            BoundaryKind::Start => write!(f, "Start"),
            BoundaryKind::Word => write!(f, "%"),
            BoundaryKind::NotWord => write!(f, "!%"),
            BoundaryKind::End => write!(f, "End"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "pretty-print", derive(Debug))]
pub enum BoundaryKind {
    /// `Start`, the start of the string (or start of line in single-line mode)
    Start,
    /// `%`, a word boundary
    Word,
    /// `!%`, not a word boundary
    NotWord,
    /// `End`, the end of the string (or end of line in single-line mode)
    End,
}
