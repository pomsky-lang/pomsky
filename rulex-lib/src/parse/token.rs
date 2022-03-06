use logos::Logos;
use nom::Parser;

use crate::{
    error::{ParseError, ParseErrorKind},
    span::Span,
};

use super::input::Input;

#[derive(Debug, Logos, Eq, PartialEq, Copy, Clone)]
pub enum Token {
    /// `<%` (`^` boundary)
    #[token("<%")]
    BStart,
    /// `%>` (`$` boundary)
    #[token("%>")]
    BEnd,
    /// `%` (`\b` boundary)
    #[token("%")]
    BWord,

    /// `*` (`*?` repetition)
    #[token("*")]
    Star,
    /// `+` (`+?` repetition)
    #[token("+")]
    Plus,
    /// `?` (`??` repetition)
    #[token("?")]
    QuestionMark,

    /// `|` (or)
    #[token("|")]
    Pipe,

    /// `:` (capturing group start)
    #[token(":")]
    Colon,
    /// `(` (open group)
    #[token("(")]
    OpenParen,
    /// `)` (close group)
    #[token(")")]
    CloseParen,

    /// `{` (open repetition)
    #[token("{")]
    OpenBrace,
    /// `}` (close repetition)
    #[token("}")]
    CloseBrace,
    /// `,` (comma in repetition)
    #[token(",")]
    Comma,

    #[token("!")]
    Not,

    /// `[` (open character class)
    #[token("[")]
    OpenBracket,

    /// `-` (unicode range)
    #[token("-")]
    Dash,

    /// `]` (close character class)
    #[token("]")]
    CloseBracket,

    /// `.` (any code point except newline)
    #[token(".")]
    Dot,

    /// `>>` (positive lookahead)
    #[token(">>")]
    LookAhead,

    /// `<<` (positive lookbehind)
    #[token("<<")]
    LookBehind,

    /// `"Hello"` or `'Hello'` (`Hello`)
    #[regex(r#""[^"]*""#)]
    #[regex("'[^']*'")]
    String,

    /// `U+FFF03` (Unicode code point)
    #[regex(r"[Uu]\+[0-9a-fA-F]+")]
    CodePoint,

    /// `12` (number in repetition)
    #[regex(r"\d+", priority = 2)]
    Number,

    /// `hello` (capturing group name)
    #[regex(r"\w[\w\d]*", priority = 1)]
    Identifier,

    // match illegal tokens for which we want to show a better error message
    #[token("^", |_| ParseErrorMsg::Caret)]
    #[token("$", |_| ParseErrorMsg::Dollar)]
    #[regex(r#"\(\?[<!>:=]?"#, |_| ParseErrorMsg::SpecialGroup)]
    #[regex(r#"\\u[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]"#, |_| ParseErrorMsg::BackslashU4)]
    #[regex(r#"\\x[0-9a-fA-F][0-9a-fA-F]"#, |_| ParseErrorMsg::BackslashX2)]
    #[regex(r#"\\[ux]\{[0-9a-fA-F]+\}"#, |_| ParseErrorMsg::BackslashUnicode)]
    #[regex(r#"\\k<[\d\w-]+>"#, |_| ParseErrorMsg::BackslashK)]
    #[regex(r#"\\."#, |_| ParseErrorMsg::Backslash)]
    #[regex(r#""[^"]*"#, |_| ParseErrorMsg::UnclosedString)]
    #[regex("'[^']*", |_| ParseErrorMsg::UnclosedString)]
    ErrorMsg(ParseErrorMsg),

    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex("#.*", logos::skip)]
    #[error]
    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseErrorMsg {
    #[error("`^` is not a valid token. Use `<%` to match the start of the string")]
    Caret,
    #[error("`$` is not a valid token. Use `%>` to match the end of the string")]
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
            Token::Not => "`!`",
            Token::OpenBracket => "`[`",
            Token::Dash => "`-`",
            Token::CloseBracket => "`]`",
            Token::Dot => "`.`",
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
            _ => Err(nom::Err::Error(
                ParseErrorKind::ExpectedToken(*self).at(input.span()),
            )),
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
            _ => Err(nom::Err::Error(
                ParseErrorKind::Expected("word").at(input.span()),
            )),
        }
    }
}
