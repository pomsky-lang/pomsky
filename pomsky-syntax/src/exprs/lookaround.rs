use crate::{error::ParseErrorKind, Span};

use super::Rule;

#[derive(Clone)]
pub struct Lookaround<'i> {
    pub kind: LookaroundKind,
    pub rule: Rule<'i>,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
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
                Err(ParseErrorKind::UnallowedMultiNot(2))
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

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        let s = match self.kind {
            LookaroundKind::Ahead => ">>",
            LookaroundKind::Behind => "<<",
            LookaroundKind::AheadNegative => "!>>",
            LookaroundKind::BehindNegative => "!<<",
        };
        if needs_parens {
            buf.push('(');
            buf.start_indentation(s);
        } else {
            buf.push_str(s);
            buf.push(' ');
        }

        self.rule.pretty_print(buf, false);

        if needs_parens {
            buf.end_indentation(")");
        }
    }
}
