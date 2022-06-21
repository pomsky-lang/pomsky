use std::{
    fmt,
    io::{self, Read, Write},
    path::PathBuf,
};

use atty::Stream;
use clap::{ArgEnum, Parser};
use miette::ReportHandler;
use rulex::{
    error::Diagnostic,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    warning::Warning,
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

    /// Does not print a new-line at the end of the compiled regular expression
    #[clap(long, short)]
    no_new_line: bool,
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
        (Some(input), None) => compile(&input, args.debug, args.flavor, args.no_new_line)?,
        (None, Some(path)) => match std::fs::read_to_string(&path) {
            Ok(input) => compile(&input, args.debug, args.flavor, args.no_new_line)?,
            Err(error) => return Err(MyError::Io { error, path }.into()),
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input, args.debug, args.flavor, args.no_new_line)?,
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

fn compile(
    input: &str,
    debug: bool,
    flavor: Option<Flavor>,
    no_new_line: bool,
) -> miette::Result<()> {
    let parse_options = ParseOptions { max_range_size: 12, ..ParseOptions::default() };
    let (parsed, warnings) = Rulex::parse(input, parse_options)
        .map_err(|err| Diagnostic::from_parse_error(err, input))?;

    if debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    #[derive(Debug)]
    struct WarningPrinter<'a>(Warning, &'a str);

    impl fmt::Display for WarningPrinter<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let diag = Diagnostic::from_warning(self.0, self.1);
            miette::MietteHandler::default().debug(&diag, f)
        }
    }

    for warning in warnings {
        eprintln!("{}", WarningPrinter(warning, input));
    }

    let compile_options = CompileOptions { flavor: flavor.unwrap_or(Flavor::Pcre).into() };
    let compiled = parsed
        .compile(compile_options)
        .map_err(|err| Diagnostic::from_compile_error(err, input))?;

    if no_new_line {
        print!("{compiled}");
        io::stdout().flush().unwrap();
    } else {
        println!("{compiled}");
    }
    Ok(())
}
