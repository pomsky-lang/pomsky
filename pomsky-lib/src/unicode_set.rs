use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::{Add, AddAssign, RangeInclusive},
};

use pomsky_syntax::exprs::Category;

use crate::{
    exprs::char_class::RegexCharSetItem,
    regex::{RegexProperty, RegexShorthand},
};

#[derive(Debug)]
pub(crate) struct UnicodeSet {
    ranges: BTreeSet<SetRange>,
    props: Vec<RegexCharSetItem>,
}

impl From<char> for UnicodeSet {
    fn from(value: char) -> Self {
        let mut set = UnicodeSet::new();
        set.ranges.insert(SetRange::single(value as u32));
        set
    }
}

#[derive(Debug, Eq, Clone, Copy)]
pub(crate) struct SetRange {
    pub(crate) first: u32,
    pub(crate) last: u32,
}

impl SetRange {
    pub(crate) fn single(char: u32) -> Self {
        SetRange { first: char, last: char }
    }

    pub(crate) fn overlaps_with(&self, other: &SetRange) -> bool {
        !(self.first > other.last || other.first > self.last)
    }

    pub(crate) fn as_chars(self) -> (char, char) {
        (self.first.try_into().unwrap(), self.last.try_into().unwrap())
    }
}

impl PartialEq for SetRange {
    fn eq(&self, other: &SetRange) -> bool {
        self.overlaps_with(other)
    }
}

impl PartialOrd for SetRange {
    fn partial_cmp(&self, other: &SetRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SetRange {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.first > other.last {
            Ordering::Greater
        } else if other.first > self.last {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl Add for SetRange {
    type Output = SetRange;

    fn add(self, rhs: SetRange) -> Self::Output {
        SetRange { first: self.first.min(rhs.first), last: self.last.max(rhs.last) }
    }
}

impl AddAssign for SetRange {
    fn add_assign(&mut self, rhs: SetRange) {
        *self = *self + rhs;
    }
}

impl UnicodeSet {
    pub fn new() -> Self {
        UnicodeSet { ranges: BTreeSet::new(), props: Vec::new() }
    }

    pub fn try_into_char(&self) -> Option<char> {
        if self.ranges.len() == 1 && self.props.is_empty() {
            let range = self.ranges.first().unwrap();
            if range.first == range.last && range.first != b'\r' as u32 {
                return Some(range.first.try_into().unwrap());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.ranges.len() + self.props.len()
    }

    pub fn add_char(&mut self, char: char) {
        self.add(SetRange::single(char as u32));
    }

    pub fn add_range(&mut self, range: RangeInclusive<char>) {
        self.add(SetRange { first: *range.start() as u32, last: *range.end() as u32 })
    }

    pub fn add_range_unchecked(&mut self, range: RangeInclusive<char>) {
        self.ranges.insert(SetRange { first: *range.start() as u32, last: *range.end() as u32 });
    }

    pub fn add_char_unchecked(&mut self, char: char) {
        self.ranges.insert(SetRange::single(char as u32));
    }

    fn add(&mut self, mut range_new: SetRange) {
        let lower = SetRange::single(range_new.first.saturating_sub(1));
        let upper = SetRange::single(range_new.last.saturating_add(1));

        let overlapping = self.ranges.range(lower..=upper).copied().collect::<MaxTwoArray<_>>();
        for &r in overlapping.iter() {
            range_new += r;
            self.ranges.remove(&r);
        }
        self.ranges.insert(range_new);
    }

    pub fn add_prop(&mut self, prop: RegexCharSetItem) {
        if self.props.contains(&prop) {
            return;
        }
        self.props.push(prop);
    }

    pub fn full_props(&self) -> Option<(RegexCharSetItem, RegexCharSetItem)> {
        let mut prev_items = vec![];

        for &(mut item) in &self.props {
            use RegexCharSetItem as RCS;
            use RegexProperty as RP;
            use RegexShorthand as RS;

            if let RCS::Property { negative, value: RP::Category(Category::Separator) } = item {
                item = RCS::Shorthand(if negative { RS::NotSpace } else { RS::Space });
            }

            if let Some(negated) = item.negate() {
                if prev_items.contains(&negated) {
                    return Some((negated, item));
                }
            }

            prev_items.push(item);
        }

        None
    }

    pub fn ranges(&self) -> impl '_ + Iterator<Item = SetRange> {
        self.ranges.iter().copied()
    }

    pub fn props(&self) -> impl '_ + Iterator<Item = RegexCharSetItem> {
        self.props.iter().copied()
    }
}

struct MaxTwoArray<T> {
    items: [Option<T>; 2],
}

impl<T> MaxTwoArray<T> {
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|it| it.as_ref())
    }
}

impl<A> FromIterator<A> for MaxTwoArray<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let mut res = MaxTwoArray { items: [const { None }; 2] };

        if let Some(item) = iter.next() {
            res.items[0] = Some(item);
            if let Some(item) = iter.next() {
                res.items[1] = Some(item);
                if iter.next().is_some() {
                    panic!("Unexpected iterator having more than 2 elements");
                }
            }
        }

        res
    }
}
