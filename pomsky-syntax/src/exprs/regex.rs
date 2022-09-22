use std::borrow::Cow;

use crate::Span;

#[derive(Clone, PartialEq, Eq)]
pub struct Regex<'i> {
    pub content: Cow<'i, str>,
    pub span: Span,
}

impl<'i> Regex<'i> {
    pub(crate) fn new(content: Cow<'i, str>, span: Span) -> Self {
        Regex { content, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.push_str("regex ");
        write!(buf, "{:?}", self.content);
    }
}
