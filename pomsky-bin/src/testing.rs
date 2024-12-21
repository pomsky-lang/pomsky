use std::{path::Path, process::exit, time::Instant};

use helptext::text;
use pomsky::options::RegexFlavor;

use crate::{
    args::{CompileOptions, GlobalOptions, Input, RegexEngine, TestOptions},
    format::Logger,
    CompilationResult,
};

pub(crate) struct TestDirectoryResult {
    pub(crate) total: usize,
    pub(crate) failed: usize,
    pub(crate) results: Vec<CompilationResult>,
}

pub(crate) fn test(logger: &Logger, args: GlobalOptions, test_args: TestOptions) {
    let (test_engine, flavor) = match (args.flavor, test_args.engine) {
        (None, None) => {
            logger.error().println("No regex engine specified");
            exit(2);
        }
        (None, Some(engine)) => match engine {
            RegexEngine::Pcre2 => (engine, RegexFlavor::Pcre),
            RegexEngine::Rust => (engine, RegexFlavor::Rust),
        },
        (Some(flavor), None) => match flavor {
            RegexFlavor::Pcre => (RegexEngine::Pcre2, flavor),
            RegexFlavor::Rust => (RegexEngine::Rust, flavor),
            _ => {
                logger
                    .error()
                    .println(format_args!("No supported regex engine for the {flavor:?} flavor"));
                exit(2);
            }
        },
        (Some(flavor), Some(engine)) => (engine, flavor),
    };

    let args = GlobalOptions { flavor: Some(flavor), ..args };
    let compile_args = CompileOptions {
        input: Input::File(test_args.path),
        no_new_line: false,
        test: Some(test_engine),
        in_test_suite: true,
    };

    let Input::File(path) = &compile_args.input else { unreachable!() };

    let Ok(current_dir) = std::env::current_dir() else {
        logger.error().println("Could not get current directory");
        exit(3);
    };
    let path = if path.is_relative() { current_dir.join(path) } else { path.to_owned() };
    let Ok(metadata) = std::fs::metadata(&path) else {
        logger.error().println("Could not get path metadata");
        exit(3);
    };

    if metadata.is_dir() {
        let start = Instant::now();

        let TestDirectoryResult { total, failed, results } =
            test_directory(logger, &path, &current_dir, &args, &compile_args);

        if total == 0 && !args.json {
            if test_args.pass_with_no_tests {
                exit(0);
            } else {
                logger.error().println("no `*.pomsky` files found to test");
                logger.note().println("run with `--pass-with-no-tests` to ignore this error");
                exit(5);
            }
        }

        logger.emptyln();

        let time = start.elapsed();
        let time_fmt = format!("{time:.2?}");
        if failed > 0 {
            logger.basic().fmtln(text![
                "test result: " R!{&failed.to_string()} R!" pomsky file(s) failed" ", "
                {&total.to_string()} " files tested in " {&time_fmt}
            ]);
        } else {
            logger.basic().fmtln(text![
                "test result: " G!"ok" ", "
                {&total.to_string()} " files tested in " {&time_fmt}
            ]);
        }
        print_json_test_result(args.json, &results);
    } else if metadata.is_file() {
        let mut results = Vec::new();
        let mut failed = 0;
        test_single(logger, &path, &current_dir, &args, &compile_args, &mut results, &mut failed);
        print_json_test_result(args.json, &results);
    } else {
        logger.error().println(format_args!(
            "expected file or directory, but `{}` is neither",
            path.display()
        ));
        exit(3);
    }
}

fn test_directory(
    logger: &Logger,
    path: &Path,
    current_dir: &Path,
    args: &GlobalOptions,
    compile_args: &CompileOptions,
) -> TestDirectoryResult {
    let mut total = 0;
    let mut failed = 0;
    let mut results = Vec::new();

    for entry in ignore::WalkBuilder::new(path)
        .follow_links(true)
        .filter_entry(is_dir_or_pomsky_file)
        .build()
    {
        if let Ok(entry) = entry.map_err(|d| handle_walk_error(d, logger, current_dir)) {
            if entry.file_type().is_some_and(|ty| ty.is_file()) {
                total += 1;

                let path = entry.path();
                test_single(
                    logger,
                    path,
                    current_dir,
                    args,
                    compile_args,
                    &mut results,
                    &mut failed,
                );
            }
        };
    }

    TestDirectoryResult { total, failed, results }
}

fn test_single(
    logger: &Logger,
    path: &Path,
    current_dir: &Path,
    args: &GlobalOptions,
    compile_args: &CompileOptions,
    results: &mut Vec<CompilationResult>,
    failed: &mut usize,
) {
    match std::fs::read_to_string(path) {
        Ok(input) => {
            logger.basic().fmt(text![C!"testing " {&show_relative(path, current_dir)} " ... "]);
            let result = super::compile(Some(path), &input, compile_args, args);
            if !result.success {
                *failed += 1;
            }
            if args.json {
                results.push(result);
            } else {
                result.output(
                    logger,
                    args.json,
                    !compile_args.no_new_line,
                    compile_args.in_test_suite,
                    &input,
                );
            }
        }
        Err(error) => {
            logger.error().println(error);
            exit(3);
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

fn handle_walk_error(error: ignore::Error, logger: &Logger, current_dir: &Path) {
    match error {
        ignore::Error::Partial(errors) => {
            for error in errors {
                handle_walk_error(error, logger, current_dir);
            }
        }
        ignore::Error::WithLineNumber { line, err } => {
            handle_walk_error(*err, logger, current_dir);
            logger.basic().fmtln(text!["    at line " C!{&line.to_string()}]);
        }
        ignore::Error::WithPath { path, err } => {
            handle_walk_error(*err, logger, current_dir);
            logger.basic().fmtln(text!["    at path " C!{&path.display().to_string()}]);
        }
        ignore::Error::WithDepth { depth, err } => {
            handle_walk_error(*err, logger, current_dir);
            logger.basic().fmtln(text!["    at depth " C!{&depth.to_string()}]);
        }
        ignore::Error::Loop { ancestor, child } => {
            let ancestor = ancestor.strip_prefix(current_dir).unwrap_or(&ancestor);
            let child = child.canonicalize().unwrap_or(child);

            logger.error().println("file system loop detected!");
            logger.basic().fmtln(text!["    ancestor: " C!{&ancestor.display().to_string()}]);
            logger.basic().fmtln(text!["    child: " C!{&child.display().to_string()}]);
        }
        ignore::Error::Io(error) => {
            logger.error().println(error);
        }
        ignore::Error::Glob { glob, err } => {
            logger.error().println(err);
            if let Some(glob) = glob {
                logger.basic().fmtln(text!["    glob: " C!{&glob}]);
            }
        }
        ignore::Error::UnrecognizedFileType(file_type) => {
            logger.error().println(format_args!("file type `{file_type}` not recognized"));
        }
        ignore::Error::InvalidDefinition => {
            logger.error().println("file type definition could not be parsed");
        }
    }
}

pub(crate) fn print_json_test_result(json: bool, results: &[CompilationResult]) {
    if json {
        match serde_json::to_string(&results) {
            Ok(string) => println!("{string}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn show_relative(path: &Path, relative_to: &Path) -> String {
    path.strip_prefix(relative_to).unwrap_or(path).display().to_string()
}
