use std::{collections::BTreeSet, mem};

use super::char_range::{CharRange, SortRange};

#[derive(Clone, Eq, Default)]
pub struct CharClassInner<'i> {
    pub(super) groups: Vec<&'i str>,
    pub(super) ranges: BTreeSet<SortRange>,
}

impl<'i> CharClassInner<'i> {
    pub(super) fn get_single_char(&self) -> Option<char> {
        if self.groups.is_empty() && self.ranges.len() == 1 {
            if let Some(range) = self.ranges.iter().next() {
                let range = range.0;
                if range.first == range.last {
                    return Some(range.first);
                }
            }
        }
        None
    }

    pub(super) fn includes_newline(&self) -> bool {
        self.ranges.contains(&SortRange(CharRange {
            first: '\n',
            last: '\n',
        })) || self.groups.iter().any(|&g| matches!(g, "s" | "C" | "Cc"))
    }

    pub(super) fn add_named(&mut self, new: &'i str) {
        self.groups.push(new);
    }

    pub(super) fn add_range(&mut self, first: char, last: char) {
        let range = CharRange { first, last };
        match self.ranges.get(&SortRange(range)) {
            Some(&SortRange(found)) => {
                let first = first.min(found.first);
                let last = last.max(found.last);
                self.remove_range_exact(found);
                self.add_range(first, last);
            }
            None => {
                self.ranges.insert(SortRange(range));
            }
        }
    }

    pub(super) fn add_ranges(&mut self, chars: &str) {
        for c in chars.chars() {
            self.add_range(c, c);
        }
    }

    pub(super) fn remove_range_exact(&mut self, rng: CharRange) {
        self.ranges.remove(&SortRange(rng));
    }

    pub(super) fn union(mut self, cc: &mut CharClassInner<'i>) -> Self {
        for range in &cc.ranges {
            self.add_range(range.0.first, range.0.last);
        }
        self.groups.extend(mem::take(&mut cc.groups).into_iter());
        self
    }
}

impl PartialEq for CharClassInner<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.groups == other.groups
            && self
                .ranges
                .iter()
                .map(|e| e.0)
                .eq(other.ranges.iter().map(|e| e.0))
    }
}
