use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process,
    sync::mpsc::Sender,
    time::Instant,
};

use crate::{color::Color::*, files::TestResult};

#[macro_use]
mod color;
mod files;
mod timeout;

struct Args {
    include_ignored: bool,
    filter: String,
}

impl Args {
    fn parse() -> Self {
        let mut include_ignored = false;
        let mut filter = String::new();
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "-i" | "--ignored" | "--include-ignored" => include_ignored = true,
                s if !s.starts_with('-') => filter = arg,
                option => eprintln!("{}: unrecognized option {option:?}", Yellow("Warning")),
            }
        }
        Args {
            include_ignored,
            filter,
        }
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
    walk_dir_recursive("./tests/testcases".into(), &mut results, tx, &args)?;
    let elapsed = start.elapsed();
    println!();

    child.join().unwrap();

    let mut ok = 0;
    let mut failed = 0;
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
        if failed == 0 {
            Green("ok")
        } else {
            Red("FAILED")
        },
        color!(Green if ok > 0; ok, " passed"),
        color!(Red if failed > 0; failed, " failed"),
        color!(Yellow if ignored > 0; ignored, " ignored"),
        color!(Yellow if filtered > 0; filtered, " filtered out"),
        elapsed,
    );

    if failed > 0 {
        if args.filter.is_empty() {
            println!(
                "{t_tip}: you can rerun a specific test case with \
                `cargo test --test it -- {t_filter}`\n\
                where {t_filter} is a substring of the test case's file path\n",
                t_tip = Yellow("tip"),
                t_filter = Blue("<filter>"),
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

    Ok(())
}

fn walk_dir_recursive(
    path: PathBuf,
    results: &mut Vec<(PathBuf, TestResult)>,
    tx: Sender<PathBuf>,
    args: &Args,
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
            walk_dir_recursive(test?.path(), results, tx.clone(), args)?;
        }
        Ok(())
    } else if path.is_file() {
        let mut content = std::fs::read_to_string(path)?;
        content.retain(|c| c != '\r');
        results.push((
            path.to_owned(),
            if filter_matches(&args.filter, path) {
                tx.send(path.to_owned()).unwrap();
                files::test_file(&content, path, args)
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
