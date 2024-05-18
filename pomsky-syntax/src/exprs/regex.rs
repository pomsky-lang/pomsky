use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Regex {
    pub content: String,
    pub span: Span,
}

impl Regex {
    pub(crate) fn new(content: String, span: Span) -> Self {
        Regex { content, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.push_str("regex ");
        write!(buf, "{:?}", self.content);
    }
}
