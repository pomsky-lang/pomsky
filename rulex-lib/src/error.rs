use std::{
    fmt::{Display, Write},
    ops::Range,
};

use crate::{
    options::RegexFlavor,
    parse::{Input, ParseErrorMsg, Token},
    repetition::RepetitionError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Range<usize>,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n  at {:?}", self.kind, self.span)
    }
}

impl ParseError {
    pub fn with_context(self, input: &str) -> impl Display + '_ {
        ContextParseError {
            kind: self.kind,
            span: self.span,
            input,
        }
    }
}

struct ContextParseError<'i> {
    kind: ParseErrorKind,
    span: Range<usize>,
    input: &'i str,
}

impl Display for ContextParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)?;
        f.write_char('\n')?;

        let slice = &self.input[self.span.clone()];
        let before = self.input[..self.span.start]
            .lines()
            .last()
            .unwrap_or_default();
        let after = self.input[self.span.end..]
            .lines()
            .next()
            .unwrap_or_default();

        if let ParseErrorKind::LexErrorWithMessage(msg) = self.kind {
            let messages = match msg {
                ParseErrorMsg::SpecialGroup => get_special_group_help(slice),
                ParseErrorMsg::BackslashSequence => get_backslash_help(slice),
                _ => None,
            };
            if let Some(messages) = messages {
                for msg in messages {
                    writeln!(f, " = help: {msg}")?;
                }
            }
        }

        writeln!(
            f,
            " |\n > {before}\x1B[1m\x1B[38;5;9m{slice}\x1B[0m{after}\n |"
        )
    }
}

fn get_special_group_help(str: &str) -> Option<Vec<String>> {
    assert!(str.starts_with("(?"));
    let str = &str[2..];
    let mut iter = str.chars();

    Some(match (iter.next(), iter.next()) {
        (Some(':'), _) => vec![
            "Non-capturing groups are just parentheses: `(...)`".into(),
            "Capturing groups use the `:(...)` syntax.".into(),
        ],
        (Some('P'), Some('<')) => {
            let str = &str[2..];
            let rest = str.trim_start_matches(char::is_alphanumeric);
            let name = &str[..str.len() - rest.len()];
            vec![
                "Named capturing groups use the `:name(...)` syntax.".into(),
                format!("Try `:{name}(...)` instead"),
            ]
        }
        (Some('>'), _) => vec!["Atomic capturing groups are not supported".into()],
        (Some('|'), _) => vec!["Branch reset groups are not supported".into()],
        (Some('('), _) => vec!["Branch reset groups are not supported".into()],
        (Some('='), _) => vec![
            "Lookahead uses the `>>` syntax.".into(),
            "For example, `>> 'bob'` matches if the position is followed by bob.".into(),
        ],
        (Some('!'), _) => vec![
            "Negative lookahead uses the `!>>` syntax.".into(),
            "For example, `!>> 'bob'` matches if the position is not followed by bob.".into(),
        ],
        (Some('<'), Some('=')) => vec![
            "Lookbehind uses the `<<` syntax.".into(),
            "For example, `<< 'bob'` matches if the position is preceded with bob.".into(),
        ],
        (Some('<'), Some('!')) => vec![
            "Negative lookbehind uses the `!<<` syntax.".into(),
            "For example, `!<< 'bob'` matches if the position is not preceded with bob.".into(),
        ],
        _ => return None,
    })
}

fn get_backslash_help(str: &str) -> Option<Vec<String>> {
    assert!(str.starts_with('\\'));
    let str = &str[1..];
    let mut iter = str.chars();

    Some(match iter.next() {
        Some('b') => vec!["Replace `\\b` with `%` to match a word boundary".into()],
        Some('B') => {
            vec!["Replace `\\B` with `!%` to match a place without a word boundary".into()]
        }
        Some('A') => vec!["Replace `\\A` with `<%` to match the start of the string".into()],
        Some('z') => vec!["Replace `\\z` with `%>` to match the end of the string".into()],
        Some('Z') => vec![
            "\\Z is not supported. Use `%>` to match the end of the string".into(),
            "Note, however, that `%>` doesn't match the position before the final newline.".into(),
        ],
        Some('N') => vec![format!("Replace `\\N` with `[.]`")],
        Some(c @ ('u' | 'x')) => {
            let (str, max_len) = if let Some('{') = iter.next() {
                (&str[2..], 6)
            } else if c == 'u' {
                (&str[1..], 4)
            } else {
                (&str[1..], 2)
            };
            let len = str
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .take(max_len)
                .count();
            let hex = &str[..len];

            vec![
                "Unicode characters are written like `U+0` or `U+FFFFF`.".into(),
                format!("Try `U+{hex}` instead"),
            ]
        }
        Some(
            c @ ('a' | 'e' | 'f' | 'n' | 'r' | 'h' | 'v' | 'X' | 'd' | 'D' | 'w' | 'W' | 's' | 'S'
            | 'R'),
        ) => vec![format!("Replace `\\{c}` with `[{c}]`")],
        Some('k') if iter.next() == Some('<') => {
            let str = &str[2..];
            let rest = str.trim_start_matches(char::is_alphanumeric);
            let name = &str[..str.len() - rest.len()];
            vec![
                "Backreferences are written like `::name`.".into(),
                format!("Replace `\\k<{name}>` with `::{name}`"),
            ]
        }
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("Unknown token")]
    LexError,
    #[error(transparent)]
    LexErrorWithMessage(ParseErrorMsg),

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
    CharClass(CharClassError),
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
    pub(crate) fn at(self, span: Range<usize>) -> ParseError {
        ParseError { kind: self, span }
    }

    pub(crate) fn unknown_index(self) -> ParseError {
        ParseError {
            kind: self,
            span: usize::MAX..usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CharStringError {
    #[error("Strings used in ranges can't be empty")]
    Empty,
    #[error("Strings used in ranges can only contain 1 code point")]
    TooManyCodePoints,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
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
    #[error("Unexpected keyword `{}`", .0)]
    Keyword(String),
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

impl<'i, 'b> nom::error::ParseError<Input<'i, 'b>> for ParseError {
    fn from_error_kind(i: Input<'i, 'b>, kind: nom::error::ErrorKind) -> Self {
        ParseErrorKind::Nom(kind).at(i.index())
    }

    fn append(_: Input<'i, 'b>, _: nom::error::ErrorKind, other: Self) -> Self {
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

    #[error("Compile error: This character class is empty")]
    EmptyClass,

    #[error("Compile error: This negated character class is empty")]
    EmptyClassNegated,

    #[error("Compile error: {}", .0)]
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Feature {
    #[error("named capture groups")]
    NamedCaptureGroups,
    #[error("lookahead/behind")]
    Lookaround,
    #[error("grapheme cluster matcher (\\X)")]
    Grapheme,
    #[error("Unicode line break (\\R)")]
    UnicodeLineBreak,
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        CompileError::ParseError(e)
    }
}
