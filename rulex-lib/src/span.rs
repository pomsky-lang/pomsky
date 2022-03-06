use std::{
    fmt::{Debug, Display},
    ops::Range,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn range(self) -> Option<Range<usize>> {
        if self.start > self.end {
            None
        } else {
            Some(self.start..self.end)
        }
    }

    pub(crate) fn join(self, other: Span) -> Span {
        Span {
            start: usize::min(self.start, other.start),
            end: usize::max(self.end, other.end),
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(Range { start, end }: Range<usize>) -> Self {
        Span { start, end }
    }
}

impl Default for Span {
    fn default() -> Self {
        Span {
            start: usize::MAX,
            end: 0,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Span({}..{})", self.start, self.end)
    }
}
