use std::path::PathBuf;

use clap::{ArgEnum, Parser};
use pomsky::options::RegexFlavor;

/// Compile a Pomsky expression to a regex
#[derive(Parser, Debug)]
#[clap(name = "pomsky")]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Pomsky expression to compile
    pub(crate) input: Option<String>,
    /// File containing the pomsky expression to compile
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    pub(crate) path: Option<PathBuf>,

    /// Show debug information
    #[clap(short, long)]
    pub(crate) debug: bool,

    /// Regex flavor
    #[clap(long, short, arg_enum, ignore_case(true))]
    pub(crate) flavor: Option<Flavor>,

    /// Does not print a new-line at the end of the compiled regular expression
    #[clap(long, short)]
    pub(crate) no_new_line: bool,
}

/// Pomsky flavor
#[derive(Clone, Copy, Debug, ArgEnum)]
#[clap(rename_all = "lower")]
pub(crate) enum Flavor {
    Pcre,
    Python,
    Java,
    #[clap(alias = "js")]
    JavaScript,
    #[clap(alias = ".net")]
    DotNet,
    Ruby,
    Rust,
}

impl From<Flavor> for RegexFlavor {
    fn from(f: Flavor) -> Self {
        match f {
            Flavor::Pcre => RegexFlavor::Pcre,
            Flavor::Python => RegexFlavor::Python,
            Flavor::Java => RegexFlavor::Java,
            Flavor::JavaScript => RegexFlavor::JavaScript,
            Flavor::DotNet => RegexFlavor::DotNet,
            Flavor::Ruby => RegexFlavor::Ruby,
            Flavor::Rust => RegexFlavor::Rust,
        }
    }
}
