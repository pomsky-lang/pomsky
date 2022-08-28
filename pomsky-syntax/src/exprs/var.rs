use crate::Span;

#[derive(Clone, PartialEq, Eq)]
pub struct Variable<'i> {
    pub name: &'i str,
    pub span: Span,
}

impl<'i> Variable<'i> {
    pub(crate) fn new(name: &'i str, span: Span) -> Self {
        Variable { name, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.write(self.name);
    }
}
