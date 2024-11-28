use super::char_group::{GroupName, ScriptExtension};
use crate::error::CharClassError;

include!(concat!(env!("OUT_DIR"), "/unicode_data.rs"));

pub(super) fn parse_group_name(
    kind: Option<&str>,
    name: &str,
) -> Result<GroupName, CharClassError> {
    let mut name_str;
    let mut name = name;
    if let Some("blk" | "block") = kind {
        name_str = String::with_capacity(name.len() + 2);
        name_str.push_str("In");
        name_str.push_str(name);
        name = &name_str;
    }

    let mut res = match PARSE_LUT.binary_search_by_key(&name, |(k, _)| k) {
        Ok(n) => PARSE_LUT[n].1,
        Err(_) => {
            return Err(CharClassError::UnknownNamedClass {
                found: name.into(),
                #[cfg(feature = "suggestions")]
                similar: crate::util::find_suggestion(
                    name,
                    PARSE_LUT.iter().map(|&(name, _)| name),
                ),
            })
        }
    };
    if let Some(kind) = kind {
        match &mut res {
            GroupName::Word
            | GroupName::Digit
            | GroupName::Space
            | GroupName::HorizSpace
            | GroupName::VertSpace
            | GroupName::OtherProperties(_) => return Err(CharClassError::UnexpectedPrefix),
            GroupName::Category(_) => {
                if kind != "gc" && kind != "general_category" {
                    return Err(CharClassError::WrongPrefix {
                        expected: "general_category (gc)",
                        has_in_prefix: false,
                    });
                }
            }
            GroupName::Script(_, is_extension) => {
                if kind == "sc" || kind == "script" {
                    *is_extension = ScriptExtension::No;
                } else if kind == "scx" || kind == "script_extensions" {
                    *is_extension = ScriptExtension::Yes;
                } else {
                    return Err(CharClassError::WrongPrefix {
                        expected: "script_extensions (scx) or script (sc)",
                        has_in_prefix: false,
                    });
                }
            }
            GroupName::CodeBlock(_) => {
                if kind != "blk" && kind != "block" {
                    return Err(CharClassError::WrongPrefix {
                        expected: "block (blk)",
                        has_in_prefix: true,
                    });
                }
            }
        }
    }

    Ok(res)
}

pub fn blocks_supported_in_dotnet() -> &'static [&'static str] {
    DOTNET_SUPPORTED
}

/// Returns the list of all accepted shorthands.
pub fn list_shorthands() -> impl Iterator<Item = (&'static str, GroupName)> {
    PARSE_LUT.iter().copied()
}
