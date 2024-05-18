use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: Box<[u8]>,
    pub end: Box<[u8]>,
    pub radix: u8,
    pub span: Span,
}

impl Range {
    pub(crate) fn new(start: Box<[u8]>, end: Box<[u8]>, radix: u8, span: Span) -> Self {
        Range { start, end, radix, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        fn hex(n: u8) -> char {
            match n {
                0..=9 => (n + b'0') as char,
                _ => (n + (b'A' - 10)) as char,
            }
        }

        buf.push_str("range '");
        buf.extend(self.start.iter().map(|&n| hex(n)));
        buf.push_str("'-'");
        buf.extend(self.end.iter().map(|&n| hex(n)));
        buf.push('\'');

        if self.radix != 10 {
            buf.push_str(" base ");
            buf.write_fmt(self.radix);
        }
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for Range {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let radix = u.int_in_range(2..=36)?;
        let start = super::arbitrary::Digits::create(u, radix)?;
        let end = super::arbitrary::Digits::create(u, radix)?;
        if start.len() > end.len() || (start.len() == end.len() && start > end) {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        Ok(Range { start, end, radix, span: Span::arbitrary(u)? })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}
