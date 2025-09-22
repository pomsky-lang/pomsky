use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

use diff::simple_diff;
use regex_test::RegexTest;

use crate::{
    args::Args,
    color::{Colored, D2, prelude::*},
    files::TestResult,
};

#[macro_use]
mod color;
mod args;
mod diff;
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
    let Ok(()) = child.join() else {
        std::process::exit(1);
    };
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
        println!("{}", yellow("including ignored cases!"));
    }

    let start = Instant::now();
    let rt = RegexTest::default();
    rt.init_processes();
    println!("test setup completed in {:.2?}", start.elapsed());
    println!(" - PCRE2 version: {}", regex_test::pcre_version());
    println!(" - Oniguruma version: {}", regex_test::onig_version());

    println!();
    let start = Instant::now();

    let results: Vec<TestResult> = samples
        .into_iter()
        .map(|(path, content)| files::test_file(content, path, &args, &rt))
        .collect();

    let elapsed = start.elapsed();
    println!();

    let mut ok: u32 = 0;
    let mut failed: u32 = 0;
    let mut blessed: u32 = 0;
    let mut ignored: u32 = 0;

    for (result, path) in results.into_iter().zip(paths) {
        match result {
            TestResult::Success => ok += 1,
            TestResult::Ignored => ignored += 1,
            TestResult::Blessed => blessed += 1,
            TestResult::IncorrectResult { input, expected, got } => {
                let l_ok = expected.is_ok();
                let r_ok = got.is_ok();
                let expected = expected.unwrap_or_else(|e| e);
                let got = got.unwrap_or_else(|e| e);
                let color;
                let (pre, l_diff, r_diff, suf) = if l_ok == r_ok {
                    color = true;
                    simple_diff(&expected, &got)
                } else {
                    color = false;
                    ("", &expected[..], &got[..], "")
                };

                failed += 1;
                println!("{}: {}", path.to_string_lossy(), red("incorrect result."));
                println!("       {}: {}", blue("input"), pad_left(&input, 14));
                println!(
                    "    {}: {}",
                    blue("expected"),
                    Print(l_ok, pre, green(l_diff).iff(color).bg(), suf, 14)
                );
                println!(
                    "         {}: {}",
                    blue("got"),
                    Print(r_ok, pre, red(r_diff).iff(color).bg(), suf, 14)
                );
                println!();
            }
            TestResult::InvalidOutput(e) => {
                failed += 1;
                println!("{}: {}", path.to_string_lossy(), red("invalid regex."));
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

    macro_rules! number_format {
        ($color:ident, $number:expr, $s:literal) => {
            (if $number > 0 { $color } else { no_color })(D2($number, $s))
        };
    }

    println!(
        "test result: {}. {}; {}; {}; {}; finished in {:.2?}\n",
        if failed == 0 { green("ok") } else { red("FAILED") },
        number_format!(green, ok, " passed"),
        if blessed > 0 {
            number_format!(yellow, blessed, " blessed")
        } else {
            number_format!(yellow, failed, " failed")
        },
        number_format!(yellow, ignored, " ignored"),
        number_format!(yellow, filtered, " filtered out"),
        elapsed,
    );

    if blessed > 0 {
        println!(
            "{t_warn}: Some failed tests were blessed. Check the git diff \
            to see if their result is correct\n",
            t_warn = yellow("warning"),
        );
    } else if failed > 0 {
        if args.filter.is_empty() {
            println!(
                "{t_tip}: you can rerun a specific test case with \
                `cargo test --test it -- {t_filter}`\n\
                where {t_filter} is a substring of the test case's file path\n",
                t_tip = yellow("help"),
                t_filter = blue("<filter>"),
            );
            println!(
                "{t_tip}: Automatically correct failed testcases with \
                `cargo test --test it -- {t_bless}`\n",
                t_tip = yellow("help"),
                t_bless = blue("--bless"),
            );
        }
    } else if ignored > 0 {
        println!(
            "{t_tip}: run ignored test cases with `cargo test --test it -- -i`",
            t_tip = yellow("help"),
        );
    }

    if failed > 0 {
        std::process::exit(1);
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
            if failed == 0 { green("ok") } else { red("FAILED") },
            number_format!(red, failed, " failed"),
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

struct Print<'a>(bool, &'a str, Colored<&'a str>, &'a str, usize);

impl fmt::Display for Print<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Print(is_ok, prefix, middle, suffix, pad) = *self;
        if is_ok {
            write!(
                f,
                "{} /{s}{m}{e}/",
                green("OK"),
                s = pad_left(prefix, pad + 4),
                m = middle.map(|middle| pad_left(middle, pad + 4)),
                e = pad_left(suffix, pad + 4)
            )
        } else {
            write!(
                f,
                "{}: {s}{m}{e}",
                red("ERR"),
                s = pad_left(prefix, pad + 5),
                m = middle.map(|middle| pad_left(middle, pad + 5)),
                e = pad_left(suffix, pad + 5)
            )
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
    filter_count: &mut u32,
) -> io::Result<()> {
    let path_ref = &path;
    if !path_ref.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file {:?} not found", blue(path_ref)),
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
        Err(io::Error::other(format!("unexpected file type of {:?}", blue(path_ref))))
    }
}
