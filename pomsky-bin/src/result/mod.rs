use std::{fmt, path::Path};

use helptext::text;
use pomsky::diagnose::{DiagnosticCode, DiagnosticKind};
use serde::{Deserialize, Serialize};

use crate::format::Logger;

mod serde_code;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompilationResult {
    /// Schema version
    pub version: Version,
    /// Whether compilation succeeded
    ///
    /// Equivalent to `result.output.is_some()`
    pub success: bool,
    /// File that was compiled
    pub path: Option<String>,
    /// Compilation result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Array of errors and warnings
    pub diagnostics: Vec<Diagnostic>,
    /// Compilation time
    pub timings: Timings,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "1")]
    V1,
}

impl CompilationResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn success(
        path: Option<&Path>,
        output: String,
        time_all_micros: u128,
        time_test_micros: u128,
        diagnostics: impl IntoIterator<Item = pomsky::diagnose::Diagnostic>,
        source_code: &str,
        warnings: &crate::args::DiagnosticSet,
        json: bool,
    ) -> Self {
        Self {
            path: path
                .map(|p| p.canonicalize().as_deref().unwrap_or(p).to_string_lossy().to_string()),
            version: Version::V1,
            success: true,
            output: Some(output),
            diagnostics: Self::convert_diagnostics(diagnostics, source_code, warnings, json),
            timings: Timings::from_micros(time_all_micros, time_test_micros),
        }
    }

    pub(crate) fn error(
        path: Option<&Path>,
        time_all_micros: u128,
        time_test_micros: u128,
        diagnostics: impl IntoIterator<Item = pomsky::diagnose::Diagnostic>,
        source_code: &str,
        warnings: &crate::args::DiagnosticSet,
        json: bool,
    ) -> Self {
        Self {
            path: path
                .map(|p| p.canonicalize().as_deref().unwrap_or(p).to_string_lossy().to_string()),
            version: Version::V1,
            success: false,
            output: None,
            diagnostics: Self::convert_diagnostics(diagnostics, source_code, warnings, json),
            timings: Timings::from_micros(time_all_micros, time_test_micros),
        }
    }

    fn convert_diagnostics(
        diagnostics: impl IntoIterator<Item = pomsky::diagnose::Diagnostic>,
        source_code: &str,
        warnings: &crate::args::DiagnosticSet,
        json: bool,
    ) -> Vec<Diagnostic> {
        let source_code = Some(source_code);
        diagnostics
            .into_iter()
            .filter_map(|d| match d.severity {
                pomsky::diagnose::Severity::Warning if !warnings.is_enabled(d.kind) => None,
                _ => Some(Diagnostic::from(d, source_code, json)),
            })
            .collect()
    }

    pub(crate) fn output(
        self,
        logger: &Logger,
        json: bool,
        new_line: bool,
        in_test_suite: bool,
        source_code: &str,
    ) {
        let success = self.success;
        if json {
            match serde_json::to_string(&self) {
                Ok(string) => println!("{string}"),
                Err(e) => eprintln!("{e}"),
            }
        } else {
            if in_test_suite {
                logger.basic().fmtln(if success { text![G!"ok"] } else { text![R!"failed"] });
            }
            self.output_human_readable(logger, new_line, in_test_suite, Some(source_code));
        }
        if !success && !in_test_suite {
            std::process::exit(1);
        }
    }

    fn output_human_readable(
        mut self,
        logger: &Logger,
        new_line: bool,
        in_test_suite: bool,
        source_code: Option<&str>,
    ) {
        if self.output.is_none() {
            self.diagnostics.retain(|d| d.severity == Severity::Error);
        }
        let initial_len = self.diagnostics.len();

        if self.diagnostics.len() > 8 {
            self.diagnostics.drain(8..);
        }

        for diag in &self.diagnostics {
            diag.print_human_readable(logger, source_code);
        }

        if !self.diagnostics.is_empty() && initial_len > self.diagnostics.len() {
            let omitted = initial_len - self.diagnostics.len();
            logger.note().println(format_args!("{omitted} diagnostic(s) were omitted"));
        }

        if let Some(compiled) = &self.output {
            if in_test_suite {
                // do nothing
            } else if new_line {
                println!("{compiled}");
            } else {
                use std::io::Write;

                print!("{compiled}");
                std::io::stdout().flush().unwrap();
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// "error" | "warning"
    pub severity: Severity,
    /// See [`DiagnosticKind`](pomsky::diagnose::DiagnosticKind)
    ///
    /// Currently "syntax" | "resolve" | "compat" | "unsupported" | "deprecated"
    /// | "limits" | "other"
    pub kind: Kind,
    /// See [`DiagnosticCode`](pomsky::diagnose::DiagnosticCode)
    #[serde(with = "serde_code", skip_serializing_if = "Option::is_none")]
    pub code: Option<DiagnosticCode>,
    /// List of locations that should be underlined
    ///
    /// Currently guaranteed to contain exactly 1 span
    pub spans: Vec<Span>,
    /// Error/warning message
    pub description: String,
    /// Help text
    ///
    /// Currently guaranteed to contain at most 1 string
    pub help: Vec<String>,
    /// Automatically applicable fixes
    ///
    /// Currently unused and guaranteed to be empty
    pub fixes: Vec<QuickFix>,
    /// Visual representation of the diagnostic as displayed in the CLI
    pub visual: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl From<pomsky::diagnose::Severity> for Severity {
    fn from(value: pomsky::diagnose::Severity) -> Self {
        match value {
            pomsky::diagnose::Severity::Error => Severity::Error,
            pomsky::diagnose::Severity::Warning => Severity::Warning,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Syntax,
    Resolve,
    Compat,
    Unsupported,
    Deprecated,
    Limits,
    Test,
    Other,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Syntax => "syntax",
            Kind::Resolve => "resolve",
            Kind::Compat => "compat",
            Kind::Unsupported => "unsupported",
            Kind::Deprecated => "deprecated",
            Kind::Limits => "limits",
            Kind::Test => "test",
            Kind::Other => "other",
        }
    }
}

impl From<DiagnosticKind> for Kind {
    fn from(value: DiagnosticKind) -> Self {
        match value {
            DiagnosticKind::Syntax => Kind::Syntax,
            DiagnosticKind::Resolve => Kind::Resolve,
            DiagnosticKind::Compat => Kind::Compat,
            DiagnosticKind::Unsupported => Kind::Unsupported,
            DiagnosticKind::Deprecated => Kind::Deprecated,
            DiagnosticKind::Limits => Kind::Limits,
            DiagnosticKind::Test => Kind::Test,
            DiagnosticKind::Other => Kind::Other,
            _ => panic!("unknown diagnostic kind"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Timings {
    /// time of all performed compilation steps in microseconds
    pub all: u128,
    pub tests: u128,
}

impl Timings {
    pub fn from_micros(all: u128, tests: u128) -> Self {
        Timings { all, tests }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset, counting from zero, assuming UTF-8 encoding.
    ///
    /// Guaranteed to be non-negative.
    pub start: usize,
    /// End byte offset, non-inclusive, counting from zero, assuming UTF-8
    /// encoding.
    ///
    /// Guaranteed to be at least `start`.
    pub end: usize,
    /// Additional details only relevant to this specific span
    ///
    /// Currently unused, guaranteed to be absent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(value: std::ops::Range<usize>) -> Self {
        Span { start: value.start, end: value.end, label: None }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct QuickFix {
    /// Short description what this quick fix does
    pub description: String,
    /// List of changes to fix this diagnostic.
    ///
    /// Guaranteed to be in source order and non-overlapping (e.g. `1-4`,
    /// `7-12`, `14-15`, `16-16`)
    pub replacements: Vec<Replacement>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Replacement {
    /// Start byte offset, counting from zero, assuming UTF-8 encoding.
    ///
    /// Guaranteed to be non-negative.
    pub start: usize,
    /// End byte offset, non-inclusive, counting from zero, assuming UTF-8
    /// encoding
    ///
    /// Guaranteed to be at least `start`.
    pub end: usize,
    /// Text to replace this part of code with
    pub insert: String,
}

impl Diagnostic {
    pub(crate) fn from(
        value: pomsky::diagnose::Diagnostic,
        source_code: Option<&str>,
        json: bool,
    ) -> Self {
        let kind = value.kind.to_string();
        let severity: &str = value.severity.into();

        let visual = if json {
            let display = value.display_ascii(source_code);
            let visual = match value.code {
                Some(code) => format!("{severity} {code}{kind}:{display}"),
                None => format!("{severity}{kind}:{display}"),
            };
            drop(display);
            visual
        } else {
            // unused
            String::new()
        };

        Diagnostic {
            severity: value.severity.into(),
            kind: value.kind.into(),
            code: value.code,
            spans: value.span.range().into_iter().map(From::from).collect(),
            description: value.msg,
            help: value.help.into_iter().collect(),
            fixes: vec![],
            visual,
        }
    }

    fn print_human_readable(&self, logger: &Logger, source_code: Option<&str>) {
        let kind = self.kind.as_str();
        let display = self.miette_display(source_code);

        match self.code {
            Some(code) => {
                logger.diagnostic_with_code(self.severity, &code.to_string(), kind).print(display);
            }
            None => logger.diagnostic(self.severity, kind).print(display),
        }
    }

    fn miette_display<'a>(&'a self, source_code: Option<&'a str>) -> impl std::fmt::Display + 'a {
        use miette::ReportHandler;
        use std::fmt;

        #[derive(Debug)]
        struct MietteDiagnostic<'a> {
            diagnostic: &'a Diagnostic,
            source_code: Option<&'a str>,
        }

        impl fmt::Display for MietteDiagnostic<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.diagnostic.description.fmt(f)
            }
        }

        impl std::error::Error for MietteDiagnostic<'_> {}

        impl miette::Diagnostic for MietteDiagnostic<'_> {
            fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
                if self.diagnostic.help.is_empty() {
                    None
                } else {
                    let help = self.diagnostic.help.join("\n");
                    Some(Box::new(help))
                }
            }

            fn source_code(&self) -> Option<&dyn miette::SourceCode> {
                self.source_code.as_ref().map(|s| s as &dyn miette::SourceCode)
            }

            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
                if let Some(Span { start, end, label }) = self.diagnostic.spans.first() {
                    let label = label.as_deref().unwrap_or(match self.diagnostic.severity {
                        Severity::Error => "error occurred here",
                        Severity::Warning => "warning originated here",
                    });
                    Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                        Some(label.into()),
                        *start,
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

        struct Handler<'a>(MietteDiagnostic<'a>);

        impl fmt::Display for Handler<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                miette::MietteHandler::default().debug(&self.0, f)
            }
        }

        Handler(MietteDiagnostic { diagnostic: self, source_code })
    }
}

impl fmt::Display for CompilationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
