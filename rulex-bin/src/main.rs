use std::{
    io::{self, Read},
    path::PathBuf,
};

use atty::Stream;
use clap::{ArgEnum, Parser};
use rulex::{
    error::Diagnostic,
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

/// Compile a rulex expression to a regex
#[derive(Parser, Debug)]
#[clap(name = "rulex")]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rulex expression to compile
    input: Option<String>,
    /// File containing the rulex expression to compile
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    path: Option<PathBuf>,

    /// Show debug information
    #[clap(short, long)]
    debug: bool,

    /// Regex flavor
    #[clap(long, short, arg_enum, ignore_case(true))]
    flavor: Option<Flavor>,
}

/// Regex flavor
#[derive(Clone, Debug, ArgEnum)]
#[clap(rename_all = "lower")]
enum Flavor {
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

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
enum MyError {
    #[diagnostic(code(error::io))]
    #[error("{}\nFile: {}", .error, .path.display())]
    Io { error: io::Error, path: PathBuf },

    #[error("{}", .0)]
    #[diagnostic(code(error::other))]
    Other(String),
}

pub fn main() -> miette::Result<()> {
    let args = Args::parse();

    match (args.input, args.path) {
        (Some(input), None) => compile(&input, args.debug, args.flavor)?,
        (None, Some(path)) => match std::fs::read_to_string(&path) {
            Ok(input) => compile(&input, args.debug, args.flavor)?,
            Err(error) => return Err(MyError::Io { error, path }.into()),
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input, args.debug, args.flavor)?,
                Err(e) => return Err(MyError::Other(format!("error parsing stdin: {e}")).into()),
            }
        }
        (Some(_), Some(_)) => {
            return Err(MyError::Other("error: Can't provide an input and a path".into()).into())
        }
        (None, None) => return Err(MyError::Other("error: No input provided".into()).into()),
    }
    Ok(())
}

fn compile(input: &str, debug: bool, flavor: Option<Flavor>) -> miette::Result<()> {
    let parsed = Rulex::parse(input, Default::default())
        .map_err(|e| Diagnostic::from_parse_error(e, input))?;

    if debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    let options = CompileOptions {
        flavor: flavor.unwrap_or(Flavor::Pcre).into(),
        ..Default::default()
    };
    let compiled = parsed
        .compile(options)
        .map_err(|e| Diagnostic::from_compile_error(e, input))?;

    println!("{compiled}");
    Ok(())
}
