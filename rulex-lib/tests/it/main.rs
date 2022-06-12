use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{self, exit},
    sync::mpsc::Sender,
    time::Instant,
};

use crate::{color::Color::*, files::TestResult};

#[macro_use]
mod color;
mod files;
mod fuzzer;
mod timeout;

struct Args {
    include_ignored: bool,
    filter: String,
    bless: bool,

    fuzz_ranges: bool,
    fuzz_start: usize,
    fuzz_step: usize,
    thoroughness: usize,
}

impl Args {
    fn parse() -> Self {
        let mut include_ignored = false;
        let mut filter = String::new();
        let mut bless = false;
        let mut fuzz_ranges = false;
        let mut fuzz_start = 0;
        let mut fuzz_step = 1;
        let mut thoroughness = 40;

        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--" => {}
                "-i" | "--ignored" | "--include-ignored" => include_ignored = true,
                "--fuzz-ranges" => fuzz_ranges = true,
                "--bless" => bless = true,
                "help" | "--help" | "-h" => {
                    eprintln!(
                        "USAGE:\n    \
                            cargo test --test it -- [OPTIONS]\n\
                        \n\
                        OPTIONS:\n    \
                            -i,--ignored            Include ignored test cases\n    \
                            --bless                 Bless failed test cases\n    \
                            --fuzz-ranges           Fuzz the `range '...'-'...' syntax`\n    \
                            --thoroughness=<NUMBER> Specify how thorough each range is fuzzed [default: 40]\n    \
                            --fuzz-start=<NUMBER>   Specify the bound where to start fuzzing [default: 0]\n    \
                            --fuzz-step=<NUMBER>    Only fuzz every n-th number (use prime number to make samples more arbitrary)\n    \
                            -h,--help               Show usage information"
                    );
                    exit(0);
                }
                s if s.starts_with("--thoroughness=") => {
                    let s = s.strip_prefix("--thoroughness=").unwrap();
                    thoroughness = s.parse().unwrap();
                }
                s if s.starts_with("--fuzz-start=") => {
                    let s = s.strip_prefix("--fuzz-start=").unwrap();
                    fuzz_start = s.parse().unwrap();
                }
                s if s.starts_with("--fuzz-step=") => {
                    let s = s.strip_prefix("--fuzz-step=").unwrap();
                    fuzz_step = s.parse().unwrap();
                }
                s if !s.starts_with('-') => filter = arg,
                option => eprintln!(
                    "{}: unrecognized option {option:?}\ntry `--help` help",
                    Yellow("Warning")
                ),
            }
        }
        Args { include_ignored, filter, bless, fuzz_ranges, fuzz_start, fuzz_step, thoroughness }
    }
}

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
    println!("\nrunning integration tests");

    let mut results = Vec::new();

    let args = Args::parse();
    if args.include_ignored {
        println!("{}", Yellow("including ignored cases!"));
    }

    let (tx, child) = timeout::timeout_thread();

    println!();
    let start = Instant::now();
    test_dir_recursive("./tests/testcases".into(), &mut results, tx, &args, args.bless)?;
    let elapsed = start.elapsed();
    println!();

    child.join().unwrap();

    let mut ok = 0;
    let mut failed = 0;
    let mut blessed = 0;
    let mut ignored = 0;
    let mut filtered = 0;

    for (path, result) in results {
        match result {
            TestResult::Success => ok += 1,
            TestResult::Ignored => ignored += 1,
            TestResult::Filtered => filtered += 1,
            TestResult::Blessed => blessed += 1,
            TestResult::IncorrectResult { input, expected, got } => {
                failed += 1;
                println!("{}: {}", path.to_string_lossy(), Red("incorrect result."));
                println!("       {}: {}", Blue("input"), input);
                println!("    {}: {}", Blue("expected"), Print(expected));
                println!("         {}: {}", Blue("got"), Print(got));
                println!();
            }
            TestResult::Panic { message } => {
                failed += 1;
                println!("{}: {}", path.to_string_lossy(), Red("test panicked."));
                if let Some(message) = message {
                    println!("     {}: {message}", Blue("message"));
                }
                println!();
            }
        }
    }

    println!(
        "test result: {}. {}; {}; {}; {}; finished in {:.2?}\n",
        if failed == 0 { Green("ok") } else { Red("FAILED") },
        color!(Green if ok > 0; ok, " passed"),
        if blessed > 0 {
            color!(Yellow if blessed > 0; blessed, " blessed")
        } else {
            color!(Red if failed > 0; failed, " failed")
        },
        color!(Yellow if ignored > 0; ignored, " ignored"),
        color!(Yellow if filtered > 0; filtered, " filtered out"),
        elapsed,
    );

    if blessed > 0 {
        println!(
            "{t_warn}: Some failed tests were blessed. Check the git diff \
            to see if their result is correct\n",
            t_warn = Yellow("warning"),
        );
    } else if failed > 0 {
        if args.filter.is_empty() {
            println!(
                "{t_tip}: you can rerun a specific test case with \
                `cargo test --test it -- {t_filter}`\n\
                where {t_filter} is a substring of the test case's file path\n",
                t_tip = Yellow("tip"),
                t_filter = Blue("<filter>"),
            );
            println!(
                "{t_tip}: Automatically correct failed testcases with \
                `cargo test --test it -- {t_bless}`\n",
                t_tip = Yellow("tip"),
                t_bless = Blue("--bless"),
            );
        }
    } else if ignored > 0 {
        println!(
            "{t_tip}: run ignored test cases with `cargo test --test it -- -i`",
            t_tip = Yellow("tip"),
        );
    }

    if failed > 0 {
        process::exit(failed);
    }

    if args.fuzz_ranges {
        println!(
            "\nfuzzing ranges (thoroughness: {}, step: {})",
            args.thoroughness, args.fuzz_step
        );

        let mut errors = Vec::new();
        println!();
        let start = Instant::now();
        fuzzer::fuzz_ranges(&mut errors, args.thoroughness, args.fuzz_start, args.fuzz_step);
        let elapsed = start.elapsed();
        println!();

        let failed = errors.len();

        println!(
            "fuzz result: {}. {}; finished in {:.2?}\n",
            if failed == 0 { Green("ok") } else { Red("FAILED") },
            color!(Red if failed > 0; failed, " failed"),
            elapsed,
        );
    }

    Ok(())
}

fn test_dir_recursive(
    path: PathBuf,
    results: &mut Vec<(PathBuf, TestResult)>,
    tx: Sender<PathBuf>,
    args: &Args,
    bless: bool,
) -> Result<(), io::Error> {
    let path = &path;
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file {:?} not found", Blue(path)),
        ));
    }
    if path.is_dir() {
        for test in fs::read_dir(path)? {
            test_dir_recursive(test?.path(), results, tx.clone(), args, bless)?;
        }
        Ok(())
    } else if path.is_file() {
        let mut content = std::fs::read_to_string(path)?;
        content.retain(|c| c != '\r');
        results.push((
            path.to_owned(),
            if filter_matches(&args.filter, path) {
                tx.send(path.to_owned()).unwrap();
                files::test_file(&content, path, args, bless)
            } else {
                TestResult::Filtered
            },
        ));
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("unexpected file type of {:?}", Blue(path)),
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

struct Print(Result<String, String>);

impl fmt::Display for Print {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Ok(s) => write!(f, "{} /{s}/", Green("OK")),
            Err(s) => write!(f, "{}: {s}", Red("ERR")),
        }
    }
}
