//! # Pomsky
//!
//! To learn about the _pomsky language_, please read [the book](https://pomsky-lang.org/docs/).
//!
//! The _pomsky macro_ can be [found here](https://docs.rs/pomsky-macro/latest/pomsky_macro/).
//!
//! ## Usage
//!
//! This library can parse a pomsky expression and generate a regex string:
//!
//! ```
//! use pomsky::Expr;
//! use pomsky::options::{CompileOptions, RegexFlavor};
//!
//! let options = CompileOptions { flavor: RegexFlavor::Java, ..Default::default() };
//! let (regex, _warnings) = match Expr::parse_and_compile("'test'", options) {
//!     Ok(regex) => regex,
//!     Err(_) => {
//!         eprintln!("The input is not a valid pomsky expression");
//!         return;
//!     }
//! };
//! ```
//!
//! You can get fancy error messages with [miette](https://docs.rs/miette/latest/miette/)
//! by enabling the `diagnostics` feature:
//!
//! ```
//! use pomsky::Expr;
//! use pomsky::options::{CompileOptions, RegexFlavor};
//! use pomsky::error::Diagnostic;
//!
//! pub fn compile(input: &str) -> miette::Result<String> {
//!     let options = CompileOptions { flavor: RegexFlavor::Java, ..Default::default() };
//!     let (compiled, _warnings) = Expr::parse_and_compile(input, options)
//!         .map_err(|e| e.diagnostic(input))?;
//!     Ok(compiled)
//! }
//! ```

#![warn(missing_docs)]

pub mod error;
pub mod features;
pub mod options;

mod compile;
mod exprs;
mod regex;

pub use exprs::Expr;
pub use pomsky_syntax::error::ParseError;
pub use pomsky_syntax::warning::ParseWarning as Warning;
