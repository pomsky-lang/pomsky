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
//! let options = CompileOptions { flavor: RegexFlavor::Java };
//! let (regex, _warnings) = match Expr::parse_and_compile("'test'", Default::default(), options) {
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
//!     let options = CompileOptions { flavor: RegexFlavor::Java };
//!     let (compiled, _warnings) = Expr::parse_and_compile(input, Default::default(), options)
//!         .map_err(|e| e.diagnostic(input))?;
//!     Ok(compiled)
//! }
//! ```

#![warn(missing_docs)]

use std::collections::HashMap;

use boundary::{Boundary, BoundaryKind};
use char_class::{CharClass, CharGroup};
use compile::CompileState;
use error::{CompileError, ParseError};
use grapheme::Grapheme;
use options::{CompileOptions, ParseOptions};
use repetition::RegexQuantifier;
use rule::Rule;
use span::Span;
use warning::Warning;

pub mod error;
pub mod features;
pub mod options;
pub mod warning;

mod alternation;
mod boundary;
mod char_class;
mod compile;
mod grapheme;
mod group;
mod literal;
mod lookaround;
mod parse;
mod range;
mod reference;
mod regex;
mod repetition;
mod rule;
mod span;
mod stmt;
mod util;
mod var;

/// A parsed pomsky expression, which might contain more sub-expressions.
#[derive(Clone)]
pub struct Expr<'i>(Rule<'i>);

impl<'i> Expr<'i> {
    /// Parse a `Expr` without generating code.
    ///
    /// The parsed `Expr` can be displayed with `Debug` if the `dbg` feature is
    /// enabled.
    pub fn parse(
        input: &'i str,
        options: ParseOptions,
    ) -> Result<(Self, Vec<Warning>), ParseError> {
        let (rule, warning) = parse::parse(input, 256)?;
        rule.validate(&options)?;
        Ok((Expr(rule), warning))
    }

    /// Compile a `Expr` that has been parsed, to a regex
    pub fn compile(&self, options: CompileOptions) -> Result<String, CompileError> {
        let mut used_names = HashMap::new();
        let mut groups_count = 0;
        self.0.get_capturing_groups(&mut groups_count, &mut used_names, false)?;

        let no_span = Span::empty();

        let start = Rule::Boundary(Boundary::new(BoundaryKind::Start, no_span));
        let end = Rule::Boundary(Boundary::new(BoundaryKind::End, no_span));
        let grapheme = Rule::Grapheme(Grapheme);
        let codepoint = Rule::CharClass(CharClass::new(CharGroup::CodePoint, no_span));

        let builtins = vec![
            ("Start", &start),
            ("End", &end),
            ("Grapheme", &grapheme),
            ("G", &grapheme),
            ("Codepoint", &codepoint),
            ("C", &codepoint),
        ];

        let mut state = CompileState {
            next_idx: 1,
            used_names,
            groups_count,
            default_quantifier: RegexQuantifier::Greedy,
            variables: builtins,
            current_vars: Default::default(),
        };
        let compiled = self.0.comp(options, &mut state)?;

        let mut buf = String::new();
        compiled.codegen(&mut buf, options.flavor);
        Ok(buf)
    }

    /// Parse a string to a `Expr` and compile it to a regex.
    pub fn parse_and_compile(
        input: &'i str,
        parse_options: ParseOptions,
        compile_options: CompileOptions,
    ) -> Result<(String, Vec<Warning>), CompileError> {
        let (parsed, warnings) = Self::parse(input, parse_options)?;
        let compiled = parsed.compile(compile_options)?;
        Ok((compiled, warnings))
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Expr<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
