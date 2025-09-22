//! Contains the [`CharGroup`] type, which is the contents of a
//! [`CharClass`](crate::char_class::CharClass).
//!
//! However, a `CharGroup` doesn't store the information whether the character
//! class is negated.
//!
//! Refer to the [`char_class` module](crate::char_class) for more information.

use crate::{Span, error::ParseErrorKind};

use super::unicode::{Category, CodeBlock, OtherProperties, Script};

/// The contents of a [`CharClass`](crate::char_class::CharClass).
///
/// Refer to the [`char_class` module](crate::char_class) for more information.
#[derive(Clone, PartialEq, Eq)]
pub struct CharGroup {
    /// This variant is used for the remaining cases.
    pub items: Vec<GroupItem>,
}

impl CharGroup {
    /// Tries to create a `CharGroup` from a range of characters (inclusive).
    /// Returns `None` if `last` is lower than `first`.
    pub(crate) fn try_from_range(first: char, last: char) -> Option<Vec<GroupItem>> {
        if first < last { Some(vec![GroupItem::Range { first, last }]) } else { None }
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
        kind: Option<&str>,
        name: &str,
        negative: bool,
        span: Span,
    ) -> Result<Vec<GroupItem>, ParseErrorKind> {
        Ok(match name {
            _ if name == "ascii" || name.starts_with("ascii_") => {
                super::ascii::parse_ascii_group(name, negative)?
            }
            _ => {
                let name = super::unicode::parse_group_name(kind, name)?;
                vec![GroupItem::Named { name, negative, span }]
            }
        })
    }
}

/// One item in a character class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Named { name: GroupName, negative: bool, span: Span },
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
            Self::Named { name, negative, .. } => {
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
                    GroupName::Script(s, e) => {
                        buf.push_str(e.as_str());
                        buf.push_str(s.as_str());
                        return;
                    }
                    GroupName::CodeBlock(b) => b.as_str(),
                    GroupName::OtherProperties(b) => b.as_str(),
                };
                buf.push_str(name);
            }
        }
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for GroupItem {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0u8..=2)? {
            0 => GroupItem::Char(u.arbitrary()?),
            1 => {
                let first = u.arbitrary()?;
                let last = u.arbitrary()?;
                if first >= last {
                    return Err(arbitrary::Error::IncorrectFormat);
                }
                GroupItem::Range { first, last }
            }
            _ => GroupItem::Named {
                name: GroupName::arbitrary(u)?,
                negative: bool::arbitrary(u)?,
                span: Span::arbitrary(u)?,
            },
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(
            u8::size_hint(depth),
            arbitrary::size_hint::or_all(&[
                char::size_hint(depth),
                arbitrary::size_hint::and(char::size_hint(depth), char::size_hint(depth)),
                arbitrary::size_hint::and(GroupName::size_hint(depth), bool::size_hint(depth)),
            ]),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum GroupName {
    Word,
    Digit,
    Space,
    HorizSpace,
    VertSpace,
    Category(Category),
    Script(Script, ScriptExtension),
    CodeBlock(CodeBlock),
    OtherProperties(OtherProperties),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum ScriptExtension {
    Yes,
    No,
    Unspecified,
}

impl GroupName {
    pub fn kind(self) -> &'static str {
        match self {
            GroupName::Word
            | GroupName::Digit
            | GroupName::Space
            | GroupName::HorizSpace
            | GroupName::VertSpace => "shorthand",
            GroupName::Category(_) => "category",
            GroupName::Script(..) => "script",
            GroupName::CodeBlock(_) => "block",
            GroupName::OtherProperties(_) => "property",
        }
    }
}

impl ScriptExtension {
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptExtension::Yes => "scx:",
            ScriptExtension::No => "sc:",
            ScriptExtension::Unspecified => "",
        }
    }
}
