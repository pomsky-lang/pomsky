//! This module contains errors that can occur during lexing.

/// An error message for a token that is invalid in a pomsky expression.
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

    #[error("The `<%` literal is deprecated.")]
    DeprStart,
    #[error("The `%>` literal is deprecated.")]
    DeprEnd,
}

impl LexErrorMsg {
    /// Returns a help message for fixing this error, if available.
    ///
    /// The `slice` argument must be the same string that you tried to parse.
    pub fn get_help(&self, slice: &str) -> Option<String> {
        super::diagnostics::get_parse_error_msg_help(*self, slice)
    }
}
