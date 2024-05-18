use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub span: Span,
}

impl Variable {
    pub(crate) fn new(name: &str, span: Span) -> Self {
        Variable { name: name.to_string(), span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.write(&self.name);
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for Variable {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let name = super::arbitrary::Ident::create(u)?;
        Ok(Variable { name, span: Span::arbitrary(u)? })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        super::arbitrary::Ident::size_hint(depth)
    }
}
