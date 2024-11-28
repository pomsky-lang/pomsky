//! Module containing all the errors that can occur during parsing

use std::{
    fmt,
    num::{IntErrorKind, ParseIntError},
};

use crate::{lexer::Token, Span};

pub use crate::lexer::LexErrorMsg;

/// An error than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {start}..{end}", self.kind)
        } else {
            self.kind.fmt(f)
        }
    }
}

/// An error kind (without a span) than can occur only during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    UnknownToken,
    LexErrorWithMessage(LexErrorMsg),
    KeywordAfterLet(String),
    KeywordAfterColon(String),
    NonAsciiIdentAfterColon(char),
    GroupNameTooLong(usize),
    UnexpectedKeyword(String),

    Deprecated(DeprecationError),

    Expected(&'static str),
    LeftoverTokens,
    ExpectedToken(Token),
    RangeIsNotIncreasing,
    RangeLeadingZeroesVariableLength,
    UnallowedNot,
    UnallowedMultiNot(usize),
    LonePipe,
    LetBindingExists,
    MissingLetKeyword,
    InvalidEscapeInStringAt(usize),
    CharString(CharStringError),
    CharClass(CharClassError),
    InvalidCodePoint,
    Number(NumberError),
    Repetition(RepetitionError),
    MultipleStringsInTestCase,

    RecursionLimit,
}

impl ParseErrorKind {
    /// Creates a [`ParseError`] from this error kind, and a [`Span`] indicating
    /// where the error occurred.
    pub fn at(self, span: Span) -> ParseError {
        ParseError { kind: self, span }
    }
}

impl From<RepetitionError> for ParseErrorKind {
    fn from(e: RepetitionError) -> Self {
        ParseErrorKind::Repetition(e)
    }
}

impl From<CharClassError> for ParseErrorKind {
    fn from(e: CharClassError) -> Self {
        ParseErrorKind::CharClass(e)
    }
}

impl From<DeprecationError> for ParseErrorKind {
    fn from(e: DeprecationError) -> Self {
        ParseErrorKind::Deprecated(e)
    }
}

impl From<NumberError> for ParseErrorKind {
    fn from(e: NumberError) -> Self {
        ParseErrorKind::Number(e)
    }
}

impl std::error::Error for ParseErrorKind {}

impl core::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseErrorKind::UnknownToken => write!(f, "Unknown token"),
            ParseErrorKind::LexErrorWithMessage(msg) => msg.fmt(f),
            ParseErrorKind::KeywordAfterLet(keyword)
            | ParseErrorKind::UnexpectedKeyword(keyword)
            | ParseErrorKind::KeywordAfterColon(keyword) => {
                write!(f, "Unexpected keyword `{keyword}`")
            }
            &ParseErrorKind::NonAsciiIdentAfterColon(char) => {
                let num = char as u32;
                write!(f, "Group name contains illegal code point `{char}` (U+{num:04X}). Group names must be ASCII only.")
            }
            &ParseErrorKind::GroupNameTooLong(len) => {
                write!(f, "Group name is too long. It is {len} code points long, but must be at most 32 code points.")
            }

            ParseErrorKind::Deprecated(deprecation) => deprecation.fmt(f),

            ParseErrorKind::Expected(expected) => write!(f, "Expected {expected}"),
            ParseErrorKind::LeftoverTokens => {
                write!(f, "There are leftover tokens that couldn't be parsed")
            }
            ParseErrorKind::ExpectedToken(token) => write!(f, "Expected {token}"),
            ParseErrorKind::RangeIsNotIncreasing => {
                write!(f, "The first number in a range must be smaller than the second")
            }
            ParseErrorKind::RangeLeadingZeroesVariableLength => write!(
                f,
                "Leading zeroes are not allowed, unless both numbers have the same number of digits"
            ),
            ParseErrorKind::UnallowedNot => write!(f, "This code point or range can't be negated"),
            ParseErrorKind::UnallowedMultiNot(_) => {
                write!(f, "A shorthand character class can't be negated more than once")
            }
            ParseErrorKind::LonePipe => write!(f, "A pipe must be followed by an expression"),
            ParseErrorKind::LetBindingExists => {
                write!(f, "A variable with the same name already exists in this scope")
            }
            ParseErrorKind::MissingLetKeyword => {
                write!(f, "A variable declaration must start with the `let` keyword")
            }
            ParseErrorKind::InvalidEscapeInStringAt(_) => {
                write!(f, "Unsupported escape sequence in string")
            }
            ParseErrorKind::InvalidCodePoint => {
                write!(f, "This code point is outside the allowed range")
            }
            ParseErrorKind::MultipleStringsInTestCase => {
                write!(f, "Test cases can't have multiple strings")
            }
            ParseErrorKind::CharString(error) => error.fmt(f),
            ParseErrorKind::CharClass(error) => error.fmt(f),
            ParseErrorKind::Number(error) => error.fmt(f),
            ParseErrorKind::Repetition(error) => error.fmt(f),

            ParseErrorKind::RecursionLimit => write!(f, "Recursion limit reached"),
        }
    }
}

/// An error that is returned when a deprecated feature is used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeprecationError {}

impl std::error::Error for DeprecationError {}

impl core::fmt::Display for DeprecationError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// An error that relates to a character string
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CharStringError {
    /// Empty string in a code point range within a character class, e.g.
    /// `[''-'z']`
    Empty,
    /// String in a code point range within a character class that contains
    /// multiple code points, e.g. `['abc'-'z']`
    TooManyCodePoints,
}

impl std::error::Error for CharStringError {}

impl core::fmt::Display for CharStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            CharStringError::Empty => "Strings used in ranges can't be empty",
            CharStringError::TooManyCodePoints => {
                "Strings used in ranges can only contain 1 code point"
            }
        };

        f.write_str(error)
    }
}

/// An error that relates to a character class
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CharClassError {
    /// Empty character class, i.e. `[]`
    Empty,
    /// This error is created when `[^` is encountered. This is a negated
    /// character class in a regex, but pomsky instead uses the `![` syntax.
    CaretInGroup,
    /// Non-ascending code point range, e.g. `['z'-'a']`
    NonAscendingRange(char, char),
    /// Invalid token within a character class
    Invalid,
    /// Character class contains incompatible shorthands, e.g. `[. codepoint]`
    Unallowed,
    /// Unknown shorthand character class or Unicode property
    UnknownNamedClass {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },
    /// A character class that can't be negated, e.g. `[!ascii]`
    Negative,
    /// The character class has a prefix where none is expected, e.g. `[scx:w]`
    UnexpectedPrefix,
    /// The character class has the wrong prefix, e.g. `[sc:Basic_Latin]` (the correct prefix would be `block:`)
    WrongPrefix { expected: &'static str, has_in_prefix: bool },
}

impl std::error::Error for CharClassError {}

impl core::fmt::Display for CharClassError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CharClassError::Empty => write!(f, "This character class is empty"),
            CharClassError::CaretInGroup => write!(f, "`^` is not allowed here"),
            &CharClassError::NonAscendingRange(a, b) => write!(
                f,
                "Character range must be in increasing order, but it is U+{:04X?} - U+{:04X?}",
                a as u32, b as u32
            ),
            CharClassError::Invalid => {
                write!(f, "Expected string, range, code point or named character class")
            }
            CharClassError::Unallowed => {
                write!(f, "This combination of character classes is not allowed")
            }
            CharClassError::UnknownNamedClass { found, .. } => {
                write!(f, "Unknown character class `{found}`")
            }
            CharClassError::Negative => write!(f, "This character class can't be negated"),
            CharClassError::UnexpectedPrefix => {
                write!(f, "This character class cannot have a prefix")
            }
            &CharClassError::WrongPrefix { expected, has_in_prefix } => {
                if has_in_prefix {
                    write!(
                        f,
                        "This character class has the wrong prefix; it should be {expected},\n\
                        and the `In` at the start should be removed"
                    )
                } else {
                    write!(f, "This character class has the wrong prefix; it should be {expected}")
                }
            }
        }
    }
}

/// An error that relates to parsing a number
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NumberError {
    /// The parsed string is empty
    Empty,
    /// The parsed string contains a character that isn't a digit
    InvalidDigit,
    /// The number is too large to fit in the target integer type
    TooLarge,
    /// The number is too small to fit in the target integer type
    TooSmall,
    /// The number is zero, but the target number type can't be zero
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

impl std::error::Error for NumberError {}

impl core::fmt::Display for NumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            NumberError::Empty => "cannot parse integer from empty string",
            NumberError::InvalidDigit => "invalid digit found in string",
            NumberError::TooLarge => "number too large",
            NumberError::TooSmall => "number too small",
            NumberError::Zero => "number would be zero for non-zero type",
        };

        f.write_str(error)
    }
}

/// An error indicating an invalid repetition, e.g. `x{4,2}`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepetitionError {
    /// The second number in the repetition is greater than the first
    NotAscending,
    /// Question mark after a repetition, e.g. `x{3}?`
    QmSuffix,
    /// Multiple consecutive repetitions
    Multi,
}

impl std::error::Error for RepetitionError {}

impl core::fmt::Display for RepetitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            RepetitionError::NotAscending => "Lower bound can't be greater than the upper bound",
            RepetitionError::QmSuffix => "Unexpected `?` following a repetition",
            RepetitionError::Multi => "Only one repetition allowed",
        };

        f.write_str(error)
    }
}
