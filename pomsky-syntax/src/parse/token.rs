#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum Token {
    /// `^` (start boundary)
    Caret,
    /// `$` (end boundary)
    Dollar,
    /// `<%` (`^` start boundary)
    BStart,
    /// `%>` (`$` end boundary)
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

    // match illegal tokens for which we want to show a better error message
    ErrorMsg(LexErrorMsg),

    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LexErrorMsg {
    #[error("This syntax is not supported")]
    GroupNonCapturing,
    #[error("This syntax is not supported")]
    GroupLookahead,
    #[error("This syntax is not supported")]
    GroupLookaheadNeg,
    #[error("This syntax is not supported")]
    GroupLookbehind,
    #[error("This syntax is not supported")]
    GroupLookbehindNeg,
    #[error("This syntax is not supported")]
    GroupNamedCapture,
    #[error("This syntax is not supported")]
    GroupPcreBackreference,
    #[error("Comments have a different syntax")]
    GroupComment,
    #[error("Atomic groups are not supported")]
    GroupAtomic,
    #[error("Conditionals are not supported")]
    GroupConditional,
    #[error("Branch reset groups are not supported")]
    GroupBranchReset,
    #[error("Subroutines are not supported")]
    GroupSubroutineCall,
    #[error("This syntax is not supported")]
    GroupOther,

    #[error("Backslash escapes are not supported")]
    Backslash,
    #[error("Backslash escapes are not supported")]
    BackslashU4,
    #[error("Backslash escapes are not supported")]
    BackslashX2,
    #[error("Backslash escapes are not supported")]
    BackslashUnicode,
    #[error("Backslash escapes are not supported")]
    BackslashProperty,
    #[error("Backslash escapes are not supported")]
    BackslashGK,

    #[error("This string literal doesn't have a closing quote")]
    UnclosedString,
}

impl LexErrorMsg {
    pub fn get_help(&self, slice: &str) -> Option<String> {
        super::diagnostics::get_parse_error_msg_help(*self, slice)
    }
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Token::Caret => "`^`",
            Token::Dollar => "`$`",
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
