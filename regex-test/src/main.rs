use std::{process::exit, str::FromStr};

use regex_test::{Outcome, RegexTest};

fn main() {
    let test = RegexTest::new();
    let res = main_impl(&test);

    test.java.kill().unwrap();
    test.js.kill().unwrap();
    test.py.kill().unwrap();
    test.dotnet.kill().unwrap();

    if let Err(e) = res {
        println!("error: {e}");
        std::process::exit(1);
    }
}

fn main_impl(test: &RegexTest) -> Result<(), String> {
    let args = parse_args()?;
    let result = match args.flavor {
        Flavor::Rust => test.test_rust_with(&args.input, &args.test),
        Flavor::Pcre => test.test_pcre_with(&args.input, &args.test),
        Flavor::Ruby => test.test_ruby_with(&args.input, &args.test),
        Flavor::DotNet => test.test_dotnet_with(&args.input, &args.test),
        Flavor::Js => test.test_js_with(&args.input, &args.test),
        Flavor::Java => test.test_java_with(&args.input, &args.test),
        Flavor::Python => test.test_python_with(&args.input, &args.test),
        #[cfg(feature = "re2")]
        Flavor::RE2 => test.test_re2_with(&args.input, &args.test),
    };

    match result {
        Outcome::Success => eprintln!("success!"),
        Outcome::Error(e) => return Err(e),
    }

    Ok(())
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
    #[cfg(feature = "re2")]
    RE2,
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
            #[cfg(feature = "re2")]
            "re2" => Flavor::RE2,
            _ => return Err(()),
        })
    }
}

struct Args {
    flavor: Flavor,
    input: String,
    test: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
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
                    None => return Err(format!("Unknown flag '--{s}'")),
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
                        _ => return Err(format!("Unknown flag '-{s}'")),
                    },
                    None => Arg::Input(part),
                },
            }
        };

        match arg {
            Arg::Help => help(),
            Arg::Flavor(None) => {
                let Some(part) = parts.next() else {
                    return Err("expected regex flavor".to_string());
                };
                set_flavor(&part, &mut flavor)?;
            }
            Arg::Flavor(Some(part)) => set_flavor(part, &mut flavor)?,
            Arg::Test(None) => {
                let Some(part) = parts.next() else {
                    return Err("expected test string".to_string());
                };
                test.push(part);
            }
            Arg::Test(Some(part)) => test.push(part.into()),
            Arg::Input(part) => set_input(part, &mut input)?,
        }
    }

    let Some(input) = input else {
        return Err("no input provided".to_string());
    };
    let Some(flavor) = flavor else {
        return Err("No flavor provided".to_string());
    };
    Ok(Args { input, flavor, test })
}

fn help() {
    println!(
        "Test if something is a valid regex

USAGE:
    regex-test -f <FLAVOR> INPUT [-t TEST]...

FLAVORS:
    pcre, rust, ruby, js, java, python"
    );
    exit(0);
}

fn set_input(part: String, input: &mut Option<String>) -> Result<(), String> {
    if input.is_some() {
        return Err("input provided multiple times".to_string());
    }
    *input = Some(part);
    Ok(())
}

fn set_flavor(part: &str, flavor: &mut Option<Flavor>) -> Result<(), String> {
    if flavor.is_some() {
        return Err("flavor provided multiple times".to_string());
    }
    let Ok(parsed) = part.parse() else {
        return Err("not a valid regex flavor".to_string());
    };
    *flavor = Some(parsed);
    Ok(())
}
