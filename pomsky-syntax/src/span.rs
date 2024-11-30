use std::{
    fmt::{Debug, Display},
    ops::Range,
};

/// A source code location, marked by the start and end byte offset. If both are
/// zero, this is considered as "empty" or "missing", and [`Span::range`]
/// returns `None`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    /// Constructs a new [`Span`]. If both `start` and `end` is 0, this is
    /// considered as "empty" or "missing"
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Span { start: start as u32, end: end as u32 }
    }

    /// Constructs an empty [`Span`].
    #[must_use]
    pub fn empty() -> Self {
        Span { start: 0, end: 0 }
    }

    /// Returns whether this span is "empty" or "missing".
    pub fn is_empty(&self) -> bool {
        self.end == 0
    }

    /// Converts this span to a [`std::ops::Range`]. If it is empty, `None` is
    /// returned.
    pub fn range(self) -> Option<Range<usize>> {
        if self.is_empty() {
            None
        } else {
            Some(self.start as usize..self.end as usize)
        }
    }

    /// Converts this span to a [`std::ops::Range`], without checking if it is
    /// empty.
    pub fn range_unchecked(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// Returns a new span that points to this span's start, but has a length of
    /// 0.
    pub fn start(&self) -> Span {
        Span { start: self.start, end: self.start }
    }

    pub fn join(self, other: Span) -> Span {
        match (self.is_empty(), other.is_empty()) {
            (false, false) => Span {
                start: u32::min(self.start, other.start),
                end: u32::max(self.end, other.end),
            },
            (false, true) => self,
            (true, false) => other,
            (true, true) => Span::empty(),
        }
    }

    pub(crate) fn join_unchecked(self, other: Span) -> Span {
        Span { start: u32::min(self.start, other.start), end: u32::max(self.end, other.end) }
    }
}

impl From<Range<usize>> for Span {
    fn from(Range { start, end }: Range<usize>) -> Self {
        Span { start: start as u32, end: end as u32 }
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

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for Span {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Span { start: 0, end: 0 })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
