//! Contains the [`CharGroup`] type, which is the contents of a
//! [`CharClass`](crate::char_class::CharClass).
//!
//! However, a `CharGroup` doesn't store the information whether the character
//! class is negated.
//!
//! Refer to the [`char_class` module](crate::char_class) for more information.

use std::fmt;

use crate::{
    error::{CharClassError, DeprecationError, ParseErrorKind},
    warning::DeprecationWarning,
};

use super::unicode::{Category, CodeBlock, OtherProperties, Script};

/// The contents of a [`CharClass`](crate::char_class::CharClass).
///
/// Refer to the [`char_class` module](crate::char_class) for more information.
#[derive(Clone, PartialEq, Eq)]
pub enum CharGroup {
    /// `[.]`, the [dot](https://www.regular-expressions.info/dot.html). Matches any code point
    /// except `\n`.
    Dot,
    /// This variant is used for the remaining cases.
    Items(Vec<GroupItem>),
}

impl CharGroup {
    /// Tries to create a `CharGroup` from a range of characters (inclusive).
    /// Returns `None` if `last` is lower than `first`.
    pub(crate) fn try_from_range(first: char, last: char) -> Option<Self> {
        if first <= last {
            Some(CharGroup::Items(vec![GroupItem::Range { first, last }]))
        } else {
            None
        }
    }

    /// Creates a `CharGroup` from a string, by iterating over the `char`s and
    /// adding each of them to the list.
    pub(crate) fn from_chars(chars: &str) -> Self {
        CharGroup::Items(chars.chars().map(GroupItem::Char).collect())
    }

    /// Creates a `CharGroup` from a single `char`.
    pub(crate) fn from_char(c: char) -> Self {
        CharGroup::Items(vec![GroupItem::Char(c)])
    }

    /// Try to create a `CharGroup` from the name of a character class. Fails if
    /// the name is lowercase and not known, or if it matches a keyword.
    ///
    /// POSIX classes (e.g. `alnum` or `blank`) are converted to ranges (e.g.
    /// `[0-9a-zA-Z]`). This is relatively simple and maximizes
    /// compatibility.
    ///
    /// If the name is uppercase (and not `R`), we just assume that it is a
    /// Unicode category, script or block. This needs to be fixed at one
    /// point!
    pub(crate) fn try_from_group_name(
        name: &str,
        negative: bool,
    ) -> Result<(Self, Option<DeprecationWarning>), ParseErrorKind> {
        Ok(match name {
            _ if name == "ascii" || name.starts_with("ascii_") => {
                (CharGroup::Items(super::ascii::parse_ascii_group(name, negative)?), None)
            }

            "codepoint" | "cp" | "." if negative => {
                return Err(CharClassError::Negative.into());
            }

            "codepoint" => return Err(DeprecationError::CodepointInSet.into()),
            "cp" => return Err(DeprecationError::CpInSet.into()),
            "." => (CharGroup::Dot, Some(DeprecationWarning::Dot)),

            _ => (
                CharGroup::Items(vec![GroupItem::Named {
                    name: super::unicode::parse_group_name(name)?,
                    negative,
                }]),
                None,
            ),
        })
    }

    /// Tries to add another `CharGroup` to this one. Fails if one of them is a
    /// `[.]` or `[cp]`. If it succeeds, it just appends the new items to
    /// the existing ones.
    ///
    /// The previous implementation was much more advanced and merged
    /// overlapping ranges using a `BTreeSet` with a custom (technically
    /// incorrect) `PartialEq` implementation. This added
    /// a lot of complexity for very little return, so I decided to ditch it.
    pub(crate) fn add(&mut self, other: CharGroup) -> Result<(), CharClassError> {
        match (self, other) {
            (CharGroup::Items(it), CharGroup::Items(other)) => {
                it.extend(other);
                Ok(())
            }
            _ => Err(CharClassError::Unallowed),
        }
    }
}

/// One item in a character class.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupItem {
    /// A Unicode code point. It can be denoted in quotes (e.g. `'a'`) or in
    /// hexadecimal notation (`U+201`).
    ///
    /// Some non-printable ASCII characters are also parsed to a
    /// [`GroupItem::Char`]: `[n]`, `[t]`, `[r]`, `[a]`, `[e]` and `[f]`.
    Char(char),
    /// A range of Unicode code points. It is denoted as `A-B`, where `A` and
    /// `B` are Unicode code points, allowing the same notation as for
    /// [`GroupItem::Char`]. Both `A` and `B` are included in the range.
    Range { first: char, last: char },
    /// A named character class, i.e. a shorthand or a Unicode
    /// category/script/block. Shorthands are `[w]`, `[s]`, `[d]`, `[v]`,
    /// `[h]` and `[R]`.
    ///
    /// Some of them (`w`, `d`, `s` and Unicode) can be negated.
    Named { name: GroupName, negative: bool },
}

impl GroupItem {
    pub(crate) fn range_unchecked(first: char, last: char) -> Self {
        GroupItem::Range { first, last }
    }

    #[cfg(feature = "dbg")]
    pub(crate) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        fn print_char(c: char, buf: &mut crate::PrettyPrinter) {
            match c {
                '\n' => buf.push('n'),
                '\r' => buf.push('r'),
                '\t' => buf.push('t'),
                '\u{07}' => buf.push('a'),
                '\u{1b}' => buf.push('e'),
                '\u{0c}' => buf.push('f'),
                _ => buf.pretty_print_char(c),
            }
        }

        match *self {
            Self::Char(c) => print_char(c, buf),
            Self::Range { first, last } => {
                print_char(first, buf);
                buf.push('-');
                print_char(last, buf);
            }
            Self::Named { name, negative } => {
                if negative {
                    buf.push('!');
                }
                let name = match name {
                    GroupName::Word => "word",
                    GroupName::Digit => "digit",
                    GroupName::Space => "space",
                    GroupName::HorizSpace => "horiz_space",
                    GroupName::VertSpace => "vert_space",
                    GroupName::Category(c) => c.as_str(),
                    GroupName::Script(s) => s.as_str(),
                    GroupName::CodeBlock(b) => b.as_str(),
                    GroupName::OtherProperties(b) => b.as_str(),
                };
                buf.push_str(name);
            }
        }
    }
}

impl fmt::Debug for GroupItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(c) => c.fmt(f),
            Self::Range { first, last } => write!(f, "{first:?}-{last:?}"),
            &Self::Named { name, negative } => {
                if negative {
                    f.write_str("!")?;
                }
                f.write_str(name.as_str())
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum GroupName {
    Word,
    Digit,
    Space,
    HorizSpace,
    VertSpace,
    Category(Category),
    Script(Script),
    CodeBlock(CodeBlock),
    OtherProperties(OtherProperties),
}

impl GroupName {
    pub fn as_str(self) -> &'static str {
        match self {
            GroupName::Word => "word",
            GroupName::Digit => "digit",
            GroupName::Space => "space",
            GroupName::HorizSpace => "horiz_space",
            GroupName::VertSpace => "vert_space",
            GroupName::Category(c) => c.as_str(),
            GroupName::Script(s) => s.as_str(),
            GroupName::CodeBlock(b) => b.as_str(),
            GroupName::OtherProperties(o) => o.as_str(),
        }
    }
}
