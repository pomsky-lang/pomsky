use crate::{error::ParseErrorKind, Span};

use super::Rule;

#[derive(Clone)]
pub struct Lookaround<'i> {
    pub kind: LookaroundKind,
    pub rule: Rule<'i>,
    pub span: Span,
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Lookaround<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Lookaround ")?;
        f.write_str(match self.kind {
            LookaroundKind::Ahead => ">> ",
            LookaroundKind::Behind => "<< ",
            LookaroundKind::AheadNegative => "!>> ",
            LookaroundKind::BehindNegative => "!<< ",
        })?;
        self.rule.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "pretty-print", derive(Debug))]
pub enum LookaroundKind {
    Ahead,
    Behind,
    AheadNegative,
    BehindNegative,
}

impl<'i> Lookaround<'i> {
    pub(crate) fn new(rule: Rule<'i>, kind: LookaroundKind, span: Span) -> Self {
        Lookaround { rule, kind, span }
    }

    pub(crate) fn negate(&mut self) -> Result<(), ParseErrorKind> {
        match self.kind {
            LookaroundKind::AheadNegative | LookaroundKind::BehindNegative => {
                Err(ParseErrorKind::UnallowedDoubleNot)
            }
            LookaroundKind::Ahead => {
                self.kind = LookaroundKind::AheadNegative;
                Ok(())
            }
            LookaroundKind::Behind => {
                self.kind = LookaroundKind::BehindNegative;
                Ok(())
            }
        }
    }
}
