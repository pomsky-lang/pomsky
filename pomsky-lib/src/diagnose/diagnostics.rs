use pomsky_syntax::{
    diagnose::{ParseDiagnostic, ParseDiagnosticKind, ParseErrorKind, ParseWarningKind},
    Span,
};

use super::{
    diagnostic_code::DiagnosticCode, help::get_compiler_help, CompileError, CompileErrorKind,
    DiagnosticKind,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
/// A struct containing detailed information about an error, which can be
/// displayed beautifully with [miette](https://docs.rs/miette/latest/miette/).
pub struct Diagnostic {
    /// Whether this is an error, a warning or advice
    pub severity: Severity,
    /// The error message
    pub msg: String,
    /// The error code (optional)
    pub code: Option<DiagnosticCode>,
    /// An (optional) help message explaining how the error could be fixed
    pub help: Option<String>,
    /// The start and end byte positions of the source code where the error
    /// occurred.
    pub span: Span,
    /// The kind or origin of error/warning
    pub kind: DiagnosticKind,
}

#[cfg(feature = "miette")]
impl std::error::Error for Diagnostic {}

#[cfg(feature = "miette")]
impl core::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.msg.fmt(f)
    }
}

/// Indicates whether a diagnostic is an error or a warning
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    /// Error
    Error,
    /// Warning
    Warning,
}

impl From<Severity> for &'static str {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

impl Diagnostic {
    pub(crate) fn from_parse_error(
        error_span: Span,
        kind: &ParseErrorKind,
        source_code: &str,
    ) -> Self {
        let range = error_span.range().unwrap_or(0..source_code.len());
        let slice = &source_code[range.clone()];
        let mut span = Span::from(range);

        let help = super::help::get_parser_help(kind, slice, &mut span);
        let code = Some(DiagnosticCode::from(kind));

        Diagnostic {
            severity: Severity::Error,
            code,
            msg: kind.to_string(),
            help,
            span,
            kind: DiagnosticKind::from(kind),
        }
    }

    pub(crate) fn from_compile_error(err: &CompileError, source_code: &str) -> Self {
        let CompileError { kind, span: error_span } = err;

        match kind {
            CompileErrorKind::ParseError(kind) => {
                Diagnostic::from_parse_error(*error_span, kind, source_code)
            }
            _ => {
                let range = error_span.range().unwrap_or(0..source_code.len());
                let slice = &source_code[range.clone()];
                let span = Span::from(range);

                let help = get_compiler_help(kind, slice, span);

                Diagnostic {
                    severity: Severity::Error,
                    code: Some(DiagnosticCode::from(kind)),
                    msg: kind.to_string(),
                    help,
                    span,
                    kind: DiagnosticKind::from(kind),
                }
            }
        }
    }

    pub(crate) fn from_warning(span: Span, kind: &ParseWarningKind, source_code: &str) -> Self {
        let range = span.range().unwrap_or(0..source_code.len());
        let span = Span::from(range);

        Diagnostic {
            severity: Severity::Warning,
            code: Some(DiagnosticCode::from(kind)),
            msg: kind.to_string(),
            help: None,
            span,
            kind: DiagnosticKind::from(kind),
        }
    }

    pub(crate) fn from_parser(diagnostic: &ParseDiagnostic, source_code: &str) -> Self {
        let span = diagnostic.span;
        match &diagnostic.kind {
            ParseDiagnosticKind::Error(e) => Diagnostic::from_parse_error(span, e, source_code),
            ParseDiagnosticKind::Warning(w) => Diagnostic::from_warning(span, w, source_code),
        }
    }

    /// Create a test failure diagnostic
    #[must_use]
    pub fn test_failure(span: Span, code: DiagnosticCode, actual_value: Option<&str>) -> Self {
        let (msg, help) = match code {
            DiagnosticCode::TestNoExactMatch => {
                ("The regex does not exactly match the test string".into(), None)
            }
            DiagnosticCode::TestMissingSubstringMatch => {
                ("The regex did not find this match within the test string".into(), None)
            }
            DiagnosticCode::TestUnexpectedSubstringMatch => (
                "The regex found an unexpected match within the test string".into(),
                Some(format!("The regex matched the substring {:?}", actual_value.unwrap())),
            ),
            DiagnosticCode::TestWrongSubstringMatch => (
                "The regex found a different match in the test string".into(),
                Some(format!("The actual match is {:?}", actual_value.unwrap())),
            ),
            DiagnosticCode::TestUnexpectedExactMatch => (
                "The regex exactly matches the test string, but no match was expected".into(),
                None,
            ),
            DiagnosticCode::TestMissingCaptureGroup => {
                ("The regex match does not have the expected capture group".into(), None)
            }
            DiagnosticCode::TestWrongCaptureGroup => (
                "The capture group does not have the expected content".into(),
                Some(format!("The actual content is {:?}", actual_value.unwrap())),
            ),
            _ => unreachable!("An unexpected diagnostic code was passed to `test_failure`"),
        };

        Diagnostic {
            severity: Severity::Error,
            code: Some(code),
            msg,
            help,
            span,
            kind: DiagnosticKind::Test,
        }
    }

    /// Create an ad-hoc diagnostic without a source code snippet
    #[must_use]
    pub fn ad_hoc(
        severity: Severity,
        code: Option<DiagnosticCode>,
        msg: String,
        help: Option<String>,
    ) -> Self {
        Diagnostic { severity, code, msg, help, span: Span::empty(), kind: DiagnosticKind::Other }
    }

    /// Returns a value that can display the diagnostic with the [`Display`]
    /// trait.
    #[cfg(feature = "miette")]
    #[must_use]
    pub fn default_display(
        &self,
        source_code: Option<impl Into<String>>,
    ) -> impl std::fmt::Display + '_ {
        use miette::ReportHandler;
        use std::fmt;

        #[derive(Debug)]
        struct MietteDiagnostic {
            diagnostic: Diagnostic,
            code: Option<String>,
        }

        impl fmt::Display for MietteDiagnostic {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.diagnostic.fmt(f)
            }
        }

        impl std::error::Error for MietteDiagnostic {}

        impl miette::Diagnostic for MietteDiagnostic {
            fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
                self.diagnostic.help.as_deref().map(|h| Box::new(h) as Box<dyn fmt::Display + 'a>)
            }

            fn source_code(&self) -> Option<&dyn miette::SourceCode> {
                self.code.as_ref().map(|s| s as &dyn miette::SourceCode)
            }

            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
                if let Some(std::ops::Range { start, end }) = self.diagnostic.span.range() {
                    let label = match self.diagnostic.severity {
                        Severity::Error => "error occurred here",
                        Severity::Warning => "warning originated here",
                    };
                    Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                        Some(label.into()),
                        start,
                        end - start,
                    ))))
                } else {
                    None
                }
            }

            fn severity(&self) -> Option<miette::Severity> {
                Some(match self.diagnostic.severity {
                    Severity::Error => miette::Severity::Error,
                    Severity::Warning => miette::Severity::Warning,
                })
            }
        }

        struct Handler(MietteDiagnostic);

        impl fmt::Display for Handler {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                miette::MietteHandler::default().debug(&self.0, f)
            }
        }

        Handler(MietteDiagnostic { diagnostic: self.clone(), code: source_code.map(Into::into) })
    }
}
