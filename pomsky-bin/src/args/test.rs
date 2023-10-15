use std::ffi::OsString;

use super::ParseArgsError;

#[derive(PartialEq)]
pub(crate) enum TestSettings {
    Pcre2,
}

impl TestSettings {
    pub(crate) fn parse(value: OsString) -> Result<Self, ParseArgsError> {
        let value = value.to_string_lossy().to_ascii_lowercase();
        match value.as_str() {
            "pcre2" | "pcre" => Ok(TestSettings::Pcre2),
            _ => Err(ParseArgsError::UnknownTestEngine(value)),
        }
    }
}
