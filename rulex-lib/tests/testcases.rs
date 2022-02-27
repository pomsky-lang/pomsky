use std::{
    fmt, fs, io,
    panic::catch_unwind,
    path::{Path, PathBuf},
    process,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use rulex::options::{CompileOptions, RegexFlavor};

pub fn main() {
    match defer_main() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

fn defer_main() -> Result<(), io::Error> {
    eprintln!("Running test cases...");

    let mut results = Vec::new();
    let marker = if atty::is(atty::Stream::Stderr) {
        TTY_MARKER
    } else {
        NO_MARKER
    };
    let Marker {
        red,
        green,
        blue,
        yellow,
        none,
        reset,
    } = marker;

    let mut include_ignored = false;
    let mut filter = String::new();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-i" | "--ignored" | "--include-ignored" => include_ignored = true,
            s if !s.starts_with('-') => filter = arg,
            option => eprintln!("warning: unrecognized option {option:?}"),
        }
    }
    if include_ignored {
        eprintln!("{yellow}Including ignored cases!{reset}");
    }

    let args = Args {
        include_ignored,
        filter,
    };

    let (tx, rx) = mpsc::channel();
    let child = thread::spawn(move || {
        let mut prev = None;
        let mut duration_millis = 0;
        const INTERVAL: u64 = 50;
        loop {
            match rx.recv_timeout(Duration::from_millis(INTERVAL)) {
                Ok(path_buf) => {
                    prev = Some(path_buf);
                    duration_millis = 0;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(prev) = &prev {
                        duration_millis += INTERVAL;
                        if duration_millis == INTERVAL {
                            eprintln!("{yellow}Warning{reset}: Test case {prev:?} is taking >50ms");
                        } else if duration_millis == 5_000 {
                            eprintln!("{red}Cancelled{reset} after 5 secs");
                            process::exit(255);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    eprintln!();
    walk_dir_recursive(marker, "./tests/testcases".into(), &mut results, tx, &args)?;
    eprintln!();

    child.join().unwrap();

    let mut ok = 0;
    let mut failed = 0;
    let mut panics = 0;
    let mut ignored = 0;
    let mut filtered = 0;

    for (path, result) in results {
        match result {
            TestResult::Success => ok += 1,
            TestResult::Ignored => ignored += 1,
            TestResult::Filtered => filtered += 1,
            TestResult::IncorrectResult {
                input,
                expected,
                got,
            } => {
                failed += 1;
                eprintln!(
                    r#"{path}: {red}incorrect result.{reset}
       {blue}input{reset}: {input}
    {blue}expected{reset}: {expected}
         {blue}got{reset}: {got}
"#,
                    path = path.to_string_lossy(),
                    expected = Print(expected, marker),
                    got = Print(got, marker),
                );
            }
            TestResult::Panic { message } => {
                panics += 1;
                eprintln!(
                    r#"{path}: {red}compilation panicked.{reset}
     {blue}message{reset}: {message:?}
"#,
                    path = path.to_string_lossy(),
                );
            }
        }
    }

    eprintln!(
        "Test cases finished. \
        {color1}{ok} ok{reset}, \
        {color2}{failed} failed{reset}, \
        {color3}{panics} panicked{reset}, \
        {color4}{filtered} filtered{reset}, \
        {color5}{ignored} ignored{reset}.\n",
        color1 = if ok > 0 { green } else { none },
        color2 = if failed > 0 { red } else { none },
        color3 = if panics > 0 { red } else { none },
        color4 = if filtered > 0 { yellow } else { none },
        color5 = if ignored > 0 { yellow } else { none },
    );

    if panics + failed > 0 {
        if !args.filter.is_empty() {
            eprintln!(
                "{yellow}Tip{reset}: You can rerun a specific test case with \
                `cargo test --test it -- {blue}<filter>{reset}`\n\
                where {blue}<filter>{reset} is a substring of the test case's file path\n"
            );
        }
    } else if ignored > 0 {
        eprintln!("{yellow}Tip{reset}: Run ignored test cases with `cargo test --test it -- -i`");
    }

    if failed > 0 || panics > 0 {
        process::exit(failed + panics);
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Marker {
    red: &'static str,
    green: &'static str,
    blue: &'static str,
    yellow: &'static str,
    none: &'static str,
    reset: &'static str,
}

const NO_MARKER: Marker = Marker {
    red: "",
    green: "",
    blue: "",
    yellow: "",
    none: "",
    reset: "",
};
const TTY_MARKER: Marker = Marker {
    red: "\x1B[38;5;9m",
    green: "\x1B[38;5;10m",
    blue: "\x1B[38;5;14m",
    yellow: "\x1B[38;5;11m",
    none: "",
    reset: "\x1B[0m",
};

fn walk_dir_recursive(
    marker @ Marker { blue, reset, .. }: Marker,
    path: PathBuf,
    results: &mut Vec<(PathBuf, TestResult)>,
    tx: Sender<PathBuf>,
    args: &Args,
) -> Result<(), io::Error> {
    let path = &path;
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File {blue}{path:?}{reset} not found"),
        ));
    }
    if path.is_dir() {
        for test in fs::read_dir(path)? {
            walk_dir_recursive(marker, test?.path(), results, tx.clone(), args)?;
        }
        Ok(())
    } else if path.is_file() {
        let content = std::fs::read_to_string(path)?;
        results.push((
            path.to_owned(),
            if filter_matches(&args.filter, path) {
                tx.send(path.to_owned()).unwrap();
                test_file(&content, path, args)
            } else {
                TestResult::Filtered
            },
        ));
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected file type of {blue}{path:?}{reset}"),
        ))
    }
}

fn filter_matches(filter: &str, path: &Path) -> bool {
    if filter.is_empty() {
        return true;
    }
    let path = path.to_string_lossy();
    path.contains(filter)
}

#[derive(Clone)]
struct Args {
    include_ignored: bool,
    filter: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Expect {
    Success,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ignored {
    Yes,
    No,
}

enum TestResult {
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

struct Print(Result<String, String>, Marker);

impl fmt::Display for Print {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Marker {
            red, green, reset, ..
        } = self.1;
        match &self.0 {
            Ok(s) => write!(f, "{green}OK{reset} /{s}/"),
            Err(s) => write!(f, "{red}ERR{reset}: {s}"),
        }
    }
}

fn test_file(content: &str, path: &Path, args: &Args) -> TestResult {
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
