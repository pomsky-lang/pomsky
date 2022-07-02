use std::{
    fmt::{Debug, Display},
    ops::Range,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub(crate) fn empty() -> Self {
        Span { start: 0, end: 0 }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.end == 0
    }

    pub fn range(self) -> Option<Range<usize>> {
        if self.is_empty() {
            None
        } else {
            Some(self.start..self.end)
        }
    }

    pub(crate) fn range_unchecked(self) -> Range<usize> {
        self.start..self.end
    }

    pub(crate) fn start(&self) -> Span {
        Span { start: self.start, end: self.start }
    }

    pub(crate) fn join(self, other: Span) -> Span {
        match (self.is_empty(), other.is_empty()) {
            (false, false) => Span {
                start: usize::min(self.start, other.start),
                end: usize::max(self.end, other.end),
            },
            (false, true) => self,
            (true, false) => other,
            (true, true) => Span::empty(),
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
        Span::empty()
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
