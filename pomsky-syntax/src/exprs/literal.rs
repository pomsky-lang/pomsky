use crate::Span;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Literal {
    pub content: String,
    pub span: Span,
}

impl Literal {
    pub fn new(content: String, span: Span) -> Self {
        Literal { content, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.write_debug(&self.content);
    }
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}", self.content, self.span)
    }
}
