use std::{
    io::{self, Read, Write},
    path::PathBuf,
};

use atty::Stream;
use clap::Parser as _;
use owo_colors::OwoColorize;
use pomsky::{
    error::{Diagnostic, ParseError},
    options::{CompileOptions, ParseOptions},
    warning::Warning,
    Expr,
};

mod parse_args;

use parse_args::{Args, Flavor};

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

    match (&args.input, &args.path) {
        (Some(input), None) => compile(input, &args)?,
        (None, Some(path)) => match std::fs::read_to_string(&path) {
            Ok(input) => compile(&input, &args)?,
            Err(error) => return Err(MyError::Io { error, path: path.clone() }.into()),
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input, &args)?,
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

fn compile(input: &str, args: &Args) -> miette::Result<()> {
    let parse_options = ParseOptions { max_range_size: 12, ..ParseOptions::default() };
    let (parsed, warnings) = match Expr::parse(input, parse_options) {
        Ok(res) => res,
        Err(err) => {
            print_parse_error(err, input);
            std::process::exit(1);
        }
    };

    if args.debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    print_warnings(warnings, input);

    let compile_options =
        CompileOptions { flavor: (*args.flavor.as_ref().unwrap_or(&Flavor::Pcre)).into() };
    let compiled = parsed
        .compile(compile_options)
        .map_err(|err| Diagnostic::from_compile_error(err, input))?;

    if args.no_new_line {
        print!("{compiled}");
        io::stdout().flush().unwrap();
    } else {
        println!("{compiled}");
    }
    Ok(())
}

fn print_parse_error(error: ParseError, input: &str) {
    let diagnostics = Diagnostic::from_parse_errors(error, input);

    for diagnostic in diagnostics.iter().take(8) {
        eprintln!("{}: {}", "error".bright_red().bold(), diagnostic.default_display());
    }

    let len = diagnostics.len();

    if len > 8 {
        eprintln!("{}: some errors were omitted", "note".cyan().bold());
    }

    eprintln!(
        "{}: could not compile expression due to {}",
        "error".bright_red().bold(),
        if len > 1 { format!("{len} previous errors") } else { "previous error".into() }
    );
}

fn print_warnings(warnings: Vec<Warning>, input: &str) {
    let len = warnings.len();

    for warning in warnings.into_iter().take(8) {
        eprintln!(
            "{}: {}",
            "warning".yellow().bold(),
            Diagnostic::from_warning(warning, input).default_display()
        );
    }

    if len > 8 {
        eprintln!("{}: some warnings were omitted", "note".cyan().bold());
    }

    if len > 0 {
        eprintln!(
            "{}: pomsky generated {len} {}",
            "warning".yellow().bold(),
            if len > 1 { "warnings" } else { "warning" },
        );
    }
}
