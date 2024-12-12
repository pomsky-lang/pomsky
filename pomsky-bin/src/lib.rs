#[macro_use]
mod format;
mod args;
mod result;
#[cfg(feature = "test")]
mod test;

pub use result::{
    CompilationResult, Diagnostic, Kind, QuickFix, Replacement, Severity, Span, Timings, Version,
};

use std::{path::Path, process::exit, time::Instant};

use pomsky::{
    options::{CompileOptions as PomskyCompileOptions, RegexFlavor},
    Expr,
};

use args::{CompileOptions, GlobalOptions, Input};

pub fn main() {
    let (subcommand, args) = args::parse_args().unwrap_or_else(|error| {
        efprintln!(R!"error" ": " {&error.to_string()});
        args::print_short_usage_and_help_err();
        exit(2)
    });

    if args.json && std::env::var_os("NO_COLOR").is_none() {
        std::env::set_var("NO_COLOR", "1");
    }

    match subcommand {
        args::Subcommand::Compile(compile_args) => {
            if compile_args.test.is_some() {
                handle_disabled_tests();
                show_tests_deprecated_warning();
            }

            match &compile_args.input {
                Input::Value(input) => {
                    compile(None, input, &compile_args, &args).output(
                        args.json,
                        !compile_args.no_new_line,
                        compile_args.in_test_suite,
                        input,
                    );
                }
                Input::File(path) => match std::fs::read_to_string(path) {
                    Ok(input) => {
                        compile(Some(path), &input, &compile_args, &args).output(
                            args.json,
                            !compile_args.no_new_line,
                            compile_args.in_test_suite,
                            &input,
                        );
                    }
                    Err(error) => {
                        efprintln!(R!"error" ": " {&error.to_string()});
                        exit(3);
                    }
                },
            }
        }
        args::Subcommand::Test(test_args) => {
            handle_disabled_tests();

            let compile_args = CompileOptions {
                input: Input::File(test_args.path),
                no_new_line: false,
                test: test_args.engine,
                in_test_suite: true,
            };

            let Input::File(path) = &compile_args.input else { unreachable!() };

            let current_dir = std::env::current_dir().expect("Could not get current directory");
            let path = if path.is_relative() { current_dir.join(path) } else { path.to_owned() };
            let metadata = std::fs::metadata(&path).expect("Could not get path metadata");
            let mut results = Vec::new();

            if metadata.is_dir() {
                let mut total = 0;
                let mut failed = 0;
                let start = Instant::now();

                for entry in ignore::WalkBuilder::new(path)
                    .follow_links(true)
                    .filter_entry(is_dir_or_pomsky_file)
                    .build()
                {
                    if let Ok(entry) = entry.map_err(|d| handle_walk_error(d, &current_dir)) {
                        if entry.file_type().is_some_and(|ty| ty.is_file()) {
                            total += 1;

                            let path = entry.path();
                            match std::fs::read_to_string(path) {
                                Ok(input) => {
                                    efprint!(C!"testing " {&path.strip_prefix(&current_dir)
                                        .unwrap_or(path).display().to_string()} " ... ");
                                    let result = compile(Some(path), &input, &compile_args, &args);
                                    if !result.success {
                                        failed += 1;
                                    }
                                    if args.json {
                                        eprintln!(
                                            "{}",
                                            if result.success { "ok" } else { "failed" }
                                        );
                                        results.push(result);
                                    } else {
                                        result.output(
                                            args.json,
                                            !compile_args.no_new_line,
                                            compile_args.in_test_suite,
                                            &input,
                                        )
                                    }
                                }
                                Err(error) => {
                                    efprintln!(R!"error" ": " {&error.to_string()});
                                    exit(3);
                                }
                            }
                        }
                    };
                }

                if total == 0 && !args.json {
                    if test_args.pass_with_no_tests {
                        exit(0);
                    } else {
                        efprintln!(R!"error" ": no " C!"*.pomsky" " files found to test");
                        exit(7);
                    }
                }

                let time = start.elapsed();
                eprintln!();
                if failed > 0 {
                    efprintln!("test result: " R!{&failed.to_string()} R!" pomsky file(s) failed" ", "
                        {&total.to_string()} " files tested in " {&format!("{time:.2?}")});
                } else {
                    efprintln!("test result: " G!"ok" ", " {&total.to_string()} " files tested");
                }
                print_json_test_result(args.json, &results);
            } else if metadata.is_file() {
                match std::fs::read_to_string(&path) {
                    Ok(input) => {
                        efprint!(C!"testing " {&path.strip_prefix(&current_dir).unwrap_or(&path).display().to_string()} " ... ");
                        let result = compile(Some(&path), &input, &compile_args, &args);
                        if args.json {
                            eprintln!("{}", if result.success { "ok" } else { "failed" });
                            results.push(result);
                            print_json_test_result(true, &results);
                        } else {
                            result.output(
                                args.json,
                                !compile_args.no_new_line,
                                compile_args.in_test_suite,
                                &input,
                            )
                        }
                    }
                    Err(error) => {
                        efprintln!(R!"error" ": " {&error.to_string()});
                        exit(3);
                    }
                }
            } else {
                efprintln!(R!"error" ": expected file or directory, but " c:{&path.display().to_string()} " is neither");
                exit(5);
            }
        }
    }
}

fn handle_disabled_tests() {
    #[cfg(not(feature = "test"))]
    {
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
}

fn show_tests_deprecated_warning() {
    efprintln!(Y!"warning" ": The `--test` argument is deprecated, \
        use the `pomsky test` subcommand instead");
}

fn compile(
    path: Option<&Path>,
    input: &str,
    compile_args: &CompileOptions,
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
            test::run_tests(&parsed, input, options, &mut test_errors);
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

fn handle_walk_error(error: ignore::Error, current_dir: &Path) {
    match error {
        ignore::Error::Partial(errors) => {
            for error in errors {
                handle_walk_error(error, current_dir);
            }
        }
        ignore::Error::WithLineNumber { line, err } => {
            handle_walk_error(*err, current_dir);
            efprintln!("    at line " C!{&line.to_string()});
        }
        ignore::Error::WithPath { path, err } => {
            handle_walk_error(*err, current_dir);
            efprintln!("    at path " C!{&path.display().to_string()});
        }
        ignore::Error::WithDepth { depth, err } => {
            handle_walk_error(*err, current_dir);
            efprintln!("    at depth " C!{&depth.to_string()});
        }
        ignore::Error::Loop { ancestor, child } => {
            let ancestor = ancestor.strip_prefix(current_dir).unwrap_or(&ancestor);
            let child = child.canonicalize().unwrap_or(child);

            efprintln!(R!"error" ": file system loop detected!");
            efprintln!("    ancestor: " C!{&ancestor.display().to_string()});
            efprintln!("    child: " C!{&child.display().to_string()});
        }
        ignore::Error::Io(error) => {
            efprintln!(R!"error" ": " {&error.to_string()})
        }
        ignore::Error::Glob { glob, err } => {
            efprintln!(R!"error" ": " {&err});
            if let Some(glob) = glob {
                efprintln!("    glob: " C!{&glob});
            }
        }
        ignore::Error::UnrecognizedFileType(file_type) => {
            efprintln!(R!"error" ": file type " C!{&file_type} " not recognized");
        }
        ignore::Error::InvalidDefinition => {
            efprintln!(R!"error" ": file type definition could not be parsed");
        }
    }
}

fn is_dir_or_pomsky_file(entry: &ignore::DirEntry) -> bool {
    let Some(ty) = entry.file_type() else { return false };
    if ty.is_dir() {
        return true;
    }
    let Some(ext) = entry.path().extension() else { return false };
    ext == "pomsky"
}

fn print_json_test_result(json: bool, results: &[CompilationResult]) {
    if json {
        match serde_json::to_string(&results) {
            Ok(string) => println!("{string}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
