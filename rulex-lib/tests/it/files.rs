use std::{panic::catch_unwind, path::Path};

use rulex::options::{CompileOptions, RegexFlavor};

use crate::{color::Color::*, Args};

pub(crate) enum TestResult {
    Success,
    Ignored,
    Filtered,
    IncorrectResult { input: String, expected: Result<String, String>, got: Result<String, String> },
    Panic { message: Option<String> },
}

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
                        ".net" => RegexFlavor::DotNet,
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

#[derive(Clone, Copy)]
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

pub(crate) fn test_file(content: &str, path: &Path, args: &Args) -> TestResult {
    let (mut input, expected) = content.split_once("\n-----").unwrap();
    let expected = expected.trim_start_matches('-').trim_start_matches('\n');

    let options = if input.starts_with("#!") {
        let (first_line, new_input) = input.split_once('\n').unwrap_or_default();
        input = new_input;
        Options::parse(first_line, path)
    } else {
        Options::default()
    };

    if options.ignore && !args.include_ignored {
        return TestResult::Ignored;
    }

    catch_unwind(|| {
        let parsed = rulex::Rulex::parse_and_compile(
            input,
            CompileOptions { flavor: options.flavor, ..Default::default() },
        );
        match parsed {
            Ok(got) => match options.expected_outcome {
                Outcome::Success if got == expected => TestResult::Success,
                outcome => TestResult::IncorrectResult {
                    input: strip_input(input),
                    expected: outcome.of(expected.to_string()),
                    got: Ok(got),
                },
            },
            Err(err) => match options.expected_outcome {
                Outcome::Error if expected.is_empty() || expected == err.to_string() => {
                    TestResult::Success
                }
                outcome => TestResult::IncorrectResult {
                    input: strip_input(input),
                    expected: outcome.of(expected.to_string()),
                    got: Err(err.to_string()),
                },
            },
        }
    })
    .unwrap_or_else(|err| TestResult::Panic {
        message: err
            .downcast_ref::<String>()
            .map(ToOwned::to_owned)
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string())),
    })
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
