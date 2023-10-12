use std::{
    io::{IsTerminal, Read},
    path::PathBuf,
};

use super::ParseArgsError;

/// Compile a Pomsky expression to a regex
#[derive(Debug, PartialEq)]
pub(crate) enum Input {
    Value(String),
    File(PathBuf),
}

impl Input {
    pub(crate) fn read_stdin() -> Result<Self, ParseArgsError> {
        let mut stdin = std::io::stdin();
        if !stdin.is_terminal() {
            let mut buf = Vec::new();
            stdin.read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => Ok(Input::Value(input)),
                Err(e) => Err(ParseArgsError::StdinUtf8(e)),
            }
        } else {
            Err(ParseArgsError::NoInput)
        }
    }
}
