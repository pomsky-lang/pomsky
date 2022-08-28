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

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.push_str("::");
        match self.target {
            ReferenceTarget::Named(n) => buf.write(n),
            ReferenceTarget::Number(i) => buf.write_fmt(i),
            ReferenceTarget::Relative(o) => {
                buf.push(if o < 0 { '-' } else { '+' });
                buf.write_fmt(o);
            }
        }
    }
}
