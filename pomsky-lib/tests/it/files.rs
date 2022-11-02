use std::{
    fmt::Write as _,
    panic::{catch_unwind, UnwindSafe},
    path::Path,
};

use pomsky::{
    error::CompileError,
    options::{CompileOptions, RegexFlavor},
};

use crate::{color::Color::*, Args};

pub(crate) enum TestResult {
    Success,
    Ignored,
    Filtered,
    Blessed,
    IncorrectResult { input: String, expected: Result<String, String>, got: Result<String, String> },
    Panic { message: Option<String> },
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
    flavor == RegexFlavor::Rust || flavor == RegexFlavor::Pcre
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
            CompileOptions { flavor: options.flavor, ..Default::default() },
        );

        match parsed {
            Ok((regex, warnings)) => {
                let mut got = regex.clone();
                for warning in warnings {
                    got.push_str("\nWARNING: ");
                    got.write_fmt(format_args!("{}", warning)).unwrap();
                }

                match options.expected_outcome {
                    Outcome::Success if got == expected => {
                        if options.compile {
                            match options.flavor {
                                RegexFlavor::Rust => match regex::Regex::new(&regex) {
                                    Ok(_) => TestResult::Success,
                                    Err(e) => TestResult::InvalidOutput(e.to_string()),
                                },
                                RegexFlavor::Pcre => match pcre2::bytes::RegexBuilder::new()
                                    .utf(true)
                                    .build(&regex)
                                {
                                    Ok(_) => TestResult::Success,
                                    Err(e) => TestResult::InvalidOutput(format!(
                                        "{e}\n>\n> {}\n> {:width$}^",
                                        &regex,
                                        "",
                                        width = regex[0..e.offset().unwrap_or(0)].chars().count()
                                    )),
                                },
                                _ => {
                                    eprintln!(
                                        "{}: Flavor {:?} can't be compiled at the moment",
                                        Yellow("Warning"),
                                        options.flavor
                                    );
                                    eprintln!("  in {path:?}");
                                    TestResult::Success
                                }
                            }
                        } else {
                            TestResult::Success
                        }
                    }
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
                        input: input.to_string(),
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
                        input: input.to_string(),
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
    let diagnostics = err.diagnostics(input);
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            if let Some(help) = diagnostic.help {
                format!("ERROR: {}\nHELP: {}\nSPAN: {}", diagnostic.msg, help, diagnostic.span)
            } else {
                format!("ERROR: {}\nSPAN: {}", diagnostic.msg, diagnostic.span)
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

fn catch_panics<R>(f: impl Fn() -> R + UnwindSafe) -> Result<R, Option<String>> {
    catch_unwind(f).map_err(|err| {
        err.downcast_ref::<String>()
            .map(ToOwned::to_owned)
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
    })
}
