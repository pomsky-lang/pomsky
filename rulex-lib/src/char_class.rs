use core::fmt;
use std::{cmp::Ordering, collections::BTreeSet};

#[derive(Clone, Eq, Default)]
pub struct CharClass<'i> {
    named_parts: Vec<&'i str>,
    ranges: BTreeSet<SortRange>,
    negated: bool,
}

impl PartialEq for CharClass<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.named_parts == other.named_parts
            && self.negated == other.negated
            && self
                .ranges
                .iter()
                .map(|e| e.0)
                .eq(other.ranges.iter().map(|e| e.0))
    }
}

impl fmt::Debug for CharClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = Vec::with_capacity(self.named_parts.len() + self.ranges.len());
        for &part in &self.named_parts {
            items.push(format!("<{part}>"));
        }
        for range in &self.ranges {
            items.push(format!("{range:?}"));
        }
        write!(f, "CharClass({})", items.join(" | "))?;
        if self.negated {
            write!(f, " negated")?;
        }
        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    first: char,
    last: char,
}

impl fmt::Debug for CharRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
struct SortRange(CharRange);

impl fmt::Debug for SortRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<'i> CharClass<'i> {
    pub fn from_chars(chars: &str) -> Self {
        let mut new = CharClass::default();
        new.add_ranges(chars);
        new
    }

    pub fn from_char(c: char) -> Self {
        let mut new = CharClass::default();
        new.add_range(c, c);
        new
    }

    pub fn try_from_range(start: char, end: char) -> Option<Self> {
        let mut new = CharClass::default();
        if start <= end {
            new.add_range(start, end);
            Some(new)
        } else {
            None
        }
    }

    pub fn from_named(name: &'i str) -> Self {
        let mut new = CharClass::default();
        new.add_named(name);
        new
    }

    pub fn add_named(&mut self, new: &'i str) {
        self.named_parts.push(new);
    }

    pub fn add_range(&mut self, first: char, last: char) {
        let range = CharRange { first, last };
        match self.ranges.get(&SortRange(range)) {
            Some(&SortRange(found)) => {
                let first = first.min(found.first);
                let last = last.max(found.last);
                self.remove_range(found);
                self.add_range(first, last);
            }
            None => {
                self.ranges.insert(SortRange(range));
            }
        }
    }

    pub fn add_ranges(&mut self, chars: &str) {
        for c in chars.chars() {
            self.add_range(c, c);
        }
    }

    pub fn remove_range(&mut self, rng: CharRange) {
        self.ranges.remove(&SortRange(rng));
    }
}

#[test]
fn test() {
    let mut c = CharClass::default();
    c.add_range('a', 'd');
    c.add_range('e', 'e');
    c.add_range('g', 'h');
    c.add_range('h', 'j');
    c.add_range('m', 'p');
    c.add_range('t', 'w');
    c.add_range('o', 'u');

    assert_eq!(
        "{'a'-'d', 'e', 'g'-'j', 'm'-'w'}",
        format!("{:?}", c.ranges)
    );
}
