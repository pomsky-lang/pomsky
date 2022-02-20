use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    error::{CompileError, ParseError},
    group::Group,
    repetition::Repetition,
};

#[derive(Clone, PartialEq, Eq)]
pub enum Rulex<'i> {
    Literal(&'i str),
    CharClass(CharClass<'i>),
    Group(Group<'i>),
    Alternation(Alternation<'i>),
    Repetition(Box<Repetition<'i>>),
    Boundary(Boundary),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileOptions {}

impl<'i> Rulex<'i> {
    pub fn parse(input: &'i str, _options: CompileOptions) -> Result<Self, ParseError> {
        crate::parse::parse(input)
    }

    pub fn compile(input: &str, options: CompileOptions) -> Result<String, CompileError> {
        Ok(Rulex::parse(input, options)?.to_string())
    }

    pub fn negate(self) -> Option<Self> {
        todo!()
    }
}

impl ToString for Rulex<'_> {
    fn to_string(&self) -> String {
        todo!()
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rulex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(arg0) => arg0.fmt(f),
            Self::CharClass(arg0) => arg0.fmt(f),
            Self::Group(arg0) => arg0.fmt(f),
            Self::Alternation(arg0) => arg0.fmt(f),
            Self::Repetition(arg0) => arg0.fmt(f),
            Self::Boundary(arg0) => arg0.fmt(f),
        }
    }
}
