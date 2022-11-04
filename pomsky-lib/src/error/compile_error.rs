use pomsky_syntax::{error::ParseErrorKind, Span};

use crate::options::RegexFlavor;

use super::{Diagnostic, ParseError};

/// An error that can occur during parsing or compiling
#[derive(Debug, Clone)]
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
    #[must_use]
    pub fn diagnostic(self, source_code: &str) -> Diagnostic {
        Diagnostic::from_compile_error(self, source_code)
    }

    /// Create one or more [Diagnostic]s from this error.
    #[must_use]
    pub fn diagnostics(self, source_code: &str) -> Vec<Diagnostic> {
        Diagnostic::from_compile_errors(self, source_code)
    }
}

impl std::error::Error for CompileError {}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {start}..{end}", self.kind)
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum CompileErrorKind {
    ParseError(ParseErrorKind),
    Unsupported(Feature, RegexFlavor),
    UnsupportedPomskySyntax(UnsupportedError),
    HugeReference,
    UnknownReferenceNumber(i32),
    UnknownReferenceName {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },
    NameUsedMultipleTimes(String),
    EmptyClass,
    EmptyClassNegated,
    CaptureInLet,
    ReferenceInLet,
    UnknownVariable {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },
    RecursiveVariable,
    RangeIsTooBig(u8),
    Other(&'static str),
}

impl CompileErrorKind {
    pub(crate) fn at(self, span: Span) -> CompileError {
        CompileError { kind: self, span }
    }
}

impl std::error::Error for CompileErrorKind {}

impl core::fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompileErrorKind::ParseError(kind) => write!(f, "Parse error: {kind}"),
            CompileErrorKind::Unsupported(feature, flavor) => {
                write!(
                    f,
                    "Compile error: Unsupported feature `{}` in the `{flavor:?}` regex flavor",
                    feature.name(),
                )
            }
            CompileErrorKind::UnsupportedPomskySyntax(inner) => inner.fmt(f),
            CompileErrorKind::HugeReference => {
                write!(f, "Group references this large aren't supported")
            }
            CompileErrorKind::UnknownReferenceNumber(group) => {
                write!(f, "Reference to unknown group. There is no group number {group}")
            }
            CompileErrorKind::UnknownReferenceName { found, .. } => {
                write!(f, "Reference to unknown group. There is no group named `{found}`")
            }
            CompileErrorKind::NameUsedMultipleTimes(name) => {
                write!(f, "Compile error: Group name `{name}` used multiple times")
            }
            CompileErrorKind::EmptyClass => {
                write!(f, "Compile error: This character class is empty")
            }
            CompileErrorKind::EmptyClassNegated => {
                write!(f, "Compile error: This negated character class matches nothing")
            }
            CompileErrorKind::CaptureInLet => {
                write!(f, "Capturing groups within `let` statements are currently not supported")
            }
            CompileErrorKind::ReferenceInLet => {
                write!(f, "References within `let` statements are currently not supported")
            }
            CompileErrorKind::UnknownVariable { found, .. } => {
                write!(f, "Variable `{found}` doesn't exist")
            }
            CompileErrorKind::RecursiveVariable => write!(f, "Variables can't be used recursively"),
            CompileErrorKind::RangeIsTooBig(digits) => {
                write!(f, "Range is too big, it isn't allowed to contain more than {digits} digits")
            }
            CompileErrorKind::Other(error) => write!(f, "Compile error: {error}"),
        }
    }
}

/// A regex feature, which might not be supported in every regex flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Feature {
    /// Named capturing groups, e.g. `(?<name>group)`
    NamedCaptureGroups,
    /// Atomic groups, e.g. `(?>group)`
    AtomicGroups,
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
            Feature::AtomicGroups => "atomic groups",
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

/// An error that indicates that an unsupported feature was used.
///
/// See [`crate::features::PomskyFeatures`] for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum UnsupportedError {
    Grapheme,
    NumberedGroups,
    NamedGroups,
    AtomicGroups,
    References,
    LazyMode,
    Ranges,
    Variables,
    Lookahead,
    Lookbehind,
    Boundaries,
    Regexes,
    Dot,
}

impl std::error::Error for UnsupportedError {}

impl core::fmt::Display for UnsupportedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = match self {
            UnsupportedError::Grapheme => "Grapheme isn't supported",
            UnsupportedError::NumberedGroups => "Numbered capturing groups aren't supported",
            UnsupportedError::NamedGroups => "Named capturing groups aren't supported",
            UnsupportedError::AtomicGroups => "Atomic groups aren't supported",
            UnsupportedError::References => "References aren't supported",
            UnsupportedError::LazyMode => "Lazy mode isn't supported",
            UnsupportedError::Ranges => "Ranges aren't supported",
            UnsupportedError::Variables => "Variables aren't supported",
            UnsupportedError::Lookahead => "Lookahead isn't supported",
            UnsupportedError::Lookbehind => "Lookbehind isn't supported",
            UnsupportedError::Boundaries => "Word boundaries aren't supported",
            UnsupportedError::Regexes => "Unescaped regexes aren't supported",
            UnsupportedError::Dot => "The dot isn't supported",
        };

        f.write_str(error)
    }
}
