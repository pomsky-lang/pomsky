use std::{io, io::Write as _, process::exit, time::Instant};

use pomsky::{
    diagnose::{Diagnostic, Severity},
    options::{CompileOptions, RegexFlavor},
    Expr,
};

#[macro_use]
mod format;
mod args;
mod result;
#[cfg(feature = "test")]
mod test;

use args::{Args, DiagnosticSet, Input, TestSettings};
use result::CompilationResult;

pub fn main() {
    let args = match args::parse_args() {
        Ok(args) => args,
        Err(error) => {
            print_diagnostic(
                &Diagnostic::ad_hoc(Severity::Error, None, error.to_string(), None),
                None,
            );
            args::print_short_usage_and_help_err();
            exit(2)
        }
    };

    #[cfg(not(feature = "test"))]
    if args.test != TestSettings::None {
        print_diagnostic(
            &Diagnostic::ad_hoc(
                Severity::Error,
                None,
                "Testing is not supported, because this pomsky binary \
                was compiled with the `test` feature disabled!"
                    .into(),
                None,
            ),
            None,
        );
        exit(4);
    }

    if args.json && std::env::var_os("NO_COLOR").is_none() {
        std::env::set_var("NO_COLOR", "1");
    }

    match &args.input {
        Input::Value(input) => compile(input, &args),
        Input::File(path) => match std::fs::read_to_string(path) {
            Ok(input) => compile(&input, &args),
            Err(error) => {
                print_diagnostic(
                    &Diagnostic::ad_hoc(Severity::Error, None, error.to_string(), None),
                    None,
                );
                exit(3);
            }
        },
    }
}

fn compile(input: &str, args: &Args) {
    let start = Instant::now();

    let options = CompileOptions {
        flavor: args.flavor.unwrap_or(RegexFlavor::Pcre),
        max_range_size: 12,
        allowed_features: args.allowed_features,
    };

    let (parsed, warnings) = match Expr::parse(input) {
        (Some(res), warnings) => (res, warnings),
        (None, err) => {
            print_parse_errors(err, Some(input), start.elapsed().as_micros(), 0, args.json);
            exit(1);
        }
    };
    let mut warnings = warnings.collect::<Vec<_>>();

    if args.debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    if !args.json {
        print_warnings(&warnings, args, Some(input));
    }

    #[allow(unused_mut)] // the `mut` is only needed when cfg(feature = "test")
    let (mut test_errors, mut time_test) = (Vec::new(), 0);

    let compiled = match parsed.compile(input, options) {
        (Some(res), compile_warnings) => {
            #[cfg(feature = "test")]
            if args.test == TestSettings::Pcre2 {
                let start = Instant::now();
                test::run_tests(parsed, input, options, &mut test_errors);
                time_test = start.elapsed().as_micros();
            }

            if args.json {
                if test_errors.is_empty() {
                    warnings.extend(compile_warnings);
                } else {
                    CompilationResult::error(start.elapsed().as_micros(), time_test)
                        .with_diagnostics(test_errors, Some(input))
                        .with_diagnostics(
                            warnings.into_iter().filter_map(|w| {
                                if args.warnings.is_enabled(w.kind) {
                                    Some(w)
                                } else {
                                    None
                                }
                            }),
                            Some(input),
                        )
                        .output_json();
                    std::process::exit(1);
                }
            } else {
                for error in &test_errors {
                    print_diagnostic(error, if error.span.is_empty() { None } else { Some(input) });
                }
                if test_errors.is_empty() {
                    print_warnings(&compile_warnings, args, Some(input));
                } else {
                    std::process::exit(1);
                }
            }

            res
        }
        (None, errors) => {
            if args.json {
                CompilationResult::error(start.elapsed().as_micros(), time_test)
                    .with_diagnostics(errors, Some(input))
                    .with_diagnostics(
                        warnings.into_iter().filter_map(|w| {
                            if args.warnings.is_enabled(w.kind) {
                                Some(w)
                            } else {
                                None
                            }
                        }),
                        Some(input),
                    )
                    .output_json();
            } else {
                for err in &errors {
                    print_diagnostic(err, Some(input));
                }
            }
            std::process::exit(1);
        }
    };

    if args.json {
        CompilationResult::success(compiled, start.elapsed().as_micros(), time_test)
            .with_diagnostics(
                warnings.into_iter().filter_map(|w| {
                    if args.warnings.is_enabled(w.kind) {
                        Some(w)
                    } else {
                        None
                    }
                }),
                Some(input),
            )
            .output_json();
    } else if args.no_new_line {
        print!("{compiled}");
        io::stdout().flush().unwrap();
    } else {
        println!("{compiled}");
    }
}

fn print_parse_errors(
    mut diagnostics: impl Iterator<Item = Diagnostic>,
    source_code: Option<&str>,
    time_all: u128,
    time_test: u128,
    json: bool,
) {
    if json {
        CompilationResult::error(time_all, time_test)
            .with_diagnostics(diagnostics, source_code)
            .output_json();
    } else {
        let mut len = 0;
        for d in (&mut diagnostics).take(8) {
            len += 1;
            print_diagnostic(&d, source_code);
        }

        len += diagnostics.count();

        if len > 8 {
            efprintln!(C!"note" ": some errors were omitted");
        }

        if len > 1 {
            let len = &len.to_string();
            efprintln!(R!"error" ": could not compile expression due to " {len} " previous errors");
        } else {
            efprintln!(R!"error" ": could not compile expression due to previous error");
        }
    }
}

fn print_warnings(warnings: &[Diagnostic], args: &Args, source_code: Option<&str>) {
    if matches!(&args.warnings, DiagnosticSet::Enabled(set) if set.is_empty()) {
        return;
    }

    let mut len = 0;

    for diagnostic in warnings {
        if args.warnings.is_enabled(diagnostic.kind) {
            len += 1;
            match len {
                1..=8 => print_diagnostic(diagnostic, source_code),
                9 => efprintln!(C!"note" ": some warnings were omitted"),
                _ => {}
            }
        }
    }

    if len > 1 {
        let len = len.to_string();
        efprintln!(Y!"warning" ": pomsky generated " {&len} " warnings");
    }
}

fn print_diagnostic(diagnostic: &Diagnostic, source_code: Option<&str>) {
    let kind = diagnostic.kind.to_string();
    let display = diagnostic.default_display(source_code).to_string();
    if let Some(code) = diagnostic.code {
        let code = code.to_string();
        match diagnostic.severity {
            Severity::Error => efprint!(R!"error " R!{&code} {&kind} ":\n" {&display}),
            Severity::Warning => efprint!(Y!"warning " Y!{&code} {&kind} ":\n" {&display}),
        }
    } else {
        match diagnostic.severity {
            Severity::Error => efprint!(R!"error" {&kind} ":\n" {&display}),
            Severity::Warning => efprint!(Y!"warning" {&kind} ":\n" {&display}),
        }
    }
}
