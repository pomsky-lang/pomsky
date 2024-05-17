use crate::Span;

use super::Rule;

#[derive(Debug, Clone)]
pub struct Negation<'i> {
    pub rule: Rule<'i>,
    pub not_span: Span,
}

impl<'i> Negation<'i> {
    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        buf.push('!');
        if needs_parens {
            buf.start_indentation("(");
        }

        self.rule.pretty_print(buf, false);

        if needs_parens {
            buf.end_indentation(")");
        }
    }
}
