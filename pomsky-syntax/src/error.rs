//! Module containing all the errors that can occur during parsing

use std::{
    fmt,
    num::{IntErrorKind, ParseIntError},
};

use crate::{
    parse::{LexErrorMsg, Token},
    span::Span,
};

/// An error than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {}..{}", self.kind, start, end)
        } else {
            self.kind.fmt(f)
        }
    }
}

/// An error kind (without a span) than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseErrorKind {
    #[error("Multiple parsing errors encountered")]
    Multiple(Box<[ParseError]>),

    #[error("Unknown token")]
    UnknownToken,
    #[error(transparent)]
    LexErrorWithMessage(LexErrorMsg),
    #[error("The dot is not supported")] // this is for a dot *not* enclosed in brackets
    Dot,
    #[error("Unexpected keyword `{}`", .0)]
    KeywordAfterLet(String),
    #[error("Unexpected keyword `{}`", .0)]
    UnexpectedKeyword(String),

    #[error("Expected {}", .0)]
    Expected(&'static str),
    #[error("There are leftover tokens that couldn't be parsed")]
    LeftoverTokens,
    #[error("Expected {}", .0)]
    ExpectedToken(Token),
    // TODO: Check if this is needed
    #[error("Expected code point or character")]
    ExpectedCodePointOrChar,
    #[error("The first number in a range must be smaller than the second")]
    RangeIsNotIncreasing,
    #[error("This expression can't be negated")]
    UnallowedNot,
    #[error("An expression can't be negated more than once")]
    UnallowedMultiNot(usize),
    #[error("A pipe must be followed by an expression")]
    LonePipe,
    #[error("A variable with the same name already exists in this scope")]
    LetBindingExists,
    #[error("Unsupported escape sequence in string")]
    InvalidEscapeInStringAt(usize),
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

    #[error("Recursion limit reached")]
    RecursionLimit,
}

impl ParseErrorKind {
    pub fn at(self, span: Span) -> ParseError {
        ParseError { kind: self, span }
    }
}

impl From<RepetitionError> for ParseErrorKind {
    fn from(e: RepetitionError) -> Self {
        ParseErrorKind::Repetition(e)
    }
}

/// An error that relates to a character string
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CharStringError {
    /// Empty string in a code point range within a character class, e.g.
    /// `[''-'z']`
    #[error("Strings used in ranges can't be empty")]
    Empty,

    /// String in a code point range within a character class that contains
    /// multiple code points, e.g. `['abc'-'z']`
    #[error("Strings used in ranges can only contain 1 code point")]
    TooManyCodePoints,
}

/// An error that relates to a character class
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CharClassError {
    /// Empty character class, i.e. `[]`
    #[error("This character class is empty")]
    Empty,

    #[error("`^` is not a valid token")]
    CaretInGroup,

    /// Descending code point range, e.g. `['z'-'a']`
    #[error(
        "Character range must be in increasing order, but it is U+{:04X?} - U+{:04X?}",
        *.0 as u32, *.1 as u32
    )]
    DescendingRange(char, char),

    /// Invalid token within a character class
    #[error("Expected string, range, code point or named character class")]
    Invalid,

    /// Character class contains incompatible shorthands, e.g. `[. codepoint]`
    #[error("This combination of character classes is not allowed")]
    Unallowed,

    /// Unknown shorthand character class or Unicode property
    #[error("Unknown character class `{}`", .found)]
    UnknownNamedClass {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },

    /// A character class that can't be negated, e.g. `[!ascii]`
    #[error("This character class can't be negated")]
    Negative,

    /// Unexpected keyword within a character class, e.g. `[let]`
    #[error("Unexpected keyword `{}`", .0)]
    Keyword(String),
}

/// An error that relates to a Unicode code point
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodePointError {
    /// Code point that is outside the allowed range, e.g. `U+200000`
    #[error("This code point is outside the allowed range")]
    Invalid,
}

/// An error that relates to parsing a number
#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NumberError {
    /// The parsed string is empty
    #[error("cannot parse integer from empty string")]
    Empty,

    /// The parsed string contains a character that isn't a digit
    #[error("invalid digit found in string")]
    InvalidDigit,

    /// The number is too large to fit in the target integer type
    #[error("number too large")]
    TooLarge,

    /// The number is too small to fit in the target integer type
    #[error("number too small")]
    TooSmall,

    /// The number is zero, but the target number type can't be zero
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

/// An error indicating an invalid repetition, e.g. `x{4,2}`
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RepetitionError {
    /// The second number in the repetition is greater than the first
    #[error("Lower bound can't be greater than the upper bound")]
    NotAscending,

    /// Question mark after a repetition, e.g. `x{3}?`
    #[error("Unexpected `?` following a repetition")]
    QmSuffix,

    /// Plus after a repetition, e.g. `x{3}+`
    #[error("Unexpected `+` following a repetition")]
    PlusSuffix,
}
