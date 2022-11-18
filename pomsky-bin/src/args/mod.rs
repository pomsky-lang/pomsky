use pomsky::{features::PomskyFeatures, options::RegexFlavor};

pub(crate) use errors::ParseArgsError;
pub(crate) use help::print_short_usage_and_help_err;
pub(crate) use input::Input;
pub(crate) use warnings::DiagnosticSet;

use self::parse::ArgsInner;

mod errors;
mod features;
mod flavors;
mod help;
mod input;
mod parse;
mod warnings;

/// Compile a Pomsky expression to a regex
#[derive(PartialEq)]
pub(crate) struct Args {
    /// Pomsky expression to compile
    pub(crate) input: Input,
    /// Show debug information
    pub(crate) debug: bool,
    /// Regex flavor
    pub(crate) flavor: Option<RegexFlavor>,
    /// Does not print a new-line at the end of the compiled regular expression
    pub(crate) no_new_line: bool,
    /// Set of allowed pomsky features
    pub(crate) allowed_features: PomskyFeatures,
    /// Set of allowed pomsky features
    pub(crate) warnings: DiagnosticSet,
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
    }
}
