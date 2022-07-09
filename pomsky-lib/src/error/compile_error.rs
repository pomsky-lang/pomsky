use crate::{options::RegexFlavor, span::Span};

use super::{Diagnostic, ParseError, ParseErrorKind};

/// An error that can occur during parsing or compiling
#[derive(Debug, Clone, thiserror::Error)]
pub struct CompileError {
    pub(super) kind: CompileErrorKind,
    pub(super) span: Span,
}

impl CompileError {
    pub(crate) fn set_missing_span(&mut self, span: Span) {
        if self.span.is_empty() {
            self.span = span;
        }
    }

    /// Create a [Diagnostic] from this error.
    pub fn diagnostic(self, source_code: &str) -> Diagnostic {
        Diagnostic::from_compile_error(self, source_code)
    }

    /// Create one or more [Diagnostic]s from this error.
    pub fn diagnostics(self, source_code: &str) -> Vec<Diagnostic> {
        Diagnostic::from_compile_errors(self, source_code)
    }
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {}..{}", self.kind, start, end)
        } else {
            self.kind.fmt(f)
        }
    }
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        CompileError { kind: CompileErrorKind::ParseError(e.kind), span: e.span }
    }
}

/// An error kind (without span) that can occur during parsing or compiling
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum CompileErrorKind {
    #[error("Parse error: {}", .0)]
    ParseError(ParseErrorKind),

    #[error("Compile error: Unsupported feature `{}` in the `{:?}` regex flavor", .0.name(), .1)]
    Unsupported(Feature, RegexFlavor),

    #[error("Group references this large aren't supported")]
    HugeReference,

    #[error("Reference to unknown group. There is no group number {}", .0)]
    UnknownReferenceNumber(i32),

    #[error("Reference to unknown group. There is no group named `{}`", .0)]
    UnknownReferenceName(String),

    #[error("Compile error: Group name `{}` used multiple times", .0)]
    NameUsedMultipleTimes(String),

    #[error("Compile error: This character class is empty")]
    EmptyClass,

    #[error("Compile error: This negated character class matches nothing")]
    EmptyClassNegated,

    #[error("Capturing groups within `let` statements are currently not supported")]
    CaptureInLet,

    #[error("References within `let` statements are currently not supported")]
    ReferenceInLet,

    #[error("Variable doesn't exist")]
    UnknownVariable,

    #[error("Variables can't be used recursively")]
    RecursiveVariable,

    #[error("Compile error: {}", .0)]
    Other(&'static str),
}

impl CompileErrorKind {
    pub(crate) fn at(self, span: Span) -> CompileError {
        CompileError { kind: self, span }
    }
}

/// A regex feature, which might not be supported in every regex flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Feature {
    /// Named capturing groups, e.g. `(?<name>group)`
    NamedCaptureGroups,
    /// Lookahead or lookbehind, e.g. `(?=lookahead)`
    Lookaround,
    /// A single grapheme cluster, `\X`
    Grapheme,
    /// Unicode blocks, e.g. `\p{InBasic_Latin}`
    UnicodeBlock,
    /// Unicode properties, e.g. `\p{Whitespace}`
    UnicodeProp,
    /// Backreferences, e.g. `\4`
    Backreference,
    /// Forward references. They're like backreferences, but refer to a group
    /// that syntactically appears _after_ the reference
    ForwardReference,
    /// A numeric reference relative to the current position, e.g. `\k<-2>`.
    ///
    /// Note that this enum variant is currently unused, because relative
    /// references are converted to absolute references by Pomsky.
    // TODO: maybe remove in next major version
    RelativeReference,
    /// A relative reference with a relative index of 0 or higher, e.g. `\k<-0>`
    /// or `\k<+3>`. These aren't supported in any regex engine that I know
    /// of.
    ///
    /// Note that this enum variant is currently unused, because relative
    /// references are converted to absolute references by Pomsky.
    // TODO: maybe remove in next major version
    NonNegativeRelativeReference,
    /// Negative `\w` shorthand, i.e. `[\W]`. This is not supported in
    /// JavaScript when polyfilling Unicode support for `\w` and `\d`.
    NegativeShorthandW,
}

impl Feature {
    fn name(self) -> &'static str {
        match self {
            Feature::NamedCaptureGroups => "named capturing groups",
            Feature::Lookaround => "lookahead/behind",
            Feature::Grapheme => "grapheme cluster matcher (\\X)",
            Feature::UnicodeBlock => "Unicode blocks (\\p{InBlock})",
            Feature::UnicodeProp => "Unicode properties (\\p{Property})",
            Feature::Backreference => "Backreference",
            Feature::ForwardReference => "Forward reference",
            Feature::RelativeReference => "Relative backreference",
            Feature::NonNegativeRelativeReference => "Non-negative relative backreference",
            Feature::NegativeShorthandW => "Negative `\\w` shorthand in character class",
        }
    }
}
