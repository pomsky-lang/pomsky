//! Contains different kinds of errors emitted by rulex.

pub(crate) use compile_error::CompileErrorKind;
pub(crate) use parse_error::ParseErrorKind;

pub use compile_error::{CompileError, Feature};
pub use diagnostics::Diagnostic;
pub use parse_error::{
    CharClassError, CharStringError, CodePointError, NumberError, ParseError, UnsupportedError,
};

mod compile_error;
mod diagnostics;
mod parse_error;
