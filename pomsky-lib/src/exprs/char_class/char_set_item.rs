use std::fmt;

use crate::options::RegexFlavor;

use super::{literal, Regex, RegexProperty, RegexShorthand, UnicodeSet};

#[cfg_attr(feature = "dbg", derive(Debug))]
#[derive(Default)]
pub(crate) struct RegexCompoundCharSet {
    pub(crate) negative: bool,
    pub(crate) intersections: Vec<RegexCharSet>,
}

impl RegexCompoundCharSet {
    pub(crate) fn new(set: RegexCharSet) -> Self {
        RegexCompoundCharSet { negative: false, intersections: vec![set] }
    }

    pub(crate) fn negate(mut self) -> RegexCompoundCharSet {
        self.negative = !self.negative;
        self
    }

    pub(crate) fn add(mut self, other: RegexCharSet) -> Option<Regex> {
        if other.negative && self.intersections.iter().all(|i| i.negative) {
            let mut intersections = self.intersections.into_iter();
            let mut char_set = intersections.next().expect("Intersection is empty");
            for next_set in intersections {
                char_set.set.extend(next_set.set);
            }
            char_set.set.extend(other.set);
            if self.negative {
                char_set = char_set.negate();
            }
            Some(Regex::CharSet(char_set))
        } else if self.may_intersect(&other) {
            self.intersections.push(other);
            Some(Regex::CompoundCharSet(self))
        } else {
            None
        }
    }

    fn may_intersect(&self, other: &RegexCharSet) -> bool {
        self.intersections.iter().any(|set| set.may_intersect(other))
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        if self.negative {
            buf.push_str("[^");
        } else {
            buf.push('[');
        }

        let mut is_first = true;
        for intersection in &self.intersections {
            if !is_first {
                buf.push_str("&&");
            }
            intersection.codegen(buf, flavor, true);
            is_first = false;
        }

        buf.push(']');
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
#[derive(Default)]
pub(crate) struct RegexCharSet {
    pub(crate) negative: bool,
    pub(crate) set: UnicodeSet,
}

impl RegexCharSet {
    pub(crate) fn new(items: UnicodeSet) -> Self {
        Self { negative: false, set: items }
    }

    pub(crate) fn negate(mut self) -> Self {
        self.negative = !self.negative;
        self
    }

    pub(crate) fn may_intersect(&self, other: &Self) -> bool {
        self.negative || other.negative || self.set.may_intersect(&other.set)
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor, inside_compound: bool) {
        if self.set.len() == 1 {
            if let Some(range) = self.set.ranges().next() {
                let (first, last) = range.as_chars();
                if first == last && !self.negative {
                    return literal::codegen_char_esc(first, buf, flavor);
                }
            } else if let Some(prop) = self.set.props().next() {
                match prop {
                    RegexCharSetItem::Shorthand(s) => {
                        let shorthand = if self.negative { s.negate() } else { Some(s) };
                        if let Some(shorthand) = shorthand {
                            return shorthand.codegen(buf);
                        }
                    }
                    RegexCharSetItem::Property { negative, value } => {
                        return value.codegen(buf, negative ^ self.negative, flavor);
                    }
                }
            }
        }

        if self.negative {
            buf.push_str("[^");
        } else if !inside_compound {
            buf.push('[');
        }

        let mut is_first = true;
        for prop in self.set.props() {
            match prop {
                RegexCharSetItem::Shorthand(s) => s.codegen(buf),
                RegexCharSetItem::Property { negative, value } => {
                    value.codegen(buf, negative, flavor);
                }
            }
            is_first = false;
        }
        for range in self.set.ranges() {
            let (first, last) = range.as_chars();
            if first == last {
                literal::compile_char_esc_in_class(first, buf, is_first, flavor);
            } else {
                literal::compile_char_esc_in_class(first, buf, is_first, flavor);
                if range.first + 1 < range.last {
                    buf.push('-');
                }
                literal::compile_char_esc_in_class(last, buf, false, flavor);
            }
            is_first = false;
        }

        if self.negative || !inside_compound {
            buf.push(']');
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegexCharSetItem {
    Shorthand(RegexShorthand),
    Property { negative: bool, value: RegexProperty },
}

impl RegexCharSetItem {
    pub(crate) fn negate(self) -> Option<Self> {
        match self {
            RegexCharSetItem::Shorthand(s) => s.negate().map(RegexCharSetItem::Shorthand),
            RegexCharSetItem::Property { negative, value } => {
                Some(RegexCharSetItem::Property { negative: !negative, value })
            }
        }
    }
}

impl fmt::Debug for RegexCharSetItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shorthand(s) => f.write_str(s.as_str()),
            &Self::Property { value, negative } => {
                if negative {
                    f.write_str("!")?;
                }
                f.write_str(value.prefix_as_str())?;
                f.write_str(value.as_str())
            }
        }
    }
}
