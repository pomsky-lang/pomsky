use std::{io::Read, path::PathBuf, string::FromUtf8Error};

use atty::Stream;
use pomsky::options::RegexFlavor;

pub(super) enum ParseArgsError {
    Lexopt(lexopt::Error),
    StdinUtf8(FromUtf8Error),
    Other(String),
}

impl ParseArgsError {
    fn unexpected_twice(s: &str) -> Self {
        ParseArgsError::Other(format!(
            "The argument '{s}' was provided more than once, \
            but cannot be used multiple times"
        ))
    }
}

impl From<lexopt::Error> for ParseArgsError {
    fn from(e: lexopt::Error) -> Self {
        ParseArgsError::Lexopt(e)
    }
}

/// Compile a Pomsky expression to a regex
#[derive(Debug)]
pub(crate) struct Args {
    /// Pomsky expression to compile
    pub(crate) input: Input,
    /// Show debug information
    pub(crate) debug: bool,
    /// Regex flavor
    pub(crate) flavor: Option<RegexFlavor>,
    /// Does not print a new-line at the end of the compiled regular expression
    pub(crate) no_new_line: bool,
}

/// Compile a Pomsky expression to a regex
#[derive(Debug)]
pub(crate) enum Input {
    Value(String),
    File(PathBuf),
}

pub(super) fn get_short_usage_and_help() -> &'static str {
    "\
USAGE:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>

For more information try '--help'"
}

fn get_help() -> &'static str {
    concat!(
        "pomksy ",
        env!("CARGO_PKG_VERSION"),
        "
Compile pomsky expressions, a new regular expression language

USAGE:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>

ARGS:
    <INPUT>    Pomsky expression to compile

OPTIONS:
    -d, --debug              Show debug information
    -f, --flavor <FLAVOR>    Regex flavor [possible values: pcre, python, java,
                             javascript, dotnet, ruby, rust]
    -h, --help               Print help information
    -n, --no-new-line        Does not print a new-line at the end of the
                             compiled regular expression
    -p, --path <FILE>        File containing the pomsky expression to compile
    -V, --version            Print version information",
    )
}

fn get_version() -> &'static str {
    concat!("pomsky ", env!("CARGO_PKG_VERSION"))
}

pub(super) fn parse_args() -> Result<Args, ParseArgsError> {
    use lexopt::prelude::*;

    let mut input_value = None;
    let mut path = None;
    let mut debug = false;
    let mut flavor = None;
    let mut no_new_line = false;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('p') | Long("path") => {
                if path.is_some() {
                    return Err(ParseArgsError::unexpected_twice("--path"));
                }
                path = Some(parser.value()?.parse()?);
            }
            Short('d') | Long("debug") => {
                if debug {
                    return Err(ParseArgsError::unexpected_twice("--debug"));
                }
                debug = true;
            }
            Short('f') | Long("flavor") => {
                if flavor.is_some() {
                    return Err(ParseArgsError::unexpected_twice("--flavor"));
                }
                let value = parser.value()?;
                let lower = value.to_string_lossy().to_ascii_lowercase();
                flavor = Some(match lower.as_str() {
                    "pcre" => RegexFlavor::Pcre,
                    "python" => RegexFlavor::Python,
                    "java" => RegexFlavor::Java,
                    "js" | "javascript" => RegexFlavor::JavaScript,
                    "dotnet" | ".net" => RegexFlavor::DotNet,
                    "ruby" => RegexFlavor::Ruby,
                    "rust" => RegexFlavor::Rust,
                    _ => {
                        return Err(ParseArgsError::Other(format!(
                            "{value:?} isn't a valid value for '--flavor <FLAVOR>'\n\
                            [possible values: pcre, python, java, javascript, dotnet, ruby, rust]"
                        )))
                    }
                });
            }
            Short('n') | Long("no-new-line") => {
                if no_new_line {
                    return Err(ParseArgsError::unexpected_twice("no-new-line"));
                }
                no_new_line = true;
            }
            Value(val) if input_value.is_none() => {
                input_value = Some(val.into_string().map_err(lexopt::Error::from)?);
            }
            Short('h') | Long("help") => {
                println!("{}", get_help());
                std::process::exit(0);
            }
            Short('V') | Long("version") => {
                println!("{}", get_version());
                std::process::exit(0);
            }
            _ => Err(arg.unexpected())?,
        }
    }

    let input = match (input_value, path) {
        (Some(input), None) => Input::Value(input),
        (None, Some(path)) => Input::File(path),
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => Input::Value(input),
                Err(e) => return Err(ParseArgsError::StdinUtf8(e)),
            }
        }
        (Some(_), Some(_)) => {
            return Err(ParseArgsError::Other(
                "You can only provide an input or a path, but not both".into(),
            ))
        }
        (None, None) => return Err(ParseArgsError::Other("No input provided".into())),
    };

    Ok(Args { input, flavor, debug, no_new_line })
}
