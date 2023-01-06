use std::{fmt::Display, str::FromStr};

use pomsky_syntax::{error::ParseErrorKind, warning::ParseWarningKind};

use super::CompileErrorKind;

/// The kind or origin of the error/warning
#[cfg_attr(feature = "miette", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Other unspecified error
    Other,
}

impl From<&CompileErrorKind> for DiagnosticKind {
    fn from(kind: &CompileErrorKind) -> Self {
        match kind {
            CompileErrorKind::ParseError(p) => DiagnosticKind::from(p),
            CompileErrorKind::Unsupported(_, _) => DiagnosticKind::Compat,
            CompileErrorKind::UnsupportedPomskySyntax(_) => DiagnosticKind::Syntax,
            CompileErrorKind::HugeReference => DiagnosticKind::Syntax,
            CompileErrorKind::UnknownReferenceNumber(_) => DiagnosticKind::Resolve,
            CompileErrorKind::UnknownReferenceName { .. } => DiagnosticKind::Resolve,
            CompileErrorKind::NameUsedMultipleTimes(_) => DiagnosticKind::Resolve,
            CompileErrorKind::EmptyClass => DiagnosticKind::Syntax,
            CompileErrorKind::EmptyClassNegated { .. } => DiagnosticKind::Resolve,
            CompileErrorKind::CaptureInLet => DiagnosticKind::Unsupported,
            CompileErrorKind::ReferenceInLet => DiagnosticKind::Unsupported,
            CompileErrorKind::UnknownVariable { .. } => DiagnosticKind::Resolve,
            CompileErrorKind::RecursiveVariable => DiagnosticKind::Unsupported,
            CompileErrorKind::RangeIsTooBig(_) => DiagnosticKind::Limits,
            CompileErrorKind::Other(_) => DiagnosticKind::Other,
        }
    }
}

impl From<&ParseErrorKind> for DiagnosticKind {
    fn from(kind: &ParseErrorKind) -> Self {
        match kind {
            ParseErrorKind::Multiple(_) => panic!("Must not be called on multiple errors"),
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
