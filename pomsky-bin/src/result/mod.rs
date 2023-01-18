use std::fmt;

use pomsky::diagnose::{DiagnosticCode, DiagnosticKind};
use serde::{Deserialize, Serialize};

mod serde_code;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompilationResult {
    /// Schema version
    pub version: Version,
    /// Whether compilation succeeded
    ///
    /// Equivalent to `result.output.is_some()`
    pub success: bool,
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
    pub fn success(output: String, time_micros: u128) -> Self {
        Self {
            version: Version::V1,
            success: true,
            output: Some(output),
            diagnostics: vec![],
            timings: Timings::from_micros(time_micros),
        }
    }

    pub fn error(time_micros: u128) -> Self {
        Self {
            version: Version::V1,
            success: false,
            output: None,
            diagnostics: vec![],
            timings: Timings::from_micros(time_micros),
        }
    }

    pub fn with_diagnostics(
        mut self,
        diagnostics: impl IntoIterator<Item = pomsky::diagnose::Diagnostic>,
        source_code: Option<&str>,
    ) -> Self {
        self.diagnostics.extend(diagnostics.into_iter().map(|d| Diagnostic::from(d, source_code)));
        self
    }

    pub fn output_json(&self) {
        match serde_json::to_string(self) {
            Ok(string) => println!("{string}"),
            Err(e) => eprintln!("{e}"),
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    Other,
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
            DiagnosticKind::Other => Kind::Other,
            _ => panic!("unknown diagnostic kind"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Timings {
    /// time of all performed compilation steps in microseconds
    pub all: u128,
}

impl Timings {
    pub fn from_micros(micros: u128) -> Self {
        Timings { all: micros }
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
    fn from(value: pomsky::diagnose::Diagnostic, source_code: Option<&str>) -> Self {
        let kind = value.kind.to_string();
        let display = value.default_display(source_code);
        let severity: &str = value.severity.into();

        let visual = match value.code {
            Some(code) => format!("{severity} {code}{kind}: {display}"),
            None => format!("{severity}{kind}: {display}"),
        };
        drop(display);

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
}

impl fmt::Display for CompilationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
