//! # pomsky-syntax
//!
//! Crate for parsing [pomsky expressions](https://pomsky-lang.org).
//!
//! ## Usage
//!
//! ```
//! let (result, warnings) = pomsky_syntax::parse("let x = 'test'; x*", 256);
//! assert!(result.is_some());
//! assert!(warnings.is_empty());
//! ```

mod error;
mod lexer;
mod parse;
mod span;
mod util;
mod warning;

#[cfg(feature = "dbg")]
mod pretty_print;

pub mod diagnose;
pub mod exprs;

pub use parse::parse;
pub use span::Span;

#[cfg(feature = "suggestions")]
pub use util::find_suggestion;

#[cfg(feature = "dbg")]
use pretty_print::PrettyPrinter;

pub use exprs::char_class::list_shorthands;
