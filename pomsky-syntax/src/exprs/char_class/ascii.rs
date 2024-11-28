use crate::error::CharClassError;

use super::GroupItem;

pub(super) fn parse_ascii_group(
    name: &str,
    negative: bool,
) -> Result<Vec<GroupItem>, CharClassError> {
    if negative {
        return Err(CharClassError::Negative);
    }

    Ok(match name {
        "ascii_alpha" => {
            vec![GroupItem::range_unchecked('a', 'z'), GroupItem::range_unchecked('A', 'Z')]
        }
        "ascii_alnum" => vec![
            GroupItem::range_unchecked('0', '9'),
            GroupItem::range_unchecked('a', 'z'),
            GroupItem::range_unchecked('A', 'Z'),
        ],
        "ascii" => vec![GroupItem::range_unchecked('\0', '\x7F')],
        "ascii_blank" => vec![GroupItem::Char(' '), GroupItem::Char('\t')],
        "ascii_cntrl" => vec![GroupItem::range_unchecked('\0', '\x1F'), GroupItem::Char('\x7F')],
        "ascii_digit" => vec![GroupItem::range_unchecked('0', '9')],
        "ascii_graph" => vec![GroupItem::range_unchecked('!', '~')],
        "ascii_lower" => vec![GroupItem::range_unchecked('a', 'z')],
        "ascii_print" => vec![GroupItem::range_unchecked(' ', '~')],
        "ascii_punct" => vec![
            GroupItem::range_unchecked('!', '/'),
            GroupItem::range_unchecked(':', '@'),
            GroupItem::range_unchecked('[', '`'),
            GroupItem::range_unchecked('{', '~'),
        ],
        "ascii_space" => vec![GroupItem::range_unchecked('\t', '\r'), GroupItem::Char(' ')],
        "ascii_upper" => vec![GroupItem::range_unchecked('A', 'Z')],
        "ascii_word" => vec![
            GroupItem::range_unchecked('0', '9'),
            GroupItem::range_unchecked('a', 'z'),
            GroupItem::range_unchecked('A', 'Z'),
            GroupItem::Char('_'),
        ],
        "ascii_xdigit" => vec![
            GroupItem::range_unchecked('0', '9'),
            GroupItem::range_unchecked('a', 'f'),
            GroupItem::range_unchecked('A', 'F'),
        ],
        _ => {
            return Err(CharClassError::UnknownNamedClass {
                found: name.into(),
                extra_in_prefix: false,
                #[cfg(feature = "suggestions")]
                similar: crate::util::find_suggestion(name, OPTION_LIST.iter().copied()),
            })
        }
    })
}

#[cfg(feature = "suggestions")]
const OPTION_LIST: &[&str] = &[
    "ascii_alpha",
    "ascii_alnum",
    "ascii",
    "ascii_blank",
    "ascii_cntrl",
    "ascii_digit",
    "ascii_graph",
    "ascii_lower",
    "ascii_print",
    "ascii_punct",
    "ascii_space",
    "ascii_upper",
    "ascii_word",
    "ascii_xdigit",
];
