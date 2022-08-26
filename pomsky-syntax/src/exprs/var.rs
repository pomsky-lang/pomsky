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
}

#[cfg(feature = "pretty-print")]
impl std::fmt::Debug for Variable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variable({})", self.name)
    }
}
