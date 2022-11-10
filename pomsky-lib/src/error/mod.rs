//! Contains different kinds of errors emitted by Pomsky.

pub(crate) use compile_error::{CompileErrorKind, UnsupportedError};

pub use compile_error::{CompileError, Feature};
pub use diagnostic_kind::DiagnosticKind;
pub use diagnostics::{Diagnostic, Severity};
pub use pomsky_syntax::error::*;

mod compile_error;
mod diagnostic_kind;
mod diagnostics;
