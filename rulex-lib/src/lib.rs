//! # rulex
//!
//! To learn about the _rulex language_, please read [the book](https://rulex-rs.github.io/rulex).
//!
//! The _rulex macro_ can be [found here](https://docs.rs/rulex-macro/latest/rulex_macro/).
//!
//! ## Usage
//!
//! This library can parse a rulex expression and generate a regex string:
//!
//! ```
//! use rulex::Rulex;
//! use rulex::options::{CompileOptions, RegexFlavor};
//!
//! let options = CompileOptions { flavor: RegexFlavor::Java };
//! let regex: String = match Rulex::parse_and_compile("'test'", Default::default(), options) {
//!     Ok(regex) => regex,
//!     Err(_) => {
//!         eprintln!("The input is not a valid rulex");
//!         return;
//!     }
//! };
//! ```
//!
//! You can get fancy error messages with [miette](https://docs.rs/miette/latest/miette/)
//! by enabling the `diagnostics` feature:
//!
//! ```
//! use rulex::Rulex;
//! use rulex::options::{CompileOptions, RegexFlavor};
//! use rulex::error::Diagnostic;
//!
//! pub fn compile(input: &str) -> miette::Result<String> {
//!     let options = CompileOptions { flavor: RegexFlavor::Java };
//!     let compiled: String = Rulex::parse_and_compile(input, Default::default(), options)
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

pub mod error;
pub mod features;
pub mod options;

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
mod var;

/// A parsed rulex expression, which might contain more sub-expressions.
#[derive(Clone)]
pub struct Rulex<'i>(Rule<'i>);

impl<'i> Rulex<'i> {
    /// Parse a `Rulex` without generating code.
    ///
    /// The parsed `Rulex` can be displayed with `Debug` if the `dbg` feature is
    /// enabled.
    pub fn parse(input: &'i str, options: ParseOptions) -> Result<Self, ParseError> {
        let rule = parse::parse(input, 256)?;
        rule.validate(&options)?;
        Ok(Rulex(rule))
    }

    /// Compile a `Rulex` that has been parsed, to a regex
    pub fn compile(&self, options: CompileOptions) -> Result<String, CompileError> {
        let mut used_names = HashMap::new();
        let mut groups_count = 0;
        self.0.get_capturing_groups(&mut groups_count, &mut used_names, false)?;

        let empty_span = Span::new(0, 0);

        let start = Rule::Boundary(Boundary::new(BoundaryKind::Start, empty_span));
        let end = Rule::Boundary(Boundary::new(BoundaryKind::End, empty_span));
        let grapheme = Rule::Grapheme(Grapheme { span: empty_span });
        let codepoint = Rule::CharClass(CharClass::new(CharGroup::CodePoint, empty_span));

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

    /// Parse a string to a `Rulex` and compile it to a regex.
    pub fn parse_and_compile(
        input: &'i str,
        parse_options: ParseOptions,
        compile_options: CompileOptions,
    ) -> Result<String, CompileError> {
        let parsed = Self::parse(input, parse_options)?;
        parsed.compile(compile_options)
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rulex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
