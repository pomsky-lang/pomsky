use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use pomsky::{
    diagnose::{Diagnostic, Severity},
    options::{CompileOptions, RegexFlavor},
};
use regex_test::RegexTest;

use crate::{args::Args, color::Color::*};

pub(crate) enum TestResult {
    Success,
    Ignored,
    Blessed,
    IncorrectResult { input: String, expected: Result<String, String>, got: Result<String, String> },
    // Panic { message: Option<String> },
    InvalidOutput(String),
}

#[derive(Clone, Copy, Debug)]
struct Options {
    /// The regex flavor to compile with
    flavor: RegexFlavor,
    /// Whether this test should be ignored entirely
    ignore: bool,
    /// Whether we expect a compilation error from pomsky or not
    expected_outcome: Outcome,
    /// Whether we attempt to compile the output with the `regex` crate.
    ///
    /// Defaults to `true` if the regex flavor is `rust`.
    compile: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            flavor: RegexFlavor::Rust,
            ignore: false,
            expected_outcome: Outcome::Success,
            compile: true,
        }
    }
}

impl Options {
    fn parse(line: &str, path: &Path) -> Self {
        let mut result = Options::default();
        let mut compile = None;

        for part in line.trim_start_matches("#!").split(',') {
            let part = part.trim();
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            match key {
                "flavor" => {
                    result.flavor = match value.to_ascii_lowercase().as_str() {
                        "pcre" => RegexFlavor::Pcre,
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
                "compile" => {
                    compile = Some(match value {
                        "yes" | "true" | "" => true,
                        "no" | "false" => false,
                        _ => {
                            eprintln!("{}: Unknown boolean {value:?}", Yellow("Warning"));
                            eprintln!("  in {path:?}");
                            continue;
                        }
                    });
                }
                _ => {
                    eprintln!("{}: Unknown option {key:?}", Yellow("Warning"));
                    eprintln!("  in {path:?}");
                }
            }
        }
        result.compile = compile.unwrap_or_else(|| can_compile_regex(result.flavor));
        result
    }
}

fn can_compile_regex(flavor: RegexFlavor) -> bool {
    use RegexFlavor::*;
    matches!(flavor, Rust | Pcre | Ruby | JavaScript | Java | Python)
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

pub(crate) fn test_file(
    content: String,
    path: PathBuf,
    args: &Args,
    proc: &RegexTest,
) -> TestResult {
    let (input, expected, options) = process_content(&content, &path);
    let input_owned = input.to_string();

    if options.ignore && !args.include_ignored {
        return TestResult::Ignored;
    }

    let parsed = pomsky::Expr::parse_and_compile(
        &input_owned,
        CompileOptions { flavor: options.flavor, ..Default::default() },
    );

    match parsed {
        (Some(regex), warnings) => {
            let mut got = regex.clone();
            for warning in warnings {
                got.push_str("\nWARNING: ");
                got.write_fmt(format_args!("{warning}\n  at {}", warning.span)).unwrap();
            }

            match options.expected_outcome {
                Outcome::Success if got == expected => {
                    if options.compile {
                        let outcome = match options.flavor {
                            RegexFlavor::Rust => proc.test_rust(&regex),
                            RegexFlavor::Pcre => proc.test_pcre(&regex),
                            RegexFlavor::Ruby => proc.test_ruby(&regex),
                            RegexFlavor::JavaScript => proc.test_js(regex),
                            RegexFlavor::Java => proc.test_java(regex),
                            RegexFlavor::Python => proc.test_python(regex),
                            _ => {
                                eprintln!(
                                    "{}: Flavor {:?} can't be compiled at the moment",
                                    Yellow("Warning"),
                                    options.flavor
                                );
                                eprintln!("  in {path:?}");
                                regex_test::Outcome::Success
                            }
                        };
                        match outcome {
                            regex_test::Outcome::Success => TestResult::Success,
                            regex_test::Outcome::Error(e) => TestResult::InvalidOutput(e),
                        }
                    } else {
                        TestResult::Success
                    }
                }
                _ if args.bless => {
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
                    input: input.to_string(),
                    expected: outcome.of(expected.to_string()),
                    got: Ok(got),
                },
            }
        }
        (None, err) => {
            let err = errors_to_string(err);

            match options.expected_outcome {
                Outcome::Error if expected.is_empty() || expected == err => TestResult::Success,
                _ if args.bless => {
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
                    input: input.to_string(),
                    expected: outcome.of(expected.to_string()),
                    got: Err(err),
                },
            }
        }
    }
}

fn errors_to_string(diagnostics: Vec<Diagnostic>) -> String {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let sev = match diagnostic.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARNING",
            };
            if let Some(help) = diagnostic.help {
                format!("{sev}: {}\nHELP: {help}\nSPAN: {}", diagnostic.msg, diagnostic.span)
            } else {
                format!("{sev}: {}\nSPAN: {}", diagnostic.msg, diagnostic.span)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn process_content<'a>(content: &'a str, path: &Path) -> (&'a str, &'a str, Options) {
    let (mut input, expected) = content.split_once("\n-----").unwrap_or((content, ""));
    let expected = expected.trim_start_matches('-');
    let expected = expected.strip_prefix('\n').unwrap_or(expected);

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
    if options.flavor != RegexFlavor::Rust {
        option_strings.push(format!("flavor={:?}", options.flavor));
    }

    let option_strings = if option_strings.is_empty() {
        "".to_string()
    } else {
        format!("#! {}\n", option_strings.join(", "))
    };

    option_strings + input + "\n-----\n" + outcome
}
