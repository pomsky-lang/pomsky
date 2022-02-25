use logos::Logos;
use nom::Parser;

use crate::error::{ParseError, ParseErrorKind};

use super::tokens::Tokens;

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

    /// `-` (unicode range)
    #[token("-")]
    Dash,
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

    /// `[` (open character class)
    #[token("[")]
    OpenBracket,

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
    #[regex(
        r#"\\(u[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]|x[0-9a-fA-F][0-9a-fA-F]|.)"#,
        |_| ParseErrorMsg::BackslashSequence
    )]
    ErrorMsg(ParseErrorMsg),

    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex("#.*", logos::skip)]
    #[error]
    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseErrorMsg {
    #[error("A character class can't be empty")]
    EmptyCharClass,
    #[error("The dot must be surrounded by angle brackets: <.>")]
    Dot,
    #[error("`^` is not a valid token. Use `<%` to match the start of the string")]
    Caret,
    #[error("`$` is not a valid token. Use `%>` to match the end of the string")]
    Dollar,
    #[error("There's an unmatched square bracket")]
    UnmatchedBracket,
    #[error("This syntax is not supported")]
    SpecialGroup,
    #[error("Backslash escapes are not supported")]
    BackslashSequence,
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
            Token::Dash => "`-`",
            Token::Pipe => "`|`",
            Token::Colon => "`:`",
            Token::OpenParen => "`(`",
            Token::CloseParen => "`)`",
            Token::OpenBrace => "`{`",
            Token::CloseBrace => "`}`",
            Token::Comma => "`,`",
            Token::LookAhead => "`>>`",
            Token::LookBehind => "`<<`",
            Token::OpenBracket => "`[`",
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

impl<'i, 'b> Parser<Tokens<'i, 'b>, &'i str, ParseError> for Token {
    fn parse(
        &mut self,
        mut input: Tokens<'i, 'b>,
    ) -> nom::IResult<Tokens<'i, 'b>, &'i str, ParseError> {
        match input.peek() {
            Some((t, s)) if t == *self => {
                let _ = input.next();
                Ok((input, s))
            }
            _ => Err(nom::Err::Error(
                ParseErrorKind::ExpectedToken(*self).at(input.index()),
            )),
        }
    }
}

impl<'i, 'b> Parser<Tokens<'i, 'b>, Token, ParseError> for &'i str {
    fn parse(
        &mut self,
        mut input: Tokens<'i, 'b>,
    ) -> nom::IResult<Tokens<'i, 'b>, Token, ParseError> {
        match input.peek() {
            Some((t, s)) if s == *self => {
                let _ = input.next();
                Ok((input, t))
            }
            _ => Err(nom::Err::Error(
                ParseErrorKind::Expected("word").at(input.index()),
            )),
        }
    }
}
