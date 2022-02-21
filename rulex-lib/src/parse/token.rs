use logos::Logos;
use nom::Parser;

use crate::error::{ParseError, ParseErrorKind};

use super::tokens::Tokens;

#[derive(Debug, Logos, Eq, PartialEq, Copy, Clone)]
pub enum Token {
    /// `%-` (`^` boundary)
    #[token("%-")]
    BStart,
    /// `-%` (`$` boundary)
    #[token("-%")]
    BEnd,
    /// `%!` (`\B` boundary)
    #[token("%!")]
    BNotWord,
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

    /// `!` (postfix negation operator)
    #[token("!")]
    ExclamationMark,

    /// `>>` (positive lookahead)
    #[token(">>")]
    LookAhead,

    /// `<<` (positive lookbehind)
    #[token("<<")]
    LookBehind,

    /// `>>!` (negative lookahead)
    #[token(">>!")]
    LookAheadNeg,

    /// `<<!` (negative lookbehind)
    #[token("<<!")]
    LookBehindNeg,

    /// * `<.>` (`.`)
    /// * `<Hello>` (`\p{Hello}`)
    #[token("<.>")]
    #[regex(r"<\w*>")]
    CWord,

    /// `[abx]`
    #[regex(r#"\[[^\]]*\]"#)]
    CharClass,

    /// `"Hello"` (`Hello`)
    #[regex(r#""[^"]*""#)]
    DoubleString,
    /// `'Hello'` (`Hello`)
    #[regex("'[^']*'")]
    SingleString,

    /// `U+FFF03` (Unicode code point)
    #[regex(r"U\+[\da-fA-F]+")]
    CodePoint,

    /// `12` (number in repetition)
    #[regex(r"\d+", priority = 2)]
    RepetitionCount,

    /// `hello` (capturing group name)
    #[regex(r"\w[\w\d]*", priority = 1)]
    GroupName,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex("#.*", logos::skip)]
    #[error]
    Error,
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Token::BStart => "`%-`",
            Token::BEnd => "`-%`",
            Token::BNotWord => "`%!`",
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
            Token::ExclamationMark => "`!`",
            Token::LookAhead => "`>>`",
            Token::LookBehind => "`<<`",
            Token::LookAheadNeg => "`>>!`",
            Token::LookBehindNeg => "`<<!`",
            Token::CWord => "named character class",
            Token::CharClass => "character class",
            Token::DoubleString => "string",
            Token::SingleString => "string",
            Token::CodePoint => "code point",
            Token::RepetitionCount => "repetition number",
            Token::GroupName => "group name",
            Token::Error => "error",
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
