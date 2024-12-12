use std::path::PathBuf;

use pomsky::{features::PomskyFeatures, options::RegexFlavor};

pub(crate) use self::engines::RegexEngine;
pub(crate) use errors::ParseArgsError;
pub(crate) use help::print_usage_and_help;
pub(crate) use input::Input;
pub(crate) use warnings::DiagnosticSet;

use crate::format::Logger;

use self::parse::{ListKind, Parsed};

mod engines;
mod errors;
mod features;
mod flavors;
mod help;
mod input;
mod parse;
mod warnings;

#[derive(PartialEq)]
pub(crate) struct GlobalOptions {
    /// Show debug information
    pub(crate) debug: bool,
    /// Whether output should be provided as JSON
    pub(crate) json: bool,
    /// Regex flavor
    pub(crate) flavor: Option<RegexFlavor>,
    /// Set of allowed pomsky features
    pub(crate) allowed_features: PomskyFeatures,
    /// Set of warnings that should be emitted
    pub(crate) warnings: DiagnosticSet,
}

#[derive(PartialEq)]
pub(crate) enum Subcommand {
    Compile(CompileOptions),
    Test(TestOptions),
}

#[derive(PartialEq)]
pub(crate) struct CompileOptions {
    /// Pomsky expression to compile
    pub(crate) input: Input,
    /// Does not print a new-line at the end of the compiled regular expression
    pub(crate) no_new_line: bool,
    /// Whether to execute tests after compilation
    pub(crate) test: Option<RegexEngine>,
    /// Whether to output the compiled expression. If false, a test report is printed instead
    pub(crate) in_test_suite: bool,
}

/// Test Pomsky expressions
#[derive(PartialEq)]
pub(crate) struct TestOptions {
    /// Path to Pomsky expression(s) to test
    pub(crate) path: PathBuf,
    /// Whether to execute tests after compilation
    pub(crate) engine: Option<RegexEngine>,
    /// Whether to pass even if no expressions were compiled
    pub(crate) pass_with_no_tests: bool,
}

pub(super) fn parse_args(logger: &Logger) -> Result<(Subcommand, GlobalOptions), ParseArgsError> {
    match parse::parse_args_inner(logger, lexopt::Parser::from_env())? {
        Parsed::Options(subcommand, opts) => Ok((subcommand, opts)),
        Parsed::Help(help) => {
            match help {
                parse::Help::Short => help::print_short_help(),
                parse::Help::Long => help::print_long_help(),
                parse::Help::TestShort => help::print_test_short_help(),
                parse::Help::TestLong => help::print_test_long_help(),
            }
            std::process::exit(0)
        }
        Parsed::Version => {
            help::print_version();
            std::process::exit(0)
        }
        Parsed::List(ListKind::Shorthands) => {
            let s = pomsky::list_shorthands().fold(String::new(), |mut acc, (name, group_name)| {
                use std::fmt::Write;
                let _ = writeln!(acc, "{name:<50} {}", group_name.kind());
                acc
            });
            println!("{s}");
            std::process::exit(0)
        }
    }
}
