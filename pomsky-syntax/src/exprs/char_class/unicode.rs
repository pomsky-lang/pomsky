use super::char_group::GroupName;
use crate::error::CharClassError;

include!(concat!(env!("OUT_DIR"), "/unicode_data.rs"));

pub(super) fn parse_group_name(name: &str) -> Result<GroupName, CharClassError> {
    match PARSE_LUT.binary_search_by_key(&name, |(k, _)| k) {
        Ok(n) => Ok(PARSE_LUT[n].1),
        Err(_) => Err(CharClassError::UnknownNamedClass {
            found: name.into(),
            #[cfg(feature = "suggestions")]
            similar: crate::util::find_suggestion(name, PARSE_LUT.iter().map(|&(name, _)| name)),
        }),
    }
}

pub fn blocks_supported_in_dotnet() -> &'static [&'static str] {
    DOTNET_SUPPORTED
}

/// Returns the list of all accepted shorthands.
pub fn list_shorthands() -> impl Iterator<Item = (&'static str, GroupName)> {
    PARSE_LUT.iter().copied()
}
