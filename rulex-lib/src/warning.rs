//! Provides warnings that are shown to the user (in addition to the output)

use std::fmt;

use crate::span::Span;

/// A warning.
#[derive(Debug, Clone, Copy)]
pub struct Warning {
    /// The kind of warning
    pub kind: WarningKind,
    /// The span pointing to the source of the warning
    pub span: Span,
}

/// A warning without a span pointing to the source of the warning
#[derive(Debug, Clone, Copy)]
pub enum WarningKind {
    /// A deprecation warning
    Deprecation(DeprecationWarning),
}

impl WarningKind {
    pub(crate) fn at(self, span: Span) -> Warning {
        Warning { kind: self, span }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            WarningKind::Deprecation(d) => {
                if let Some(std::ops::Range { start, end }) = self.span.range() {
                    write!(f, "{d}\n  at {}..{}", start, end)
                } else {
                    write!(f, "{d}")
                }
            }
        }
    }
}

impl fmt::Display for WarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let WarningKind::Deprecation(c) = self;
        c.fmt(f)
    }
}

/// A deprecation warning: Indicates that something shouldn't be used anymore
#[derive(Debug, Clone, Copy)]
pub enum DeprecationWarning {
    /// The `<%` start literal
    StartLiteral,
    /// The `<%` start literal
    EndLiteral,
}

impl fmt::Display for DeprecationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeprecationWarning::StartLiteral => {
                f.write_str("The `<%` literal is deprecated. Use `Start` instead.")
            }
            DeprecationWarning::EndLiteral => {
                f.write_str("The `%>` literal is deprecated. Use `End` instead.")
            }
        }
    }
}
