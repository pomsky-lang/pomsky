use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

use regex_test::RegexTest;

use crate::{args::Args, color::Color::*, files::TestResult};

#[macro_use]
mod color;
mod args;
mod files;
mod fuzzer;

pub fn main() {
    let child = thread::Builder::new()
        .name("pomsky-it".into())
        // a large stack is required in debug builds
        .stack_size(8 * 1024 * 1024)
        .spawn(defer_main)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

fn defer_main() {
    println!("\nrunning integration tests");

    let args = Args::parse();

    let mut filtered = 0;
    let mut samples = vec![];
    collect_samples("./tests/testcases".into(), &mut samples, &args.filter, &mut filtered).unwrap();
    let paths = samples.iter().map(|(path, _)| path.to_owned()).collect::<Vec<_>>();

    println!("{} test cases found", samples.len());

    if args.include_ignored {
        println!("{}", Yellow("including ignored cases!"));
    }

    let start = Instant::now();
    let rt = RegexTest::default();
    rt.init_processes();
    println!("test setup completed in {:.2?}", start.elapsed());

    println!();
    let start = Instant::now();

    let results: Vec<TestResult> = samples
        .into_iter()
        .map(|(path, content)| files::test_file(content, path, &args, &rt))
        .collect();

    let elapsed = start.elapsed();
    println!();

    let mut ok = 0;
    let mut failed = 0;
    let mut blessed = 0;
    let mut ignored = 0;

    for (result, path) in results.into_iter().zip(paths) {
        match result {
            TestResult::Success => ok += 1,
            TestResult::Ignored => ignored += 1,
            TestResult::Blessed => blessed += 1,
            TestResult::IncorrectResult { input, expected, got } => {
                failed += 1;
                println!("{}: {}", path.to_string_lossy(), Red("incorrect result."));
                println!("       {}: {}", Blue("input"), pad_left(&input, 14));
                println!("    {}: {}", Blue("expected"), Print(expected, 14));
                println!("         {}: {}", Blue("got"), Print(got, 14));
                println!();
            }
            TestResult::InvalidOutput(e) => {
                failed += 1;
                println!("{}: {}", path.to_string_lossy(), Red("invalid regex."));
                println!("{e}");
                println!();
            }
        }
    }

    if args.stats {
        eprintln!("Stats");
        eprintln!("  Java   was invoked {} times", rt.java.get_count());
        eprintln!("  JS     was invoked {} times", rt.js.get_count());
        eprintln!("  Python was invoked {} times", rt.py.get_count());
        eprintln!("  Ruby   was invoked {} times", rt.ruby.get_count());
        eprintln!("  Rust   was invoked {} times", rt.rust.get_count());
        eprintln!("  PCRE   was invoked {} times", rt.pcre.get_count());

        #[cfg(target_os = "linux")]
        eprintln!("  .NET   was invoked {} times", rt.dotnet.get_count());
    }

    rt.kill_processes().unwrap();

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
                t_tip = Yellow("help"),
                t_filter = Blue("<filter>"),
            );
            println!(
                "{t_tip}: Automatically correct failed testcases with \
                `cargo test --test it -- {t_bless}`\n",
                t_tip = Yellow("help"),
                t_bless = Blue("--bless"),
            );
        }
    } else if ignored > 0 {
        println!(
            "{t_tip}: run ignored test cases with `cargo test --test it -- -i`",
            t_tip = Yellow("help"),
        );
    }

    if failed > 0 {
        std::process::exit(failed);
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
}

fn filter_matches(filter: &str, path: &Path) -> bool {
    if filter.is_empty() {
        return true;
    }
    let path = path.to_string_lossy();
    path.contains(filter)
}

struct Print(Result<String, String>, usize);

impl fmt::Display for Print {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Ok(s) => write!(f, "{} /{s}/", Green("OK"), s = pad_left(s, self.1 + 4)),
            Err(s) => write!(f, "{}: {s}", Red("ERR"), s = pad_left(s, self.1 + 5)),
        }
    }
}

fn pad_left(s: &str, padding: usize) -> String {
    s.lines()
        .enumerate()
        .map(|(i, line)| if i == 0 { line.to_string() } else { format!("\n{:padding$}{line}", "") })
        .collect()
}

fn collect_samples(
    path: PathBuf,
    buf: &mut Vec<(PathBuf, String)>,
    filter: &str,
    filter_count: &mut u64,
) -> io::Result<()> {
    let path_ref = &path;
    if !path_ref.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file {:?} not found", Blue(path_ref)),
        ));
    }

    if path_ref.is_dir() {
        for test in fs::read_dir(path_ref)? {
            collect_samples(test?.path(), buf, filter, filter_count)?;
        }
        Ok(())
    } else if path_ref.is_file() {
        let mut content = std::fs::read_to_string(path_ref)?;
        content.retain(|c| c != '\r');

        if filter_matches(filter, path_ref) {
            buf.push((path, content));
        } else {
            *filter_count += 1;
        }
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("unexpected file type of {:?}", Blue(path_ref)),
        ))
    }
}
