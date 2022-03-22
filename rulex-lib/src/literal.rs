use crate::{compile::CompileResult, options::RegexFlavor, regex::Regex, span::Span};

#[derive(Clone, PartialEq, Eq)]
pub struct Literal<'i> {
    content: &'i str,
    pub(crate) span: Span,
}

impl<'i> Literal<'i> {
    pub(crate) fn new(content: &'i str, span: Span) -> Self {
        Literal { content, span }
    }

    pub(crate) fn compile(&self) -> CompileResult<'i> {
        Ok(Regex::Literal(self.content))
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Literal<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

/// Write a char to the output buffer with proper escaping. Assumes the char is inside a
/// character class.
pub(crate) fn compile_char_esc_in_class(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '-' => buf.push_str(r#"\-"#),
        ']' => buf.push_str(r#"\]"#),
        '^' => buf.push_str(r#"\^"#),
        c => compile_char(c, buf, flavor),
    }
}

/// Write a char to the output buffer with proper escaping. Assumes the char is not in a
/// character class.
pub(crate) fn codegen_char_esc(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '[' => buf.push_str(r#"\["#),
        '{' => buf.push_str(r#"\{"#),
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

/// Write a char to the output buffer. This escapes characters that are neither alphanumeric, nor
/// printable ASCII characters. It does _not_ escape characters like `(` or `]` that have a
/// special meaning.
pub(crate) fn compile_char(c: char, buf: &mut String, flavor: RegexFlavor) {
    use std::fmt::Write;

    match c {
        '\n' => buf.push_str("\\n"),
        '\r' => buf.push_str("\\r"),
        '\t' => buf.push_str("\\t"),
        '\x07' => buf.push_str("\\a"),
        '\x1B' => buf.push_str("\\e"),
        '\x0C' => buf.push_str("\\f"),
        ' ' => buf.push(' '),
        _ if c <= '\u{FF}' => {
            if c.is_ascii_graphic() {
                buf.push(c);
            } else {
                write!(buf, "\\x{:02X}", c as u8).unwrap();
            }
        }
        _ if c.is_alphanumeric() && c.len_utf16() == 1 => {
            buf.push(c);
        }
        _ if c.len_utf16() == 1 && !matches!(flavor, RegexFlavor::Pcre) => {
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

pub(super) fn needs_parens_before_repetition(s: &str) -> bool {
    s.chars().nth(1).is_some()
}
