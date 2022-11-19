use pomsky_syntax::exprs::Literal;

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Literal<'i> {
    fn compile<'c>(&'c self, _: CompileOptions, _: &mut CompileState<'c, 'i>) -> CompileResult<'i> {
        Ok(Regex::Literal(self.content.clone()))
    }
}

/// Write a char to the output buffer with proper escaping. Assumes the char is
/// inside a character class.
pub(crate) fn compile_char_esc_in_class(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '-' => buf.push_str(r#"\-"#),
        '[' => buf.push_str(r#"\["#),
        ']' => buf.push_str(r#"\]"#),
        '^' => buf.push_str(r#"\^"#),
        '&' if flavor != RegexFlavor::JavaScript => buf.push_str(r#"\&"#),
        '|' if flavor != RegexFlavor::JavaScript => buf.push_str(r#"\|"#),
        c => compile_char(c, buf, flavor),
    }
}

/// Write a char to the output buffer with proper escaping. Assumes the char is
/// not in a character class.
pub(crate) fn codegen_char_esc(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '[' => buf.push_str(r#"\["#),
        '{' => buf.push_str(r#"\{"#),
        '}' => buf.push_str(r#"\}"#),
        '(' => buf.push_str(r#"\("#),
        ')' => buf.push_str(r#"\)"#),
        '.' => buf.push_str(r#"\."#),
        '+' => buf.push_str(r#"\+"#),
        '*' => buf.push_str(r#"\*"#),
        '?' => buf.push_str(r#"\?"#),
        '|' => buf.push_str(r#"\|"#),
        '^' => buf.push_str(r#"\^"#),
        '$' => buf.push_str(r#"\$"#),
        c => compile_char(c, buf, flavor),
    }
}

/// Write a char to the output buffer. This escapes characters that are neither
/// alphanumeric, nor printable ASCII characters. It does _not_ escape
/// characters like `(` or `]` that have a special meaning.
pub(crate) fn compile_char(c: char, buf: &mut String, flavor: RegexFlavor) {
    use std::fmt::Write;

    match c {
        '\n' => buf.push_str("\\n"),
        '\r' => buf.push_str("\\r"),
        '\t' => buf.push_str("\\t"),
        '\x07' => buf.push_str("\\a"),
        '\x0C' => buf.push_str("\\f"),
        // not supported in Rust:
        // '\x1B' => buf.push_str("\\e"),
        ' ' => buf.push(' '),
        _ if c.is_ascii() => {
            if c.is_ascii_graphic() {
                buf.push(c);
            } else {
                write!(buf, "\\x{:02X}", c as u8).unwrap();
            }
        }
        _ if c.is_alphanumeric() && c.len_utf16() == 1 => {
            buf.push(c);
        }
        _ if c as u32 <= 0xFF => {
            write!(buf, "\\x{:02X}", c as u32).unwrap();
        }
        _ if c as u32 <= 0xFFFF && !matches!(flavor, RegexFlavor::Pcre) => {
            write!(buf, "\\u{:04X}", c as u32).unwrap();
        }
        _ => {
            match flavor {
                RegexFlavor::Pcre => buf.push_str("\\x"),
                _ => buf.push_str("\\u"),
            }
            write!(buf, "{{{:X}}}", c as u32).unwrap();
        }
    }
}

pub(crate) fn needs_parens_before_repetition(s: &str) -> bool {
    s.is_empty() || s.chars().nth(1).is_some()
}
