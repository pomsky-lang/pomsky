//! Crate containing diagnostics, i.e. errors and warnings

pub(crate) use compile_error::{CompileError, CompileErrorKind, UnsupportedError};
pub(crate) use warning::{CompatWarning, CompileWarningKind};

pub use diagnostic_code::DiagnosticCode;
pub use diagnostic_kind::DiagnosticKind;
pub use diagnostics::{Diagnostic, Severity};
pub use feature::Feature;

mod compile_error;
mod diagnostic_code;
mod diagnostic_kind;
mod diagnostics;
mod feature;
mod help;
mod warning;
