use std::borrow::Cow;

use crate::Span;

#[derive(Clone, PartialEq, Eq)]
pub struct Literal<'i> {
    pub content: Cow<'i, str>,
    pub(crate) span: Span,
}

impl<'i> Literal<'i> {
    pub fn new(content: Cow<'i, str>, span: Span) -> Self {
        Literal { content, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.write_debug(&self.content);
    }
}
