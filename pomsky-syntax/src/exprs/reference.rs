//! Implements ([relative](https://www.regular-expressions.info/backrefrel.html))
//! [backreferences](https://www.regular-expressions.info/backref.html),
//! [forward references](https://www.regular-expressions.info/backref2.html#forward) and
//! [named references](https://www.regular-expressions.info/named.html).

use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Reference {
    pub target: ReferenceTarget,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTarget {
    Named(String),
    Number(u32),
    Relative(i32),
}

impl Reference {
    pub(crate) fn new(target: ReferenceTarget, span: Span) -> Self {
        Reference { target, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.push_str("::");
        match &self.target {
            ReferenceTarget::Named(n) => buf.write(n),
            ReferenceTarget::Number(i) => buf.write_fmt(i),
            &ReferenceTarget::Relative(o) => {
                buf.push(if o < 0 { '-' } else { '+' });
                buf.write_fmt(o);
            }
        }
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for ReferenceTarget {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0u8..=2)? {
            0 => ReferenceTarget::Named(super::arbitrary::Ident::create(u)?),
            1 => ReferenceTarget::Number(u.int_in_range(0u8..=15)? as u32),
            _ => ReferenceTarget::Relative(u.int_in_range(-15i8..=15)? as i32),
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(super::arbitrary::Ident::size_hint(depth), (3, Some(3)))
    }
}
