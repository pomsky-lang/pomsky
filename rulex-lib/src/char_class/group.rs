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

    pub fn from_group_name(name: &'i str) -> Self {
        match name {
            "codepoint" | "cp" => CharGroup::CodePoint,
            "X" => CharGroup::X,
            "." => CharGroup::Dot,
            _ => CharGroup::Items(vec![GroupItem::Named(name)]),
        }
    }

    pub fn add(&mut self, other: CharGroup<'i>) -> Result<(), CharClassError> {
        match (self, other) {
            (CharGroup::Items(it), CharGroup::Items(other)) => {
                it.extend(other);
                Ok(())
            }
            (a, b) => {
                eprintln!("{a:?}, {b:?}");
                Err(CharClassError::Unallowed)
            }
        }
    }

    #[cfg(test)]
    pub fn union(mut self, other: Self) -> Result<Self, CharClassError> {
        self.add(other)?;
        Ok(self)
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
