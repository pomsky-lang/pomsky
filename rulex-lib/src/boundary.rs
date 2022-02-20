#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Start,
    Word,
    NotWord,
    End,
}
