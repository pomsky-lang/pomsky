use std::{process::exit, str::FromStr};

use regex_test::{Outcome, RegexTest};

fn main() {
    let args = parse_args();

    let test = RegexTest::new();
    let result = match args.flavor {
        Flavor::Pcre => test.test_pcre(&args.input),
        Flavor::Rust => test.test_rust(&args.input),
        Flavor::Ruby => test.test_ruby(&args.input),
        Flavor::Js => test.test_js(&args.input),
        Flavor::Java => test.test_java(&args.input),
        Flavor::Python => test.test_python(&args.input),
    };

    match result {
        Outcome::Success => eprintln!("success!"),
        Outcome::Error(e) => error(&e),
    }
}

enum Flavor {
    Pcre,
    Rust,
    Ruby,
    Js,
    Java,
    Python,
}

impl FromStr for Flavor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        Ok(match s.as_str() {
            "pcre" => Flavor::Pcre,
            "rust" => Flavor::Rust,
            "ruby" => Flavor::Ruby,
            "js" | "javascript" => Flavor::Js,
            "java" => Flavor::Java,
            "py" | "python" => Flavor::Python,
            _ => return Err(()),
        })
    }
}

struct Args {
    flavor: Flavor,
    input: String,
}

fn parse_args() -> Args {
    let mut raw = false;
    let mut input = None;
    let mut flavor = None;

    let mut parts = std::env::args().skip(1);
    while let Some(part) = parts.next() {
        enum Arg<'a> {
            Help,
            Flavor(Option<&'a str>),
            Input(String),
        }

        let arg = if raw {
            Arg::Input(part)
        } else if part == "--" {
            raw = true;
            continue;
        } else {
            match part.strip_prefix("--") {
                Some("help") => Arg::Help,
                Some("flavor") => Arg::Flavor(None),
                Some(s) => match s.strip_prefix("flavor=") {
                    Some(flavor) => Arg::Flavor(Some(flavor)),
                    None => error(&format!("Unknown flag '--{s}'")),
                },
                None => match part.strip_prefix('-') {
                    Some("h") => Arg::Help,
                    Some("f") => Arg::Flavor(None),
                    Some(s) => match s.strip_prefix('f') {
                        Some(flavor) => {
                            Arg::Flavor(Some(flavor.strip_prefix('=').unwrap_or(flavor)))
                        }
                        None => error(&format!("Unknown flag '-{s}'")),
                    },
                    None => Arg::Input(part),
                },
            }
        };

        match arg {
            Arg::Help => help(),
            Arg::Flavor(None) => {
                let part = parts.next().unwrap_or_else(|| error("expected regex flavor"));
                set_flavor(&part, &mut flavor);
            }
            Arg::Flavor(Some(part)) => {
                set_flavor(part, &mut flavor);
            }
            Arg::Input(part) => set_input(part, &mut input),
        }
    }

    let input = input.unwrap_or_else(|| error("no input provided"));
    let flavor = flavor.unwrap_or_else(|| error("No flavor provided"));
    Args { input, flavor }
}

fn error(msg: &str) -> ! {
    println!("error: {msg}");
    exit(1)
}

fn help() {
    println!(
        "Test if something is a valid regex

USAGE:
    regex-test -f <FLAVOR> [INPUT]

FLAVORS:
    pcre, rust, ruby, js, java, python"
    );
    exit(0);
}

fn set_input(part: String, input: &mut Option<String>) {
    if input.is_some() {
        error("input provided multiple times");
    }
    *input = Some(part);
}

fn set_flavor(part: &str, flavor: &mut Option<Flavor>) {
    if flavor.is_some() {
        error("flavor provided multiple times");
    }
    *flavor = Some(part.parse().unwrap_or_else(|_| error("not a valid regex flavor")));
}
