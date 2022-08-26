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
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Literal<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}
