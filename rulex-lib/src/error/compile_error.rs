use crate::{options::RegexFlavor, span::Span};

use super::{ParseError, ParseErrorKind};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub struct CompileError {
    pub(super) kind: CompileErrorKind,
    pub(super) span: Span,
}

impl CompileErrorKind {
    pub(crate) fn at(self, span: Span) -> CompileError {
        CompileError { kind: self, span }
    }
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n  at {}", self.kind, self.span)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompileErrorKind {
    #[error("Parse error: {}", .0)]
    ParseError(ParseErrorKind),

    #[error("Compile error: Unsupported feature `{}` in the `{:?}` regex flavor", .0.name(), .1)]
    Unsupported(Feature, RegexFlavor),

    #[error("Compile error: Group name `{}` used multiple times", .0)]
    NameUsedMultipleTimes(String),

    #[error("Compile error: This character class is empty")]
    EmptyClass,

    #[error("Compile error: This negated character class matches nothing")]
    EmptyClassNegated,

    #[error("Compile error: `{}` can't be negated within a character class", .0)]
    UnsupportedNegatedClass(String),

    #[error("Compile error: {}", .0)]
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    NamedCaptureGroups,
    Lookaround,
    Grapheme,
    UnicodeLineBreak,
}

impl Feature {
    fn name(self) -> &'static str {
        match self {
            Feature::NamedCaptureGroups => "named capture groups",
            Feature::Lookaround => "lookahead/behind",
            Feature::Grapheme => "grapheme cluster matcher (\\X)",
            Feature::UnicodeLineBreak => "Unicode line break (\\R)",
        }
    }
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        CompileErrorKind::ParseError(e.kind).at(e.span)
    }
}
