//! Implements ([relative](https://www.regular-expressions.info/backrefrel.html))
//! [backreferences](https://www.regular-expressions.info/backref.html),
//! [forward references](https://www.regular-expressions.info/backref2.html#forward) and
//! [named references](https://www.regular-expressions.info/named.html).

use crate::Span;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Reference<'i> {
    pub target: ReferenceTarget<'i>,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTarget<'i> {
    Named(&'i str),
    Number(u32),
    Relative(i32),
}

impl<'i> Reference<'i> {
    pub(crate) fn new(target: ReferenceTarget<'i>, span: Span) -> Self {
        Reference { target, span }
    }
}

#[cfg(feature = "pretty-print")]
impl std::fmt::Debug for Reference<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.target {
            ReferenceTarget::Named(n) => write!(f, "::{}", n),
            ReferenceTarget::Number(i) => write!(f, "::{}", i),
            ReferenceTarget::Relative(o) => write!(f, "::{}{}", if o < 0 { '-' } else { '+' }, o),
        }
    }
}
