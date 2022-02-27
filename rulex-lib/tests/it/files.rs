use std::{panic::catch_unwind, path::Path};

use rulex::options::{CompileOptions, RegexFlavor};

use crate::Args;

pub(crate) enum TestResult {
    Success,
    Ignored,
    Filtered,
    IncorrectResult {
        input: String,
        expected: Result<String, String>,
        got: Result<String, String>,
    },
    Panic {
        message: Option<String>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expect {
    Success,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ignored {
    Yes,
    No,
}

pub(crate) fn test_file(content: &str, path: &Path, args: &Args) -> TestResult {
    let (mut input, expected) = content.split_once("\n-----").unwrap();
    let expected = expected.trim_start_matches('-').trim_start_matches('\n');

    let (flavor, expect, ignored) = if input.starts_with("#!") {
        let (first_line, new_input) = input.split_once('\n').unwrap_or_default();
        input = new_input;
        parse_first_line_comment(first_line, path)
    } else {
        (RegexFlavor::Pcre, Expect::Success, Ignored::No)
    };

    if ignored == Ignored::Yes && !args.include_ignored {
        return TestResult::Ignored;
    }

    catch_unwind(|| {
        let parsed = rulex::Rulex::parse_and_compile(
            input,
            CompileOptions {
                flavor,
                ..Default::default()
            },
        );
        match parsed {
            Ok(got) if expect == Expect::Success => {
                if got == expected {
                    TestResult::Success
                } else {
                    TestResult::IncorrectResult {
                        input: input.to_string(),
                        expected: Ok(expected.to_string()),
                        got: Ok(got),
                    }
                }
            }
            Ok(got) => {
                // if expecting error
                TestResult::IncorrectResult {
                    input: input.to_string(),
                    expected: Err(expected.to_string()),
                    got: Ok(got),
                }
            }
            Err(err) if expect == Expect::Success => TestResult::IncorrectResult {
                input: input.to_string(),
                expected: Ok(expected.to_string()),
                got: Err(err.to_string()),
            },
            Err(err) => {
                // if expecting error
                if expected.is_empty() {
                    return TestResult::Success;
                }
                let err = err.to_string();
                if err == expected {
                    TestResult::Success
                } else {
                    TestResult::IncorrectResult {
                        input: input.to_string(),
                        expected: Err(expected.to_string()),
                        got: Err(err),
                    }
                }
            }
        }
    })
    .unwrap_or_else(|e| TestResult::Panic {
        message: e
            .downcast_ref::<String>()
            .map(ToOwned::to_owned)
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())),
    })
}

fn parse_first_line_comment(string: &str, path: &Path) -> (RegexFlavor, Expect, Ignored) {
    let mut flavor = RegexFlavor::Pcre;
    let mut expect = Expect::Success;
    let mut ignored = Ignored::No;

    for part in string.trim_start_matches("#!").split(',') {
        let part = part.trim();
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        match key {
            "flavor" | "flavour" => {
                flavor = match value.to_ascii_lowercase().as_str() {
                    "pcre" | "" => RegexFlavor::Pcre,
                    "js" | "javascript" => RegexFlavor::JavaScript,
                    "java" => RegexFlavor::Java,
                    ".net" => RegexFlavor::DotNet,
                    "python" => RegexFlavor::Python,
                    "rust" => RegexFlavor::Rust,
                    "ruby" => RegexFlavor::Ruby,
                    _ => {
                        eprintln!("Warning: Unknown flavor {value:?}");
                        eprintln!("  in {path:?}");
                        continue;
                    }
                };
            }
            "expect" => {
                expect = match value {
                    "success" => Expect::Success,
                    "error" => Expect::Error,
                    _ => {
                        eprintln!("Warning: Unknown expected outcome {value:?}");
                        eprintln!("  in {path:?}");
                        continue;
                    }
                }
            }
            "ignore" | "ignored" => {
                ignored = match value {
                    "yes" | "true" | "" => Ignored::Yes,
                    "no" | "false" => Ignored::No,
                    _ => {
                        eprintln!("Warning: Unknown boolean {value:?}");
                        eprintln!("  in {path:?}");
                        continue;
                    }
                }
            }
            _ => {
                eprintln!("Warning: Unknown option {key:?}");
                eprintln!("  in {path:?}");
            }
        }
    }
    (flavor, expect, ignored)
}
