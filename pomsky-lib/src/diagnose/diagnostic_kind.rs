use std::{fmt::Display, str::FromStr};

use pomsky_syntax::diagnose::{ParseErrorKind, ParseWarningKind};

use super::CompileErrorKind;

/// The kind or origin of the error/warning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticKind {
    /// Invalid syntax error
    Syntax,
    /// Error during name resolution
    Resolve,
    /// Diagnostic related to regex flavor compatibility
    Compat,
    /// Diagnostic indicating something is not implemented/allowed in Pomsky
    Unsupported,
    /// Deprecated syntax or feature was used
    Deprecated,
    /// A limitation that was deliberately enforced
    Limits,
    /// The generated regex is invalid (detected after variable expansion)
    Invalid,
    /// Unit test failure
    Test,
    /// Other unspecified error
    Other,
}

impl From<&CompileErrorKind> for DiagnosticKind {
    fn from(kind: &CompileErrorKind) -> Self {
        use CompileErrorKind as K;
        match kind {
            K::ParseError(p) => DiagnosticKind::from(p),
            K::Unsupported(..) => DiagnosticKind::Compat,
            K::UnsupportedPomskySyntax(_) | K::HugeReference => DiagnosticKind::Syntax,
            K::UnknownReferenceNumber(_)
            | K::UnknownReferenceName { .. }
            | K::NameUsedMultipleTimes(_)
            | K::UnknownVariable { .. }
            | K::RelativeRefZero => DiagnosticKind::Resolve,
            K::EmptyClassNegated { .. } | K::IllegalNegation { .. } => DiagnosticKind::Invalid,
            K::CaptureInLet
            | K::ReferenceInLet
            | K::RecursiveVariable
            | K::NegativeShorthandInAsciiMode
            | K::UnicodeInAsciiMode
            | K::JsWordBoundaryInUnicodeMode
            | K::NestedTest
            | K::NegatedHorizVertSpace
            | K::DotNetNumberedRefWithMixedGroups
            | K::RubyLookaheadInLookbehind { .. }
            | K::UnsupportedInLookbehind { .. }
            | K::LookbehindNotConstantLength { .. } => DiagnosticKind::Unsupported,
            K::RangeIsTooBig(_) => DiagnosticKind::Limits,
        }
    }
}

impl From<&ParseErrorKind> for DiagnosticKind {
    fn from(kind: &ParseErrorKind) -> Self {
        match kind {
            ParseErrorKind::LetBindingExists => DiagnosticKind::Resolve,
            ParseErrorKind::Deprecated(_) => DiagnosticKind::Deprecated,
            ParseErrorKind::RecursionLimit => DiagnosticKind::Limits,
            _ => DiagnosticKind::Syntax,
        }
    }
}

impl From<&ParseWarningKind> for DiagnosticKind {
    fn from(kind: &ParseWarningKind) -> Self {
        match kind {
            ParseWarningKind::Deprecation(_) => DiagnosticKind::Deprecated,
        }
    }
}

impl Display for DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DiagnosticKind::Syntax => "(syntax)",
            DiagnosticKind::Resolve => "(resolve)",
            DiagnosticKind::Compat => "(compat)",
            DiagnosticKind::Unsupported => "(unsupported)",
            DiagnosticKind::Deprecated => "(deprecated)",
            DiagnosticKind::Limits => "(limits)",
            DiagnosticKind::Invalid => "(invalid)",
            DiagnosticKind::Test => "(test)",
            DiagnosticKind::Other => "",
        })
    }
}

impl From<DiagnosticKind> for &'static str {
    fn from(val: DiagnosticKind) -> Self {
        match val {
            DiagnosticKind::Syntax => "syntax",
            DiagnosticKind::Resolve => "resolve",
            DiagnosticKind::Compat => "compat",
            DiagnosticKind::Unsupported => "unsupported",
            DiagnosticKind::Deprecated => "deprecated",
            DiagnosticKind::Limits => "limits",
            DiagnosticKind::Invalid => "invalid",
            DiagnosticKind::Test => "test",
            DiagnosticKind::Other => "other",
        }
    }
}

impl FromStr for DiagnosticKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "syntax" => DiagnosticKind::Syntax,
            "resolve" => DiagnosticKind::Resolve,
            "compat" => DiagnosticKind::Compat,
            "unsupported" => DiagnosticKind::Unsupported,
            "deprecated" => DiagnosticKind::Deprecated,
            "limits" => DiagnosticKind::Limits,
            "other" => DiagnosticKind::Other,
            _ => return Err(()),
        })
    }
}
