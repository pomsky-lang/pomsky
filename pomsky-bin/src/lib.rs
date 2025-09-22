#[macro_use]
mod format;
mod args;
mod result;
#[cfg(feature = "test")]
mod test_runner;
#[cfg(feature = "test")]
mod testing;

use format::Logger;
pub use result::{
    CompilationResult, Diagnostic, Kind, QuickFix, Replacement, Severity, Span, Timings, Version,
};

use std::{path::Path, process::exit, time::Instant};

use pomsky::{
    Expr,
    options::{CompileOptions as PomskyCompileOptions, RegexFlavor},
};

use args::{CompileOptions, GlobalOptions, Input};

pub fn main() {
    let mut logger = Logger::new();

    let (subcommand, args) = args::parse_args(&logger).unwrap_or_else(|error| {
        logger.error().println(error);
        args::print_usage_and_help();
        exit(2)
    });

    if args.json {
        logger = logger.color(false).enabled(false);
    }

    match subcommand {
        args::Subcommand::Compile(compile_args) => {
            if compile_args.test.is_some() {
                handle_disabled_tests(&logger);
                show_tests_deprecated_warning(&logger);
            }

            match &compile_args.input {
                Input::Value(input) => {
                    compile(None, input, &compile_args, &args).output(
                        &logger,
                        args.json,
                        !compile_args.no_new_line,
                        compile_args.in_test_suite,
                        input,
                    );
                }
                Input::File(path) => match std::fs::read_to_string(path) {
                    Ok(input) => {
                        compile(Some(path), &input, &compile_args, &args).output(
                            &logger,
                            args.json,
                            !compile_args.no_new_line,
                            compile_args.in_test_suite,
                            &input,
                        );
                    }
                    Err(error) => {
                        logger.error().println(error);
                        exit(3);
                    }
                },
            }
        }
        args::Subcommand::Test(_test_args) => {
            handle_disabled_tests(&logger);

            #[cfg(feature = "test")]
            testing::test(&logger, args, _test_args);
        }
    }
}

fn handle_disabled_tests(_logger: &Logger) {
    #[cfg(not(feature = "test"))]
    {
        _logger.error_plain(
            "Testing is not supported, because this pomsky binary \
            was compiled with the `test` feature disabled!",
        );
        exit(4);
    }
}

fn show_tests_deprecated_warning(logger: &Logger) {
    logger
        .warn()
        .println("The `--test` argument is deprecated, use the `pomsky test` subcommand instead");
}

// TODO: refactor this
fn compile(
    path: Option<&Path>,
    input: &str,
    #[allow(unused)] compile_args: &CompileOptions,
    args: &GlobalOptions,
) -> CompilationResult {
    let start = Instant::now();

    let options = PomskyCompileOptions {
        flavor: args.flavor.unwrap_or(RegexFlavor::Pcre),
        max_range_size: 12,
        allowed_features: args.allowed_features,
    };

    let (parsed, warnings) = match Expr::parse(input) {
        (Some(res), warnings) => (res, warnings),
        (None, err) => {
            return CompilationResult::error(
                path,
                start.elapsed().as_micros(),
                0,
                err,
                input,
                &args.warnings,
                args.json,
            );
        }
    };

    if args.debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:?}\n");
    }

    let mut diagnostics = warnings.collect::<Vec<_>>();

    let (output, compile_diagnostics) = parsed.compile(input, options);
    diagnostics.extend(compile_diagnostics);

    if let Some(output) = output {
        #[allow(unused_mut)] // the `mut` is only needed when cfg(feature = "test")
        let mut time_test = 0;

        #[cfg(feature = "test")]
        if compile_args.test.is_some() {
            let mut test_errors = Vec::new();

            let start = Instant::now();
            test_runner::run_tests(&parsed, input, options, &mut test_errors);
            time_test = start.elapsed().as_micros();

            if !test_errors.is_empty() {
                diagnostics.extend(test_errors);
                return CompilationResult::error(
                    path,
                    start.elapsed().as_micros(),
                    time_test,
                    diagnostics,
                    input,
                    &args.warnings,
                    args.json,
                );
            }
        }

        CompilationResult::success(
            path,
            output,
            start.elapsed().as_micros(),
            time_test,
            diagnostics,
            input,
            &args.warnings,
            args.json,
        )
    } else {
        CompilationResult::error(
            path,
            start.elapsed().as_micros(),
            0,
            diagnostics,
            input,
            &args.warnings,
            args.json,
        )
    }
}
