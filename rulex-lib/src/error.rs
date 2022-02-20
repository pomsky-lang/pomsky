use crate::{parse::Token, repetition::RepetitionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    NomErrorKind(nom::error::ErrorKind),
    Incomplete,
    LeftoverTokens(usize),
    ExpectedToken(Token),
    ExpectedWord,
    NumberTooLarge,
    InvalidNot,
    InvalidCodePoint,
    InvalidCodePointRange,
    InvalidCharString,
    RepetitionError(RepetitionError),
}

impl From<nom::Err<ParseError>> for ParseError {
    fn from(e: nom::Err<ParseError>) -> Self {
        match e {
            nom::Err::Incomplete(_) => ParseError::Incomplete,
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        }
    }
}

impl From<RepetitionError> for ParseError {
    fn from(e: RepetitionError) -> Self {
        ParseError::RepetitionError(e)
    }
}

impl<'i, 'b, I> nom::error::ParseError<I> for ParseError {
    fn from_error_kind(_: I, kind: nom::error::ErrorKind) -> Self {
        ParseError::NomErrorKind(kind)
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileError;

impl From<ParseError> for CompileError {
    fn from(_: ParseError) -> Self {
        CompileError
    }
}
