use crate::{
    options::RegexFlavor,
    parse::{Token, Tokens},
    repetition::RepetitionError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    index: usize,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n  at byte {}", self.kind, self.index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("Unknown token")]
    LexError,

    #[error("Expected {}", .0)]
    Expected(&'static str),
    #[error("There are leftover tokens that couldn't be parsed")]
    LeftoverTokens,
    #[error("Expected token {}", .0)]
    ExpectedToken(Token),
    #[error("Expected code point or character")]
    ExpectedCodePointOrChar,
    #[error("Expected on of: {}", ListWithoutBrackets(.0))]
    ExpectedOneOf(Box<[Token]>),
    #[error("This token can't be negated")]
    InvalidNot,
    #[error(transparent)]
    CharString(CharStringError),
    #[error(transparent)]
    CodePoint(CodePointError),
    #[error(transparent)]
    Number(NumberError),
    #[error(transparent)]
    Repetition(RepetitionError),

    #[error("Unknown error: {:?}", .0)]
    Nom(nom::error::ErrorKind),
    #[error("Incomplete parse")]
    Incomplete,
}

struct ListWithoutBrackets<'a, T>(&'a [T]);

impl<T: core::fmt::Display> core::fmt::Display for ListWithoutBrackets<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl ParseErrorKind {
    pub(crate) fn at(self, index: usize) -> ParseError {
        ParseError { kind: self, index }
    }

    pub(crate) fn unknown_index(self) -> ParseError {
        ParseError {
            kind: self,
            index: usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CharStringError {
    #[error("This char string is invalid")]
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CodePointError {
    #[error("This code point is outside the allowed range")]
    Invalid,
    #[error("This code point range is invalid")]
    InvalidRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NumberError {
    #[error("Numbers this large are not supported")]
    TooLarge,
}

impl From<nom::Err<ParseError>> for ParseError {
    fn from(e: nom::Err<ParseError>) -> Self {
        match e {
            nom::Err::Incomplete(_) => ParseErrorKind::Incomplete.unknown_index(),
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        }
    }
}

impl From<RepetitionError> for ParseErrorKind {
    fn from(e: RepetitionError) -> Self {
        ParseErrorKind::Repetition(e)
    }
}

impl<'i, 'b> nom::error::ParseError<Tokens<'i, 'b>> for ParseError {
    fn from_error_kind(i: Tokens<'i, 'b>, kind: nom::error::ErrorKind) -> Self {
        ParseErrorKind::Nom(kind).at(i.index())
    }

    fn append(_: Tokens<'i, 'b>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompileError {
    #[error("Parse error: {}", .0)]
    ParseError(ParseError),

    #[error("Compile error: Unsupported feature `{}` in the `{:?}` regex flavor", .0, .1)]
    Unsupported(Feature, RegexFlavor),

    #[error("Compile error: Group name `{}` used multiple times", .0)]
    NameUsedMultipleTimes(String),

    #[error("Compile error: {}", .0)]
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Feature {
    #[error("named capture groups")]
    NamedCaptureGroups,
    #[error("lookahead/behind")]
    Lookaround,
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        CompileError::ParseError(e)
    }
}
