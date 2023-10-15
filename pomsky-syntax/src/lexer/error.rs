//! This module contains errors that can occur during lexing.

/// An error message for a token that is invalid in a pomsky expression.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexErrorMsg {
    GroupNonCapturing,
    GroupLookahead,
    GroupLookaheadNeg,
    GroupLookbehind,
    GroupLookbehindNeg,
    GroupNamedCapture,
    GroupPcreBackreference,
    GroupComment,
    GroupAtomic,
    GroupConditional,
    GroupBranchReset,
    GroupSubroutineCall,
    GroupOther,

    Backslash,
    BackslashU4,
    BackslashX2,
    BackslashUnicode,
    BackslashProperty,
    BackslashGK,

    UnclosedString,
    LeadingZero,
    InvalidCodePoint,
}

impl LexErrorMsg {
    /// Returns a help message for fixing this error, if available.
    ///
    /// The `slice` argument must be the same string that you tried to parse.
    pub fn get_help(&self, slice: &str) -> Option<String> {
        super::diagnostics::get_parse_error_msg_help(*self, slice)
    }
}

impl std::error::Error for LexErrorMsg {}

impl core::fmt::Display for LexErrorMsg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = match self {
            LexErrorMsg::GroupNonCapturing
            | LexErrorMsg::GroupLookahead
            | LexErrorMsg::GroupLookaheadNeg
            | LexErrorMsg::GroupLookbehind
            | LexErrorMsg::GroupLookbehindNeg
            | LexErrorMsg::GroupNamedCapture
            | LexErrorMsg::GroupPcreBackreference
            | LexErrorMsg::GroupOther => "This syntax is not supported",
            LexErrorMsg::GroupComment => "Comments have a different syntax",
            LexErrorMsg::GroupAtomic => "Atomic groups are not supported",
            LexErrorMsg::GroupConditional => "Conditionals are not supported",
            LexErrorMsg::GroupBranchReset => "Branch reset groups are not supported",
            LexErrorMsg::GroupSubroutineCall => "Subroutines are not supported",

            LexErrorMsg::Backslash
            | LexErrorMsg::BackslashU4
            | LexErrorMsg::BackslashX2
            | LexErrorMsg::BackslashUnicode
            | LexErrorMsg::BackslashProperty
            | LexErrorMsg::BackslashGK => "Backslash escapes are not supported",

            LexErrorMsg::UnclosedString => "This string literal doesn't have a closing quote",
            LexErrorMsg::LeadingZero => "Numbers can't have leading zeroes",
            LexErrorMsg::InvalidCodePoint => "Code point contains non-hexadecimal digit",
        };

        f.write_str(error)
    }
}
