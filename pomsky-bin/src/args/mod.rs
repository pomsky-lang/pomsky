use pomsky::{features::PomskyFeatures, options::RegexFlavor};

pub(crate) use errors::ParseArgsError;
pub(crate) use help::print_short_usage_and_help_err;
pub(crate) use input::Input;
pub(crate) use test::TestSettings;
pub(crate) use warnings::DiagnosticSet;

use self::parse::{ArgsInner, ListKind};

mod errors;
mod features;
mod flavors;
mod help;
mod input;
mod parse;
mod test;
mod warnings;

/// Compile a Pomsky expression to a regex
#[derive(PartialEq)]
pub(crate) struct Args {
    /// Pomsky expression to compile
    pub(crate) input: Input,
    /// Show debug information
    pub(crate) debug: bool,
    /// Whether output should be provided as JSON
    pub(crate) json: bool,
    /// Regex flavor
    pub(crate) flavor: Option<RegexFlavor>,
    /// Does not print a new-line at the end of the compiled regular expression
    pub(crate) no_new_line: bool,
    /// Set of allowed pomsky features
    pub(crate) allowed_features: PomskyFeatures,
    /// Set of warnings that should be emitted
    pub(crate) warnings: DiagnosticSet,
    /// Whether to execute tests after compilation
    pub(crate) test: Option<TestSettings>,
}

pub(super) fn parse_args() -> Result<Args, ParseArgsError> {
    match parse::parse_args_inner(lexopt::Parser::from_env())? {
        ArgsInner::Args(args) => Ok(args),
        ArgsInner::HelpLong => {
            help::print_long_help();
            std::process::exit(0)
        }
        ArgsInner::HelpShort => {
            help::print_help();
            std::process::exit(0)
        }
        ArgsInner::Version => {
            help::print_version();
            std::process::exit(0)
        }
        ArgsInner::List(ListKind::Shorthands) => {
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
