use std::ffi::OsString;

use super::ParseArgsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegexEngine {
    Pcre2,
}

impl RegexEngine {
    pub(crate) fn parse(value: OsString) -> Result<Self, ParseArgsError> {
        let lower = value.to_string_lossy().to_ascii_lowercase();
        Ok(match lower.as_str() {
            "pcre2" => RegexEngine::Pcre2,
            _ => return Err(ParseArgsError::UnknownEngine(lower)),
        })
    }
}
