use std::{io::Read, path::PathBuf};

use atty::Stream;

use super::ParseArgsError;

/// Compile a Pomsky expression to a regex
#[derive(Debug, PartialEq)]
pub(crate) enum Input {
    Value(String),
    File(PathBuf),
}

impl Input {
    pub(crate) fn read_stdin() -> Result<Self, ParseArgsError> {
        if atty::isnt(Stream::Stdin) {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => Ok(Input::Value(input)),
                Err(e) => Err(ParseArgsError::StdinUtf8(e)),
            }
        } else {
            Err(ParseArgsError::NoInput)
        }
    }
}
