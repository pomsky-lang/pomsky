use std::ffi::OsString;

use pomsky::options::RegexFlavor;

use super::ParseArgsError;

pub(super) fn parse_flavor(value: OsString) -> Result<RegexFlavor, ParseArgsError> {
    let lower = value.to_string_lossy().to_ascii_lowercase();
    Ok(match lower.as_str() {
        "pcre" => RegexFlavor::Pcre,
        "python" => RegexFlavor::Python,
        "java" => RegexFlavor::Java,
        "js" | "javascript" => RegexFlavor::JavaScript,
        "dotnet" | ".net" => RegexFlavor::DotNet,
        "ruby" => RegexFlavor::Ruby,
        "rust" => RegexFlavor::Rust,
        "re2" => RegexFlavor::RE2,
        _ => return Err(ParseArgsError::UnknownFlavor(lower)),
    })
}
