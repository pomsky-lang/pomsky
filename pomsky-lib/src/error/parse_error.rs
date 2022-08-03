use std::num::{IntErrorKind, ParseIntError};

use crate::{
    parse::{Input, LexErrorMsg, Token},
    span::Span,
};

use super::Diagnostic;

/// An error than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub(crate) kind: ParseErrorKind,
    pub(crate) span: Span,
}

impl ParseError {
    /// Create a [Diagnostic] from this error.
    pub fn diagnostic(self, source_code: &str) -> Diagnostic {
        Diagnostic::from_parse_error(self, source_code)
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {}..{}", self.kind, start, end)
        } else {
            self.kind.fmt(f)
        }
    }
}

impl From<nom::Err<ParseError>> for ParseError {
    fn from(e: nom::Err<ParseError>) -> Self {
        match e {
            nom::Err::Incomplete(_) => ParseErrorKind::Incomplete.at(Span::empty()),
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        }
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

/// An error kind (without a span) than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ParseErrorKind {
    #[error("Multiple parsing errors encountered")]
    Multiple(Box<[ParseError]>),

    #[error("Unknown token")]
    UnknownToken,
    #[error(transparent)]
    LexErrorWithMessage(LexErrorMsg),
    #[error("Unexpected dot")]
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
    #[error("Expected code point or character")]
    ExpectedCodePointOrChar,
    #[error("The first number in a range must be smaller than the second")]
    RangeIsNotIncreasing,
    #[error("This expression can't be negated")]
    UnallowedNot,
    #[error("An expression can't be negated twice")]
    UnallowedDoubleNot,
    #[error("Range is too big, it isn't allowed to contain more than {} digits", .0)]
    RangeIsTooBig(u8),
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
    #[error(transparent)]
    Unsupported(UnsupportedError),

    #[error("Recursion limit reached")]
    RecursionLimit,

    #[error("Unknown error: {:?}", .0)]
    Nom(nom::error::ErrorKind),
    #[error("Incomplete parse")]
    Incomplete,
}

impl ParseErrorKind {
    pub(crate) fn at(self, span: Span) -> ParseError {
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
pub(crate) enum CharStringError {
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
pub(crate) enum CharClassError {
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
pub(crate) enum CodePointError {
    /// Code point that is outside the allowed range, e.g. `U+200000`
    #[error("This code point is outside the allowed range")]
    Invalid,
}

/// An error that relates to parsing a number
#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum NumberError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum RepetitionError {
    #[error("Lower bound can't be greater than the upper bound")]
    NotAscending,
    #[error("Unexpected `?` following a repetition")]
    QuestionMarkAfterRepetition,
    #[error("Unexpected `+` following a repetition")]
    PlusAfterRepetition,
}

/// An error that indicates that an unsupported feature was used.
///
/// See [`crate::features::PomskyFeatures`] for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum UnsupportedError {
    #[error("Grapheme is not supported")]
    Grapheme,

    #[error("Numbered capturing groups is not supported")]
    NumberedGroups,

    #[error("Named capturing groups is not supported")]
    NamedGroups,

    #[error("References aren't supported")]
    References,

    #[error("Lazy mode isn't supported")]
    LazyMode,

    #[error("Ranges aren't supported")]
    Ranges,

    #[error("Variables aren't supported")]
    Variables,

    #[error("Lookahead isn't supported")]
    Lookahead,

    #[error("Lookbehind isn't supported")]
    Lookbehind,

    #[error("Word boundaries aren't supported")]
    Boundaries,
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
