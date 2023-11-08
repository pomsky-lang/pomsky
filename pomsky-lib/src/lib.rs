//! # Pomsky
//!
//! To learn about the _pomsky language_, please read [the book][book].
//!
//! The _pomsky macro_ can be [found here][macro].
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
//! let regex = match Expr::parse_and_compile("'test'", options) {
//!     (Some(regex), _warnings, _tests) => regex,
//!     (None, diagnostics, _tests) => {
//!         eprintln!("The input is not a valid pomsky expression");
//!         return;
//!     }
//! };
//! ```
//!
//! You can get fancy error messages with [miette] by enabling the `diagnostics`
//! feature:
//!
//! ```
//! use pomsky::Expr;
//! use pomsky::options::{CompileOptions, RegexFlavor};
//! use pomsky::diagnose::Diagnostic;
//!
//! pub fn compile(input: &str) -> miette::Result<String> {
//!     let options = CompileOptions { flavor: RegexFlavor::Java, ..Default::default() };
//!     let compiled = match Expr::parse_and_compile(input, options) {
//!         (Some(regex), _warnings, _tests) => regex,
//!         (None, diagnostics, _tests) => {
//!             for diagnostic in diagnostics {
//!                 eprintln!("{diagnostic}");
//!             }
//!             miette::bail!("Failed to compile pomsky expression");
//!         }
//!     };
//!     Ok(compiled)
//! }
//! ```
//!
//! [book]: https://pomsky-lang.org/docs/
//! [macro]: https://docs.rs/pomsky-macro/latest/pomsky_macro/
//! [miette]: https://docs.rs/miette/latest/miette/

#![warn(missing_docs)]

#[macro_use]
mod defer;

pub mod diagnose;
pub mod error;
pub mod features;
pub mod options;

mod capturing_groups;
mod compile;
mod exprs;
mod regex;
mod validation;
mod visitor;

/// Re-exports syntax node types related to tests
pub mod test {
    pub use pomsky_syntax::exprs::test::*;
}

pub use exprs::Expr;
pub use pomsky_syntax::{
    diagnose::{ParseError, ParseWarning as Warning},
    Span,
};

pub use pomsky_syntax::list_shorthands;
