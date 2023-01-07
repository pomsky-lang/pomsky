use pomsky_syntax::{
    diagnose::{ParseDiagnostic, ParseDiagnosticKind, ParseErrorKind, ParseWarningKind},
    Span,
};

use super::{diagnostic_code::DiagnosticCode, CompileError, CompileErrorKind, DiagnosticKind};

#[derive(Debug)]
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
    /// The source code where the error occurred
    pub source_code: Option<String>,
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
#[derive(Debug)]
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

#[cfg(feature = "miette")]
impl miette::Diagnostic for Diagnostic {
    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help.as_deref().map(|h| Box::new(h) as Box<dyn std::fmt::Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source_code.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            let label = match self.severity {
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
        Some(match self.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
        })
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

        let help = super::help::get_help(kind, slice, &mut span);
        let code = Some(DiagnosticCode::from(kind));

        Diagnostic {
            severity: Severity::Error,
            code,
            msg: kind.to_string(),
            source_code: Some(source_code.into()),
            help,
            span,
            kind: DiagnosticKind::from(kind),
        }
    }

    pub(crate) fn from_compile_error(err: &CompileError, source_code: &str) -> Self {
        let CompileError { kind, span } = err;
        match kind {
            CompileErrorKind::ParseError(kind) => {
                Diagnostic::from_parse_error(*span, kind, source_code)
            }
            #[cfg(feature = "suggestions")]
            CompileErrorKind::UnknownVariable { similar: Some(ref similar), .. }
            | CompileErrorKind::UnknownReferenceName { similar: Some(ref similar), .. } => {
                let range = span.range().unwrap_or(0..source_code.len());
                let code = Some(DiagnosticCode::from(kind));

                Diagnostic {
                    severity: Severity::Error,
                    code,
                    msg: kind.to_string(),
                    source_code: Some(source_code.into()),
                    help: Some(format!("Perhaps you meant `{similar}`")),
                    span: Span::from(range),
                    kind: DiagnosticKind::Resolve,
                }
            }
            CompileErrorKind::EmptyClassNegated { group1, group2 } => {
                let range = span.range().unwrap_or(0..source_code.len());
                let code = Some(DiagnosticCode::from(kind));

                Diagnostic {
                    severity: Severity::Error,
                    code,
                    msg: kind.to_string(),
                    source_code: Some(source_code.into()),
                    help: Some(format!(
                        "The group is empty because it contains both \
                        `{group1:?}` and `{group2:?}`, which together match every code point",
                    )),
                    span: Span::from(range),
                    kind: DiagnosticKind::Resolve,
                }
            }
            kind => {
                let range = span.range().unwrap_or(0..source_code.len());
                let span = Span::from(range);
                let code = Some(DiagnosticCode::from(kind));

                Diagnostic {
                    severity: Severity::Error,
                    code,
                    msg: kind.to_string(),
                    source_code: Some(source_code.into()),
                    help: None,
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
            source_code: Some(source_code.into()),
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

    /// Create an ad-hoc diagnostic without a source code snippet
    #[must_use]
    pub fn ad_hoc(
        severity: Severity,
        code: Option<DiagnosticCode>,
        msg: String,
        help: Option<String>,
    ) -> Self {
        Diagnostic {
            severity,
            code,
            msg,
            source_code: None,
            help,
            span: Span::empty(),
            kind: DiagnosticKind::Other,
        }
    }

    /// Returns a value that can display the diagnostic with the [`Display`]
    /// trait.
    #[cfg(feature = "miette")]
    #[must_use]
    pub fn default_display(&self) -> impl std::fmt::Display + '_ {
        use miette::ReportHandler;
        use std::fmt;

        #[derive(Debug)]
        struct DiagnosticPrinter<'a>(&'a Diagnostic);

        impl fmt::Display for DiagnosticPrinter<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                miette::MietteHandler::default().debug(self.0, f)
            }
        }

        DiagnosticPrinter(self)
    }
}
