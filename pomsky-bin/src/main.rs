use std::{
    io::{self, Write as _},
    process::exit,
};

use atty::Stream;
use owo_colors::{OwoColorize, Style};
use pomsky::{
    error::{Diagnostic, ParseError, Severity},
    options::{CompileOptions, RegexFlavor},
    Expr, Warning,
};

#[macro_use]
mod colors;
mod args;

use args::{Args, Input, ParseArgsError};

pub fn main() {
    let args = match args::parse_args() {
        Ok(args) => args,
        Err(error) => {
            let msg = match error {
                ParseArgsError::Lexopt(error) => error.to_string(),
                ParseArgsError::StdinUtf8(e) => format!("Could not parse stdin: {e}"),
                ParseArgsError::Other(msg) => msg,
            };
            print_diagnostic(&Diagnostic::ad_hoc(Severity::Error, None, msg, None));
            eprintln!("{}", args::get_short_usage_and_help(Stream::Stderr));
            exit(2)
        }
    };

    match &args.input {
        Input::Value(input) => compile(input, &args),
        Input::File(path) => match std::fs::read_to_string(&path) {
            Ok(input) => compile(&input, &args),
            Err(error) => {
                let msg = error.to_string();
                print_diagnostic(&Diagnostic::ad_hoc(Severity::Error, None, msg, None));
                exit(3);
            }
        },
    }
}

fn compile(input: &str, args: &Args) {
    let options = CompileOptions {
        flavor: args.flavor.unwrap_or(RegexFlavor::Pcre),
        max_range_size: 12,
        allowed_features: args.allowed_features,
    };

    let (parsed, warnings) = match Expr::parse(input) {
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

    let compiled =
        match parsed.compile(options).map_err(|err| Diagnostic::from_compile_error(err, input)) {
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
        eprintln!("{}: some errors were omitted", cyan_bold!(Stderr, "note"));
    }

    eprintln!(
        "{}: could not compile expression due to {}",
        red_bold!(Stderr, "error"),
        if len > 1 { format!("{len} previous errors") } else { "previous error".into() }
    );
}

fn print_warnings(warnings: Vec<Warning>, input: &str) {
    let len = warnings.len();

    for warning in warnings.into_iter().take(8) {
        print_diagnostic(&Diagnostic::from_warning(warning, input));
    }

    if len > 8 {
        eprintln!("{}: some warnings were omitted", cyan_bold!(Stderr, "note"));
    }

    if len > 0 {
        eprintln!(
            "{}: pomsky generated {len} {}",
            yellow_bold!(Stderr, "warning"),
            if len > 1 { "warnings" } else { "warning" },
        );
    }
}

fn print_diagnostic(diagnostic: &Diagnostic) {
    match diagnostic.severity {
        Severity::Error => {
            eprintln!("{}: {}", red_bold!(Stderr, "error"), diagnostic.default_display())
        }
        Severity::Warning => {
            eprintln!("{}: {}", yellow_bold!(Stderr, "warning"), diagnostic.default_display())
        }
    }
}
