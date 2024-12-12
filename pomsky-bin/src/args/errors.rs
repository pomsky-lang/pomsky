use std::{fmt, string::FromUtf8Error};

#[derive(Debug)]
pub(crate) enum ParseArgsError {
    Lexopt(lexopt::Error),
    StdinUtf8(FromUtf8Error),
    UnexpectedTwice(&'static str),
    NoInput,
    InputAndPath,
    UnknownFlavor(String),
    UnknownEngine(String),
    UnknownList(String),
    WarningsNoEquals(String),
    WarningsNoZero(String),
    WarningsNotAllowed(String),
    Other(String),
}

impl From<lexopt::Error> for ParseArgsError {
    fn from(e: lexopt::Error) -> Self {
        ParseArgsError::Lexopt(e)
    }
}

impl fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseArgsError::Lexopt(error) => write!(f, "{error}"),
            ParseArgsError::StdinUtf8(e) => write!(f, "Could not parse stdin: {e}"),
            ParseArgsError::UnexpectedTwice(option) => write!(
                f,
                "The argument '{option}' was provided more than once, \
                    but cannot be used multiple times"
            ),
            ParseArgsError::UnknownFlavor(flavor) => write!(
                f,
                "'{flavor}' isn't a valid flavor\n\
                    possible values: pcre, python, java, javascript, dotnet, ruby, rust"
            ),
            ParseArgsError::UnknownEngine(engine) => {
                write!(f, "'{engine}' isn't a valid regex engine\npossible values: pcre2")
            }
            ParseArgsError::UnknownList(list) => {
                write!(f, "'{list}' isn't a valid list\npossible values: shorthands")
            }
            ParseArgsError::NoInput => write!(f, "No input provided"),
            ParseArgsError::InputAndPath => {
                write!(f, "You can only provide an input or a path, but not both")
            }
            ParseArgsError::WarningsNoEquals(warning) => write!(
                f,
                "'{warning}' contains no '='\n\
                    try '-W{warning}=0' to disable {warning} warnings"
            ),
            ParseArgsError::WarningsNoZero(warning) => {
                write!(f, "warnings can only be disabled, try '-W{warning}=0'")
            }
            ParseArgsError::WarningsNotAllowed(warning) => {
                write!(f, "`{warning}` diagnostic kind cannot be disabled")
            }
            ParseArgsError::Other(msg) => f.write_str(msg),
        }
    }
}
