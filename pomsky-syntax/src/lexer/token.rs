use super::LexErrorMsg;

/// A token encountered while lexing a pomsky expression.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum Token {
    /// `^` (start boundary)
    Caret,
    /// `$` (end boundary)
    Dollar,
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

    /// `!` (negation)
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

    /// `<` (word start)
    AngleLeft,

    /// `>` (word end)
    AngleRight,

    /// `::` (back reference)
    DoubleColon,

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

    /// `lazy` (reserved name)
    ReservedName,

    /// Illegal token for which we want to show a better error message
    ErrorMsg(LexErrorMsg),

    /// Token representing an unknown character
    Error,
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Token::Caret => "`^`",
            Token::Dollar => "`$`",
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
            Token::AngleLeft => "`<`",
            Token::AngleRight => "`>`",
            Token::DoubleColon => "`::`",
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
            Token::ReservedName => "reserved name",
            Token::ErrorMsg(_) | Token::Error => "error",
        })
    }
}
