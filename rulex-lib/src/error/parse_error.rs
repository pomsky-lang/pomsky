use std::num::{IntErrorKind, ParseIntError};

use crate::{
    parse::{Input, ParseErrorMsg, Token},
    repetition::RepetitionError,
    span::Span,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub(super) kind: ParseErrorKind,
    pub(super) span: Option<Span>,
}

impl ParseErrorKind {
    pub(crate) fn at(self, span: Span) -> ParseError {
        ParseError { kind: self, span: Some(span) }
    }

    pub(crate) fn unknown_index(self) -> ParseError {
        ParseError { kind: self, span: None }
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = self.span {
            write!(f, "{}\n  at {}", self.kind, span)
        } else {
            self.kind.fmt(f)
        }
    }
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

impl<'i, 'b> nom::error::ParseError<Input<'i, 'b>> for ParseError {
    fn from_error_kind(i: Input<'i, 'b>, kind: nom::error::ErrorKind) -> Self {
        ParseErrorKind::Nom(kind).at(i.span())
    }

    fn append(_: Input<'i, 'b>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseErrorKind {
    #[error("Unknown token")]
    UnknownToken,
    #[error(transparent)]
    LexErrorWithMessage(ParseErrorMsg),
    #[error("Unexpected dot. Use `[.]` instead")]
    Dot,

    #[error("Expected {}", .0)]
    Expected(&'static str),
    #[error("There are leftover tokens that couldn't be parsed")]
    LeftoverTokens,
    #[error("Expected {}", .0)]
    ExpectedToken(Token),
    #[error("Expected code point or character")]
    ExpectedCodePointOrChar,
    #[error("Expected on of: {}", ListWithoutBrackets(.0))]
    ExpectedOneOf(Box<[Token]>),
    #[error("This token can't be negated")]
    InvalidNot,
    #[error("The first number in a range must be smaller than the second")]
    RangeIsNotIncreasing,
    #[error(transparent)]
    CharString(CharStringError),
    #[error(transparent)]
    CharClass(CharClassError),
    #[error(transparent)]
    CodePoint(CodePointError),
    #[error(transparent)]
    Number(#[from] NumberError),
    #[error(transparent)]
    Repetition(RepetitionError),

    #[error("Unknown error: {:?}", .0)]
    Nom(nom::error::ErrorKind),
    #[error("Incomplete parse")]
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CharStringError {
    #[error("Strings used in ranges can't be empty")]
    Empty,
    #[error("Strings used in ranges can only contain 1 code point")]
    TooManyCodePoints,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CharClassError {
    #[error("This character class is empty")]
    Empty,
    #[error(
        "Character range must be in increasing order, but it is U+{:04X?} - U+{:04X?}",
        *.0 as u32, *.1 as u32
    )]
    DescendingRange(char, char),
    #[error("Expected string, range, code point or named character class")]
    Invalid,
    #[error("This character class is unknown")]
    Unknown,
    #[error("This combination of character classes is not allowed")]
    Unallowed,
    #[error("Unknown character class `{}`", .0)]
    UnknownNamedClass(String),
    #[error("This character class can't be negated")]
    Negative,
    #[error("A character class can't contain `Grapheme`")]
    Grapheme,
    #[error("Unexpected keyword `{}`", .0)]
    Keyword(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodePointError {
    #[error("This code point is outside the allowed range")]
    Invalid,
    #[error("This code point range is invalid")]
    InvalidRange,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NumberError {
    #[error("cannot parse integer from empty string")]
    Empty,
    #[error("invalid digit found in string")]
    InvalidDigit,
    #[error("number too large")]
    TooLarge,
    #[error("number too small")]
    TooSmall,
    #[error("number would be zero for non-zero type")]
    Zero,
}

impl From<ParseIntError> for NumberError {
    fn from(e: ParseIntError) -> Self {
        match e.kind() {
            IntErrorKind::Empty => NumberError::Empty,
            IntErrorKind::InvalidDigit => NumberError::InvalidDigit,
            IntErrorKind::PosOverflow => NumberError::TooLarge,
            IntErrorKind::NegOverflow => NumberError::TooSmall,
            IntErrorKind::Zero => NumberError::Zero,
            _ => unimplemented!(),
        }
    }
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
