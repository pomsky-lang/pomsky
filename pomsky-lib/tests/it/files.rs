use std::{
    fmt::Write as _,
    panic::{catch_unwind, UnwindSafe},
    path::Path,
};

use pomsky::{
    error::CompileError,
    options::{CompileOptions, ParseOptions, RegexFlavor},
};

use crate::{color::Color::*, Args};

pub(crate) enum TestResult {
    Success,
    Ignored,
    Filtered,
    Blessed,
    IncorrectResult { input: String, expected: Result<String, String>, got: Result<String, String> },
    Panic { message: Option<String> },
}

#[derive(Clone, Copy)]
struct Options {
    flavor: RegexFlavor,
    ignore: bool,
    expected_outcome: Outcome,
}

impl Default for Options {
    fn default() -> Self {
        Self { flavor: RegexFlavor::Pcre, ignore: false, expected_outcome: Outcome::Success }
    }
}

impl Options {
    fn parse(line: &str, path: &Path) -> Self {
        let mut result = Options::default();

        for part in line.trim_start_matches("#!").split(',') {
            let part = part.trim();
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            match key {
                "flavor" => {
                    result.flavor = match value.to_ascii_lowercase().as_str() {
                        "pcre" | "" => RegexFlavor::Pcre,
                        "js" | "javascript" => RegexFlavor::JavaScript,
                        "java" => RegexFlavor::Java,
                        ".net" | "dotnet" => RegexFlavor::DotNet,
                        "python" => RegexFlavor::Python,
                        "rust" => RegexFlavor::Rust,
                        "ruby" => RegexFlavor::Ruby,
                        _ => {
                            eprintln!("{}: Unknown flavor {value:?}", Yellow("Warning"));
                            eprintln!("  in {path:?}");
                            continue;
                        }
                    };
                }
                "expect" => {
                    result.expected_outcome = match value {
                        "success" => Outcome::Success,
                        "error" => Outcome::Error,
                        _ => {
                            eprintln!("{}: Unknown expected outcome {value:?}", Yellow("Warning"));
                            eprintln!("  in {path:?}");
                            continue;
                        }
                    }
                }
                "ignore" | "ignored" => {
                    result.ignore = match value {
                        "yes" | "true" | "" => true,
                        "no" | "false" => false,
                        _ => {
                            eprintln!("{}: Unknown boolean {value:?}", Yellow("Warning"));
                            eprintln!("  in {path:?}");
                            continue;
                        }
                    }
                }
                _ => {
                    eprintln!("{}: Unknown option {key:?}", Yellow("Warning"));
                    eprintln!("  in {path:?}");
                }
            }
        }
        result
    }
}

#[derive(Clone, Copy, Debug)]
enum Outcome {
    Success,
    Error,
}

impl Outcome {
    fn of(self, inner: String) -> Result<String, String> {
        match self {
            Outcome::Success => Ok(inner),
            Outcome::Error => Err(inner),
        }
    }
}

pub(crate) fn test_file(content: &str, path: &Path, args: &Args, bless: bool) -> TestResult {
    let (input, expected, options) = process_content(content, path);

    if options.ignore && !args.include_ignored {
        return TestResult::Ignored;
    }

    catch_panics(|| {
        let parsed = pomsky::Expr::parse_and_compile(
            input,
            ParseOptions::default(),
            CompileOptions { flavor: options.flavor },
        );

        match parsed {
            Ok((mut got, warnings)) => {
                for warning in warnings {
                    got.push_str("\nWARNING: ");
                    got.write_fmt(format_args!("{}", warning)).unwrap();
                }

                match options.expected_outcome {
                    Outcome::Success if got == expected => TestResult::Success,
                    _ if bless => {
                        let contents = create_content(
                            input,
                            &got,
                            Options { expected_outcome: Outcome::Success, ..options },
                        );
                        std::fs::write(path, contents)
                            .expect("Failed to bless test because of IO error");

                        TestResult::Blessed
                    }
                    outcome => TestResult::IncorrectResult {
                        input: strip_input(input),
                        expected: outcome.of(expected.to_string()),
                        got: Ok(got),
                    },
                }
            }
            Err(err) => {
                let err = error_to_string(err, input);

                match options.expected_outcome {
                    Outcome::Error if expected.is_empty() || expected == err => TestResult::Success,
                    _ if bless => {
                        let contents = create_content(
                            input,
                            &err,
                            Options { expected_outcome: Outcome::Error, ..options },
                        );
                        std::fs::write(path, contents)
                            .expect("Failed to bless test because of IO error");

                        TestResult::Blessed
                    }
                    outcome => TestResult::IncorrectResult {
                        input: strip_input(input),
                        expected: outcome.of(expected.to_string()),
                        got: Err(err),
                    },
                }
            }
        }
    })
    .unwrap_or_else(|message| TestResult::Panic { message })
}

fn error_to_string(err: CompileError, input: &str) -> String {
    let diagnostic = err.diagnostic(input);
    if let Some(help) = diagnostic.help {
        format!("ERROR: {}\nHELP: {}\nSPAN: {}", diagnostic.msg, help, diagnostic.span)
    } else {
        format!("ERROR: {}\nSPAN: {}", diagnostic.msg, diagnostic.span)
    }
}

fn process_content<'a>(content: &'a str, path: &Path) -> (&'a str, &'a str, Options) {
    let (mut input, expected) = content.split_once("\n-----").unwrap_or((content, ""));
    let expected = expected.trim_start_matches('-').trim_start_matches('\n');

    let options = if input.starts_with("#!") {
        let (first_line, new_input) = input.split_once('\n').unwrap_or_default();
        input = new_input;
        Options::parse(first_line, path)
    } else {
        Options::default()
    };
    (input, expected, options)
}

fn create_content(input: &str, outcome: &str, options: Options) -> String {
    let mut option_strings = vec![];
    if let Outcome::Error = options.expected_outcome {
        option_strings.push(String::from("expect=error"));
    }
    if options.ignore {
        option_strings.push(String::from("ignore"));
    }
    if options.flavor != RegexFlavor::Pcre {
        option_strings.push(format!("flavor={:?}", options.flavor));
    }

    let option_strings = if option_strings.is_empty() {
        "".to_string()
    } else {
        format!("#! {}\n", option_strings.join(", "))
    };

    option_strings + input + "\n-----\n" + outcome
}

fn strip_input(input: &str) -> String {
    input
        .lines()
        .filter(|l| {
            let l = l.trim_start();
            !l.is_empty() && !l.starts_with('#')
        })
        .collect()
}

fn catch_panics<R>(f: impl Fn() -> R + UnwindSafe) -> Result<R, Option<String>> {
    catch_unwind(f).map_err(|err| {
        err.downcast_ref::<String>()
            .map(ToOwned::to_owned)
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
    })
}
