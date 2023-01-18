//! Compilation warnings

use std::fmt;

use pomsky_syntax::Span;

use super::{Diagnostic, DiagnosticCode, DiagnosticKind, Severity};

#[derive(Clone)]
pub struct CompileWarning {
    pub(crate) kind: CompileWarningKind,
    pub(crate) span: Span,
}

impl CompileWarning {
    pub(crate) fn diagnostic(&self) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            msg: self.kind.to_string(),
            code: Some(self.kind.code()),
            help: self.kind.help(),
            span: self.span,
            kind: DiagnosticKind::Compat,
        }
    }
}

/// A warning emitted during compilation
#[derive(Clone)]
pub enum CompileWarningKind {
    /// Compatibility warning
    Compat(CompatWarning),
}

impl CompileWarningKind {
    fn code(&self) -> DiagnosticCode {
        match self {
            CompileWarningKind::Compat(CompatWarning::JsLookbehind) => {
                DiagnosticCode::PossiblyUnsupported
            }
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            CompileWarningKind::Compat(CompatWarning::JsLookbehind) => {
                Some("Avoid lookbehind if the regex should work in different browsers".into())
            }
        }
    }
}

impl fmt::Display for CompileWarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileWarningKind::Compat(c) => c.fmt(f),
        }
    }
}

impl CompileWarningKind {
    pub(crate) fn at(self, span: Span) -> CompileWarning {
        CompileWarning { kind: self, span }
    }
}

/// A compatibility warning: Indicates that something might not be supported
/// everywhere
#[derive(Debug, Clone, Copy)]
pub enum CompatWarning {
    /// Lookbehind encountered targeting the JS flavor
    JsLookbehind,
}

impl fmt::Display for CompatWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatWarning::JsLookbehind => {
                f.write_str("Lookbehind is not supported in all browsers, e.g. Safari")
            }
        }
    }
}
