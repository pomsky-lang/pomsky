use nom::Parser;

use crate::{
    error::{ParseError, ParseErrorKind},
    span::Span,
};

use super::input::Input;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum Token {
    /// `<%` (`^` boundary)
    BStart,
    /// `%>` (`$` boundary)
    BEnd,
    /// `%` (`\b` boundary)
    BWord,

    /// `*` (`*?` repetition)
    Star,
    /// `+` (`+?` repetition)
    Plus,
    /// `?` (`??` repetition)
    QuestionMark,

    /// `|` (or)
    Pipe,

    /// `:` (capturing group start)
    Colon,
    /// `(` (open group)
    OpenParen,
    /// `)` (close group)
    CloseParen,

    /// `{` (open repetition)
    OpenBrace,
    /// `}` (close repetition)
    CloseBrace,
    /// `,` (comma in repetition)
    Comma,

    Not,

    /// `[` (open character class)
    OpenBracket,

    /// `-` (unicode range)
    Dash,

    /// `]` (close character class)
    CloseBracket,

    /// `.` (any code point except newline)
    Dot,

    /// `>>` (positive lookahead)
    LookAhead,

    /// `<<` (positive lookbehind)
    LookBehind,

    /// `::` (back reference)
    Backref,

    /// `;` (delimits modifiers)
    Semicolon,

    /// `=` (for assignments)
    Equals,

    /// `"Hello"` or `'Hello'` (`Hello`)
    String,

    /// `U+FFF03` (Unicode code point)
    CodePoint,

    /// `12` (number in repetition)
    Number,

    /// `hello` (capturing group name)
    Identifier,

    // match illegal tokens for which we want to show a better error message
    ErrorMsg(ParseErrorMsg),

    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseErrorMsg {
    #[error("`^` is not a valid token")]
    Caret,
    #[error("`$` is not a valid token")]
    Dollar,
    #[error("This syntax is not supported")]
    SpecialGroup,
    #[error("Backslash escapes are not supported")]
    Backslash,
    #[error("Backslash escapes are not supported")]
    BackslashU4,
    #[error("Backslash escapes are not supported")]
    BackslashX2,
    #[error("Backslash escapes are not supported")]
    BackslashUnicode,
    #[error("Backslash escapes are not supported")]
    BackslashK,
    #[error("This string literal doesn't have a closing quote")]
    UnclosedString,
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Token::BStart => "`<%`",
            Token::BEnd => "`%>`",
            Token::BWord => "`%`",
            Token::Star => "`*`",
            Token::Plus => "`+`",
            Token::QuestionMark => "`?`",
            Token::Pipe => "`|`",
            Token::Colon => "`:`",
            Token::OpenParen => "`(`",
            Token::CloseParen => "`)`",
            Token::OpenBrace => "`{`",
            Token::CloseBrace => "`}`",
            Token::Comma => "`,`",
            Token::LookAhead => "`>>`",
            Token::LookBehind => "`<<`",
            Token::Backref => "`::`",
            Token::Not => "`!`",
            Token::OpenBracket => "`[`",
            Token::Dash => "`-`",
            Token::CloseBracket => "`]`",
            Token::Dot => "`.`",
            Token::Semicolon => "`;`",
            Token::Equals => "`=`",
            Token::String => "string",
            Token::CodePoint => "code point",
            Token::Number => "number",
            Token::Identifier => "identifier",
            Token::ErrorMsg(_) | Token::Error => "error",
        })
    }
}

impl<'i, 'b> Parser<Input<'i, 'b>, (&'i str, Span), ParseError> for Token {
    fn parse(
        &mut self,
        mut input: Input<'i, 'b>,
    ) -> nom::IResult<Input<'i, 'b>, (&'i str, Span), ParseError> {
        match input.peek() {
            Some((t, s)) if t == *self => {
                let span = input.span();
                let _ = input.next();
                Ok((input, (s, span)))
            }
            _ => Err(nom::Err::Error(ParseErrorKind::ExpectedToken(*self).at(input.span()))),
        }
    }
}

impl<'i, 'b> Parser<Input<'i, 'b>, (Token, Span), ParseError> for &'i str {
    fn parse(
        &mut self,
        mut input: Input<'i, 'b>,
    ) -> nom::IResult<Input<'i, 'b>, (Token, Span), ParseError> {
        match input.peek() {
            Some((t, s)) if s == *self => {
                let span = input.span();
                let _ = input.next();
                Ok((input, (t, span)))
            }
            _ => Err(nom::Err::Error(ParseErrorKind::Expected("word").at(input.span()))),
        }
    }
}
