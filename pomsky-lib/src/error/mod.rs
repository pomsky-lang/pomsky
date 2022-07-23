//! Contains different kinds of errors emitted by Pomsky.

pub(crate) use compile_error::CompileErrorKind;
pub(crate) use parse_error::{
    CharClassError, CharStringError, CodePointError, NumberError, ParseErrorKind, RepetitionError,
    UnsupportedError,
};

pub use compile_error::{CompileError, Feature};
pub use diagnostics::{Diagnostic, Severity};
pub use parse_error::ParseError;

mod compile_error;
mod diagnostics;
mod parse_error;
