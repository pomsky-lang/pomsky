//! Module for parsing a pomsky expression

mod diagnostics;
mod micro_regex;
mod parser;
mod token;
mod tokenize;

pub use parser::parse;
pub use token::{LexErrorMsg, Token};
