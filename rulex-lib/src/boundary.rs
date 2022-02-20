#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Start,
    Word,
    NotWord,
    End,
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Start => write!(f, "%-"),
            Self::Word => write!(f, "%"),
            Self::NotWord => write!(f, "%!"),
            Self::End => write!(f, "-%"),
        }
    }
}
