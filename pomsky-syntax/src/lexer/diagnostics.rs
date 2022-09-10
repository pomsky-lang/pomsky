//! Regex diagnostics. These are emitted when the syntax is valid in a regex, but
//! not in a pomsky expression.
//!
//! Regex diagnostics should contain all the information needed to convert the syntax to a
//! correct pomsky expression. This information is accumulated by the functions in this module.

use super::LexErrorMsg;

pub(super) fn get_parse_error_msg_help(msg: LexErrorMsg, slice: &str) -> Option<String> {
    Some(match msg {
        LexErrorMsg::GroupNonCapturing => "Non-capturing groups are just parentheses: `(...)`. \
            Capturing groups use the `:(...)` syntax."
            .into(),
        LexErrorMsg::GroupLookahead => "Lookahead uses the `>>` syntax. \
            For example, `>> 'bob'` matches if the position is followed by bob."
            .into(),
        LexErrorMsg::GroupLookaheadNeg => "Negative lookahead uses the `!>>` syntax. \
            For example, `!>> 'bob'` matches if the position is not followed by bob."
            .into(),
        LexErrorMsg::GroupLookbehind => "Lookbehind uses the `<<` syntax. \
            For example, `<< 'bob'` matches if the position is preceded with bob."
            .into(),
        LexErrorMsg::GroupLookbehindNeg => "Negative lookbehind uses the `!<<` syntax. \
            For example, `!<< 'bob'` matches if the position is not preceded with bob."
            .into(),
        LexErrorMsg::GroupComment => "Comments start with `#` and go until the \
            end of the line."
            .into(),
        LexErrorMsg::GroupNamedCapture => return get_named_capture_help(slice),
        LexErrorMsg::GroupPcreBackreference => return get_pcre_backreference_help(slice),
        LexErrorMsg::Backslash => return get_backslash_help(slice),
        LexErrorMsg::BackslashU4 => return get_backslash_help_u4(slice),
        LexErrorMsg::BackslashX2 => return get_backslash_help_x2(slice),
        LexErrorMsg::BackslashUnicode => return get_backslash_help_unicode(slice),
        LexErrorMsg::BackslashGK => return get_backslash_gk_help(slice),
        LexErrorMsg::BackslashProperty => return get_backslash_property_help(slice),

        LexErrorMsg::DeprStart => return Some("Use `^` instead".into()),
        LexErrorMsg::DeprEnd => return Some("Use `$` instead".into()),

        LexErrorMsg::GroupAtomic
        | LexErrorMsg::GroupConditional
        | LexErrorMsg::GroupBranchReset
        | LexErrorMsg::GroupSubroutineCall
        | LexErrorMsg::GroupOther
        | LexErrorMsg::UnclosedString => return None,
    })
}

fn get_named_capture_help(str: &str) -> Option<String> {
    // (?<name>), (?P<name>)
    let name =
        str.trim_start_matches("(?").trim_start_matches('P').trim_matches(&['<', '>', '\''][..]);

    if name.contains('-') {
        Some("Balancing groups are not supported".into())
    } else {
        Some(format!(
            "Named capturing groups use the `:name(...)` syntax. Try `:{name}(...)` instead"
        ))
    }
}

fn get_pcre_backreference_help(str: &str) -> Option<String> {
    // (?P=name)
    let name = str.trim_start_matches("(?P=").trim_end_matches(')');
    Some(format!("Backreferences use the `::name` syntax. Try `::{name}` instead"))
}

fn get_backslash_help(str: &str) -> Option<String> {
    assert!(str.starts_with('\\'));
    let str = &str[1..];
    let mut iter = str.chars();

    Some(match iter.next() {
        Some('b') => "Replace `\\b` with `%` to match a word boundary".into(),
        Some('B') => "Replace `\\B` with `!%` to match a place without a word boundary".into(),
        Some('A') => "Replace `\\A` with `Start` to match the start of the string".into(),
        Some('z') => "Replace `\\z` with `End` to match the end of the string".into(),
        Some('Z') => "\\Z is not supported. Use `End` to match the end of the string.\n\
            Note, however, that `End` doesn't match the position before the final newline."
            .into(),
        Some('N') => "Replace `\\N` with `![n]`".into(),
        Some('X') => "Replace `\\X` with `Grapheme`".into(),
        Some('R') => "Replace `\\R` with `([r] [n] | [v])`".into(),
        Some('D') => "Replace `\\D` with `[!d]`".into(),
        Some('W') => "Replace `\\W` with `[!w]`".into(),
        Some('S') => "Replace `\\S` with `[!s]`".into(),
        Some('V') => "Replace `\\V` with `![v]`".into(),
        Some('H') => "Replace `\\H` with `![h]`".into(),
        Some('G') => "Match attempt anchors are not supported".into(),
        Some(c @ ('a' | 'e' | 'f' | 'n' | 'r' | 't' | 'h' | 'v' | 'd' | 'w' | 's')) => {
            format!("Replace `\\{c}` with `[{c}]`")
        }
        Some('0') => "Replace `\\0` with `U+00`".into(),
        Some(c @ '1'..='7') => format!(
            "If this is a backreference, replace it with `::{c}`.\n\
            If this is an octal escape, replace it with `U+0{c}`."
        ),
        Some(c @ '1'..='9') => format!("Replace `\\{c}` with `::{c}`"),
        _ => return None,
    })
}

fn get_backslash_help_u4(str: &str) -> Option<String> {
    // \uFFFF
    let hex = &str[2..];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_x2(str: &str) -> Option<String> {
    // \xFF
    let hex = &str[2..];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_unicode(str: &str) -> Option<String> {
    // \u{...}, \x{...}
    let hex = str[2..].trim_matches(&['{', '}'][..]);
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_gk_help(str: &str) -> Option<String> {
    // \k<name>, \k'name', \k{name}, \k0, \k-1, \k+1,
    // \g<name>, \g'name', \g{name}, \g0, \g-1, \g+1
    let name = str[2..].trim_matches(&['{', '}', '<', '>', '\''][..]);

    if name == "0" {
        Some("Recursion is currently not supported".to_string())
    } else {
        Some(format!("Replace `{str}` with `::{name}`"))
    }
}

fn get_backslash_property_help(str: &str) -> Option<String> {
    // \pL, \PL, \p{Letter}, \P{Letter}, \p{^Letter}, \P{^Letter}
    let is_negative =
        (str.starts_with("\\P") && !str.starts_with("\\P{^")) || str.starts_with("\\p{^");
    let name = str[2..].trim_matches(&['{', '}', '^'][..]).replace(&['+', '-'][..], "_");

    if is_negative {
        Some(format!("Replace `{str}` with `[!{name}]`"))
    } else {
        Some(format!("Replace `{str}` with `[{name}]`"))
    }
}
