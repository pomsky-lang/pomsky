use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct CompilationResult {
    /// Schema version
    ///
    /// Currently "1"
    version: &'static str,
    /// Whether compilation succeeded
    ///
    /// Equivalent to `result.output.is_some()`
    success: bool,
    /// Compilation result
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    /// Array of errors and warnings
    diagnostics: Vec<Diagnostic>,
    /// Compilation time
    timings: Timings,
}

impl CompilationResult {
    pub(crate) fn success(output: String, time_micros: u128) -> Self {
        Self {
            version: "1",
            success: true,
            output: Some(output),
            diagnostics: vec![],
            timings: Timings::from_micros(time_micros),
        }
    }

    pub(crate) fn error(time_micros: u128) -> Self {
        Self {
            version: "1",
            success: false,
            output: None,
            diagnostics: vec![],
            timings: Timings::from_micros(time_micros),
        }
    }

    pub(crate) fn with_diagnostics(
        mut self,
        diagnostics: impl IntoIterator<Item = pomsky::error::Diagnostic>,
    ) -> Self {
        self.diagnostics.extend(diagnostics.into_iter().map(From::from));
        self
    }

    pub(crate) fn output_json(&self) {
        match serde_json::to_string(self) {
            Ok(string) => println!("{string}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Diagnostic {
    /// "error" | "warning"
    severity: &'static str,
    /// See [`DiagnosticKind`](pomsky::error::DiagnosticKind)
    ///
    /// Currently "syntax" | "resolve" | "compat" | "unsupported" | "deprecated"
    /// | "limits" | "other"
    kind: &'static str,
    /// List of locations that should be underlined
    ///
    /// Currently guaranteed to contain exactly 1 span
    spans: Vec<Span>,
    /// Error/warning message
    description: String,
    /// Help text
    ///
    /// Currently guaranteed to contain at most 1 string
    help: Vec<String>,
    /// Automatically applicable fixes
    ///
    /// Currently unused and guaranteed to be empty
    fixes: Vec<QuickFix>,
}

#[derive(Serialize)]
pub(crate) struct Timings {
    /// time of all performed compilation steps in microseconds
    all: u128,
}

impl Timings {
    pub(crate) fn from_micros(micros: u128) -> Self {
        Timings { all: micros }
    }
}

#[derive(Serialize)]
pub(crate) struct Span {
    /// Start byte offset, counting from zero, assuming UTF-8 encoding.
    ///
    /// Guaranteed to be non-negative.
    start: usize,
    /// End byte offset, non-inclusive, counting from zero, assuming UTF-8
    /// encoding.
    ///
    /// Guaranteed to be at least `start`.
    end: usize,
    /// Additional details only relevant to this specific span
    ///
    /// Currently unused, guaranteed to be absent
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(value: std::ops::Range<usize>) -> Self {
        Span { start: value.start, end: value.end, label: None }
    }
}

#[derive(Serialize)]
pub(crate) struct QuickFix {
    /// Short description what this quick fix does
    description: String,
    /// List of changes to fix this diagnostic.
    ///
    /// Guaranteed to be in source order and non-overlapping (e.g. `1-4`,
    /// `7-12`, `14-15`, `16-16`)
    replacements: Vec<Replacement>,
}

#[derive(Serialize)]
pub(crate) struct Replacement {
    /// Start byte offset, counting from zero, assuming UTF-8 encoding.
    ///
    /// Guaranteed to be non-negative.
    start: usize,
    /// End byte offset, non-inclusive, counting from zero, assuming UTF-8
    /// encoding
    ///
    /// Guaranteed to be at least `start`.
    end: usize,
    /// Text to replace this part of code with
    insert: String,
}

impl From<pomsky::error::Diagnostic> for Diagnostic {
    fn from(value: pomsky::error::Diagnostic) -> Self {
        Diagnostic {
            severity: value.severity.into(),
            kind: value.kind.into(),
            spans: value.span.range().into_iter().map(From::from).collect(),
            description: value.msg,
            help: value.help.into_iter().collect(),
            fixes: vec![],
        }
    }
}
