pub use crate::{error::*, warning::*};

use crate::Span;

#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ParseDiagnosticKind {
    Error(ParseErrorKind),
    Warning(ParseWarningKind),
}

impl From<ParseError> for ParseDiagnostic {
    fn from(value: ParseError) -> Self {
        ParseDiagnostic { kind: ParseDiagnosticKind::Error(value.kind), span: value.span }
    }
}

impl From<ParseWarning> for ParseDiagnostic {
    fn from(value: ParseWarning) -> Self {
        ParseDiagnostic { kind: ParseDiagnosticKind::Warning(value.kind), span: value.span }
    }
}
