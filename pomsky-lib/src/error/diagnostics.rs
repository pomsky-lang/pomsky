use pomsky_syntax::{
    error::{DeprecationError, ParseError, ParseErrorKind, RepetitionError},
    warning::ParseWarning,
    Span,
};

use super::{compile_error::CompileErrorKind, CharClassError, CharStringError, CompileError};

#[cfg_attr(feature = "miette", derive(Debug))]
#[non_exhaustive]
/// A struct containing detailed information about an error, which can be
/// displayed beautifully with [miette](https://docs.rs/miette/latest/miette/).
pub struct Diagnostic {
    /// Whether this is an error, a warning or advice
    pub severity: Severity,
    /// The error message
    pub msg: String,
    /// The error code (optional, currently unused)
    pub code: Option<String>,
    /// The source code where the error occurred
    pub source_code: Option<String>,
    /// An (optional) help message explaining how the error could be fixed
    pub help: Option<String>,
    /// The start and end byte positions of the source code where the error
    /// occurred.
    pub span: Span,
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

#[cfg(feature = "miette")]
impl miette::Diagnostic for Diagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.code.as_deref().map(|c| Box::new(c) as Box<dyn std::fmt::Display + 'a>)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help.as_deref().map(|h| Box::new(h) as Box<dyn std::fmt::Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source_code.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            Some(Box::new(
                [miette::LabeledSpan::new(
                    Some(
                        (match self.severity {
                            Severity::Error => "error occurred here",
                            Severity::Warning => "warning originated here",
                        })
                        .into(),
                    ),
                    start,
                    end - start,
                )]
                .into_iter(),
            ))
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
    /// Create a [`Diagnostic`] from a [`ParseError`]
    #[must_use]
    pub fn from_parse_error(error: ParseError, source_code: &str) -> Self {
        let range = error.span.range().unwrap_or(0..source_code.len());
        let slice = &source_code[range.clone()];
        let mut span = Span::from(range);

        let help = match error.kind {
            ParseErrorKind::LexErrorWithMessage(msg) => msg.get_help(slice),
            ParseErrorKind::RangeIsNotIncreasing => {
                let dash_pos = slice.find('-').unwrap();
                let (part1, part2) = slice.split_at(dash_pos);
                let part2 = part2.trim_start_matches('-');
                Some(format!("Switch the numbers: {}-{}", part2.trim(), part1.trim()))
            }
            ParseErrorKind::Dot => Some(
                "Use `Codepoint` to match any code point, or `![n]` to exclude line breaks".into(),
            ),
            #[cfg(feature = "suggestions")]
            ParseErrorKind::CharClass(CharClassError::UnknownNamedClass {
                similar: Some(ref similar),
                ..
            }) => Some(format!("Perhaps you meant `{similar}`")),
            ParseErrorKind::CharClass(CharClassError::DescendingRange(..)) => {
                let dash_pos = slice.find('-').unwrap();
                let (part1, part2) = slice.split_at(dash_pos);
                let part2 = part2.trim_start_matches('-');
                Some(format!("Switch the characters: {}-{}", part2.trim(), part1.trim()))
            }
            ParseErrorKind::CharClass(CharClassError::Empty) => {
                Some("You can use `![s !s]` to match nothing, and `C` to match anything".into())
            }
            ParseErrorKind::CharClass(CharClassError::CaretInGroup) => {
                Some("Use `![...]` to negate a character class".into())
            }
            ParseErrorKind::CharString(CharStringError::TooManyCodePoints)
                if slice.trim_matches(&['"', '\''][..]).chars().all(|c| c.is_ascii_digit()) =>
            {
                Some(
                    "Try a `range` expression instead:\n\
                    https://pomsky-lang.org/docs/language-tour/ranges/"
                        .into(),
                )
            }
            ParseErrorKind::KeywordAfterLet(_) => Some("Use a different variable name".into()),
            ParseErrorKind::UnallowedMultiNot(n) => Some(if n % 2 == 0 {
                "The number of exclamation marks is even, so you can remove all of them".into()
            } else {
                "The number of exclamation marks is odd, so you can remove all of them but one"
                    .into()
            }),
            ParseErrorKind::LetBindingExists => Some("Use a different name".into()),
            ParseErrorKind::Repetition(RepetitionError::QmSuffix) => Some(
                "If you meant to make the repetition lazy, append the `lazy` keyword instead.\n\
                If this is intentional, consider adding parentheses around the inner repetition."
                    .into(),
            ),
            ParseErrorKind::Repetition(RepetitionError::PlusSuffix) => {
                Some("Add parentheses around the inner repetition.".into())
            }
            ParseErrorKind::InvalidEscapeInStringAt(offset) => {
                let span_start = span.range_unchecked().start;
                span = Span::new(span_start + offset - 1, span_start + offset + 1);
                None
            }
            ParseErrorKind::RecursionLimit => Some(
                "Try a less nested expression. It helps to refactor it using variables:\n\
                https://pomsky-lang.org/docs/language-tour/variables/"
                    .into(),
            ),
            ParseErrorKind::Deprecated(DeprecationError::CodepointInSet) => {
                Some("Use `Codepoint` without brackets instead".into())
            }
            ParseErrorKind::Deprecated(DeprecationError::CpInSet) => {
                Some("Use `C` without brackets instead".into())
            }
            _ => None,
        };

        Diagnostic {
            severity: Severity::Error,
            code: None,
            msg: error.kind.to_string(),
            source_code: Some(source_code.into()),
            help,
            span,
        }
    }

    /// Same as [`Diagnostic::from_parse_error`], but returns a `Vec` and
    /// recursively flattens [`ParseErrorKind::Multiple`].
    #[must_use]
    pub fn from_parse_errors(error: ParseError, source_code: &str) -> Vec<Diagnostic> {
        match error.kind {
            ParseErrorKind::Multiple(multiple) => Vec::from(multiple)
                .into_iter()
                .flat_map(|err| Diagnostic::from_parse_errors(err, source_code))
                .collect(),
            _ => vec![Diagnostic::from_parse_error(error, source_code)],
        }
    }

    /// Create a [`Diagnostic`] from a [`CompileError`]
    #[must_use]
    pub fn from_compile_error(
        CompileError { kind, span }: CompileError,
        source_code: &str,
    ) -> Self {
        match kind {
            CompileErrorKind::ParseError(kind) => {
                Diagnostic::from_parse_error(ParseError { kind, span }, source_code)
            }
            #[cfg(feature = "suggestions")]
            CompileErrorKind::UnknownVariable { similar: Some(ref similar), .. }
            | CompileErrorKind::UnknownReferenceName { similar: Some(ref similar), .. } => {
                let range = span.range().unwrap_or(0..source_code.len());
                let span = Span::from(range);

                Diagnostic {
                    severity: Severity::Error,
                    code: None,
                    msg: kind.to_string(),
                    source_code: Some(source_code.into()),
                    help: Some(format!("Perhaps you meant `{similar}`")),
                    span,
                }
            }
            _ => {
                let range = span.range().unwrap_or(0..source_code.len());
                let span = Span::from(range);

                Diagnostic {
                    severity: Severity::Error,
                    code: None,
                    msg: kind.to_string(),
                    source_code: Some(source_code.into()),
                    help: None,
                    span,
                }
            }
        }
    }

    /// Create one or multiple [`Diagnostic`]s from a [`CompileError`]
    #[must_use]
    pub fn from_compile_errors(
        CompileError { kind, span }: CompileError,
        source_code: &str,
    ) -> Vec<Self> {
        if let CompileErrorKind::ParseError(kind) = kind {
            Diagnostic::from_parse_errors(ParseError { kind, span }, source_code)
        } else {
            let range = span.range().unwrap_or(0..source_code.len());
            let span = Span::from(range);

            vec![Diagnostic {
                severity: Severity::Error,
                code: None,
                msg: kind.to_string(),
                source_code: Some(source_code.into()),
                help: None,
                span,
            }]
        }
    }

    /// Create a [`Diagnostic`] from a [`CompileError`]
    #[must_use]
    pub fn from_warning(warning: ParseWarning, source_code: &str) -> Self {
        let range = warning.span.range().unwrap_or(0..source_code.len());
        let span = Span::from(range);

        Diagnostic {
            severity: Severity::Warning,
            code: None,
            msg: warning.kind.to_string(),
            source_code: Some(source_code.into()),
            help: None,
            span,
        }
    }

    /// Create an ad-hoc diagnostic without a source code snippet
    #[must_use]
    pub fn ad_hoc(
        severity: Severity,
        code: Option<String>,
        msg: String,
        help: Option<String>,
    ) -> Self {
        Diagnostic { severity, code, msg, source_code: None, help, span: Span::empty() }
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
