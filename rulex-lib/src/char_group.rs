use crate::error::CharClassError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CharGroup<'i> {
    Dot,
    CodePoint,
    X,
    Items(Vec<GroupItem<'i>>),
}

impl<'i> CharGroup<'i> {
    pub fn try_from_range(first: char, last: char) -> Option<Self> {
        if first <= last {
            let range = GroupItem::Range { first, last };
            Some(CharGroup::Items(vec![range]))
        } else {
            None
        }
    }

    pub fn from_chars(chars: &str) -> Self {
        CharGroup::Items(chars.chars().map(GroupItem::Char).collect())
    }

    pub fn from_char(c: char) -> Self {
        CharGroup::Items(vec![GroupItem::Char(c)])
    }

    pub fn try_from_group_name(name: &'i str) -> Result<Self, CharClassError> {
        Ok(match name {
            "codepoint" | "cp" => CharGroup::CodePoint,
            "X" => CharGroup::X,
            "." => CharGroup::Dot,
            _ if name.starts_with(|c: char| c.is_ascii_uppercase()) => {
                CharGroup::Items(vec![GroupItem::Named(name)])
            }
            "n" | "r" | "t" | "w" | "d" | "s" | "h" | "v" => {
                CharGroup::Items(vec![GroupItem::Named(name)])
            }
            "alpha" => CharGroup::Items(vec![
                GroupItem::range_unchecked('a', 'z'),
                GroupItem::range_unchecked('A', 'Z'),
            ]),
            "alnum" => CharGroup::Items(vec![
                GroupItem::range_unchecked('0', '9'),
                GroupItem::range_unchecked('a', 'z'),
                GroupItem::range_unchecked('A', 'Z'),
            ]),
            "ascii" => CharGroup::Items(vec![GroupItem::range_unchecked('\0', '\x7F')]),
            "blank" => CharGroup::Items(vec![GroupItem::Char(' '), GroupItem::Char('\t')]),
            "cntrl" => CharGroup::Items(vec![
                GroupItem::range_unchecked('\0', '\x1F'),
                GroupItem::Char('\x7F'),
            ]),
            "digit" => CharGroup::Items(vec![GroupItem::range_unchecked('0', '9')]),
            "graph" => CharGroup::Items(vec![GroupItem::range_unchecked('!', '~')]),
            "lower" => CharGroup::Items(vec![GroupItem::range_unchecked('a', 'z')]),
            "print" => CharGroup::Items(vec![GroupItem::range_unchecked(' ', '~')]),
            "punct" => CharGroup::Items(vec![
                GroupItem::range_unchecked('!', '/'),
                GroupItem::range_unchecked(':', '@'),
                GroupItem::range_unchecked('[', '`'),
                GroupItem::range_unchecked('{', '~'),
            ]),
            "space" => CharGroup::Items(vec![
                GroupItem::Char(' '),
                GroupItem::Char('\t'),
                GroupItem::Char('\n'),
                GroupItem::Char('\r'),
                GroupItem::Char('\x0B'),
                GroupItem::Char('\x0C'),
            ]),
            "upper" => CharGroup::Items(vec![GroupItem::range_unchecked('A', 'Z')]),
            "word" => CharGroup::Items(vec![
                GroupItem::range_unchecked('0', '9'),
                GroupItem::range_unchecked('a', 'z'),
                GroupItem::range_unchecked('A', 'Z'),
                GroupItem::Char('_'),
            ]),
            "xdigit" => CharGroup::Items(vec![
                GroupItem::range_unchecked('0', '9'),
                GroupItem::range_unchecked('a', 'f'),
                GroupItem::range_unchecked('A', 'F'),
            ]),

            "let" | "not" | "greedy" | "atomic" | "enable" | "disable" => {
                return Err(CharClassError::Keyword(name.to_string()))
            }
            _ => return Err(CharClassError::UnknownNamedClass(name.to_string())),
        })
    }

    pub fn add(&mut self, other: CharGroup<'i>) -> Result<(), CharClassError> {
        match (self, other) {
            (CharGroup::Items(it), CharGroup::Items(other)) => {
                it.extend(other);
                Ok(())
            }
            _ => Err(CharClassError::Unallowed),
        }
    }
}

impl core::fmt::Display for CharGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharGroup::Dot => f.write_str("`.`"),
            CharGroup::CodePoint => f.write_str("`codepoint`"),
            CharGroup::X => f.write_str("`X`"),
            CharGroup::Items(i) => core::fmt::Debug::fmt(i, f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupItem<'i> {
    Char(char),
    Range { first: char, last: char },
    Named(&'i str),
}

impl GroupItem<'_> {
    fn range_unchecked(first: char, last: char) -> Self {
        GroupItem::Range { first, last }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for GroupItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Char(c) => c.fmt(f),
            Self::Range { first, last } => write!(f, "{first:?}-{last:?}"),
            Self::Named(name) => f.write_str(name),
        }
    }
}
