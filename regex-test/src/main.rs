use std::{process::exit, str::FromStr};

use regex_test::{Outcome, RegexTest};

fn main() {
    let args = parse_args();

    let test = RegexTest::new();
    let result = match args.flavor {
        Flavor::Rust => test.test_rust_with(&args.input, &args.test),
        Flavor::Pcre => test.test_pcre_with(&args.input, &args.test),
        _ if !args.test.is_empty() => {
            error(&format!("The flavor {:?} doesn't support test cases yet!", args.flavor))
        }
        Flavor::Ruby => test.test_ruby(&args.input),
        Flavor::Js => test.test_js(&args.input),
        Flavor::Java => test.test_java(&args.input),
        Flavor::Python => test.test_python(&args.input),
        Flavor::DotNet => test.test_dotnet(&args.input),
    };

    match result {
        Outcome::Success => eprintln!("success!"),
        Outcome::Error(e) => error(&e),
    }
}

#[derive(Debug)]
enum Flavor {
    Pcre,
    Rust,
    Ruby,
    Js,
    Java,
    Python,
    DotNet,
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
            ".net" | "dotnet" => Flavor::DotNet,
            _ => return Err(()),
        })
    }
}

struct Args {
    flavor: Flavor,
    input: String,
    test: Vec<String>,
}

fn parse_args() -> Args {
    let mut raw = false;
    let mut input = None;
    let mut flavor = None;
    let mut test = vec![];

    let mut parts = std::env::args().skip(1);
    while let Some(part) = parts.next() {
        enum Arg<'a> {
            Help,
            Flavor(Option<&'a str>),
            Test(Option<&'a str>),
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
                Some("test") => Arg::Test(None),
                Some(s) => match s.strip_prefix("flavor=") {
                    Some(flavor) => Arg::Flavor(Some(flavor)),
                    None => error(&format!("Unknown flag '--{s}'")),
                },
                None => match part.strip_prefix('-') {
                    Some("h") => Arg::Help,
                    Some("f") => Arg::Flavor(None),
                    Some("t") => Arg::Test(None),
                    Some(s) => match s.as_bytes() {
                        [b'f', ..] => {
                            let flavor = &s[1..];
                            Arg::Flavor(Some(flavor.strip_prefix('=').unwrap_or(flavor)))
                        }
                        [b't', ..] => {
                            let test = &s[1..];
                            Arg::Test(Some(test.strip_prefix('=').unwrap_or(test)))
                        }
                        _ => error(&format!("Unknown flag '-{s}'")),
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
            Arg::Flavor(Some(part)) => set_flavor(part, &mut flavor),
            Arg::Test(None) => {
                let part = parts.next().unwrap_or_else(|| error("expected test string"));
                test.push(part);
            }
            Arg::Test(Some(part)) => test.push(part.into()),
            Arg::Input(part) => set_input(part, &mut input),
        }
    }

    let input = input.unwrap_or_else(|| error("no input provided"));
    let flavor = flavor.unwrap_or_else(|| error("No flavor provided"));
    Args { input, flavor, test }
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
