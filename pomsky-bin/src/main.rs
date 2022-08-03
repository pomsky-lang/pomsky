use std::{
    io::{self, Read, Write},
    process::exit,
};

use atty::Stream;
use clap::Parser as _;
use owo_colors::OwoColorize;
use pomsky::{
    error::{Diagnostic, ParseError, Severity},
    options::{CompileOptions, ParseOptions},
    warning::Warning,
    Expr,
};

mod parse_args;

use parse_args::{Args, Flavor};

pub fn main() {
    let args = Args::parse();

    match (&args.input, &args.path) {
        (Some(input), None) => compile(input, &args),
        (None, Some(path)) => match std::fs::read_to_string(&path) {
            Ok(input) => compile(&input, &args),
            Err(error) => {
                print_diagnostic(&Diagnostic::ad_hoc(
                    Severity::Error,
                    None,
                    error.to_string(),
                    None,
                ));
                exit(2);
            }
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input, &args),
                Err(e) => {
                    print_diagnostic(&Diagnostic::ad_hoc(
                        Severity::Error,
                        None,
                        format!("Could not parse stdin: {e}"),
                        None,
                    ));
                    exit(3);
                }
            }
        }
        (Some(_), Some(_)) => {
            print_diagnostic(&Diagnostic::ad_hoc(
                Severity::Error,
                None,
                "You can only provide an input or a path, but not both".into(),
                None,
            ));
            exit(4);
        }
        (None, None) => {
            print_diagnostic(&Diagnostic::ad_hoc(
                Severity::Error,
                None,
                "No input provided".into(),
                None,
            ));
            exit(5);
        }
    }
}

fn compile(input: &str, args: &Args) {
    let parse_options = ParseOptions { max_range_size: 12, ..ParseOptions::default() };
    let (parsed, warnings) = match Expr::parse(input, parse_options) {
        Ok(res) => res,
        Err(err) => {
            print_parse_error(err, input);
            exit(1);
        }
    };

    if args.debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    print_warnings(warnings, input);

    let compile_options =
        CompileOptions { flavor: (*args.flavor.as_ref().unwrap_or(&Flavor::Pcre)).into() };
    let compiled = match parsed
        .compile(compile_options)
        .map_err(|err| Diagnostic::from_compile_error(err, input))
    {
        Ok(res) => res,
        Err(err) => {
            print_diagnostic(&err);
            std::process::exit(1);
        }
    };

    if args.no_new_line {
        print!("{compiled}");
        io::stdout().flush().unwrap();
    } else {
        println!("{compiled}");
    }
}

fn print_parse_error(error: ParseError, input: &str) {
    let diagnostics = Diagnostic::from_parse_errors(error, input);

    for diagnostic in diagnostics.iter().take(8) {
        print_diagnostic(diagnostic);
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
        print_diagnostic(&Diagnostic::from_warning(warning, input));
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

fn print_diagnostic(diagnostic: &Diagnostic) {
    match diagnostic.severity {
        Severity::Error => {
            eprintln!("{}: {}", "error".bright_red().bold(), diagnostic.default_display())
        }
        Severity::Warning => {
            eprintln!("{}: {}", "warning".yellow().bold(), diagnostic.default_display())
        }
    }
}
