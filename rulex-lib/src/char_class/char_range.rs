use std::cmp::Ordering;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct CharRange {
    pub(super) first: char,
    pub(super) last: char,
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for CharRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.first == self.last {
            self.first.fmt(f)
        } else {
            write!(f, "{:?}-{:?}", self.first, self.last)
        }
    }
}

impl PartialOrd for CharRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.last < other.first {
            Some(Ordering::Less)
        } else if self.first > other.last {
            Some(Ordering::Greater)
        } else if self == other {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

/// This type of range is equal to any range it overlaps with!
#[derive(Clone, Copy, Eq)]
pub(super) struct SortRange(pub(super) CharRange);

#[cfg(feature = "dbg")]
impl core::fmt::Debug for SortRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for SortRange {
    #[allow(clippy::double_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        // clippy complains that this can be simplified.
        // this is a false positive, because CharRange does not impl Eq
        !(self.0 < other.0 || self.0 > other.0)
    }
}

impl PartialOrd for SortRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.last < other.0.first {
            Some(Ordering::Less)
        } else if self.0.first > other.0.last {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for SortRange {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.last < other.0.first {
            Ordering::Less
        } else if self.0.first > other.0.last {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
