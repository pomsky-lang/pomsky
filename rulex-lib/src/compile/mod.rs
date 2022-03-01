use std::collections::HashMap;
use std::fmt::Write;

use crate::{
    error::CompileError,
    options::{CompileOptions, RegexFlavor},
};

pub(crate) type CompileResult = Result<(), CompileError>;

pub(crate) trait Compile {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult;
}

impl Compile for &'_ str {
    fn comp(
        &self,
        options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        for c in self.chars() {
            compile_char_esc(c, buf, options.flavor);
        }
        Ok(())
    }
}

/// Write a char to the output buffer with proper escaping. Assumes the char is not in a
/// character class.
pub(crate) fn compile_char_esc(c: char, buf: &mut String, flavor: RegexFlavor) {
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
    match c {
        '\n' => buf.push_str("\\n"),
        '\r' => buf.push_str("\\r"),
        '\t' => buf.push_str("\\t"),
        '\x07' => buf.push_str("\\a"),
        '\x1B' => buf.push_str("\\e"),
        '\x0C' => buf.push_str("\\f"),
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
        _ => {
            match flavor {
                RegexFlavor::Pcre => buf.push_str("\\x"),
                _ => buf.push_str("\\u"),
            }
            if c.len_utf16() == 1 {
                write!(buf, "{:X}", c as u32).unwrap();
            } else {
                write!(buf, "{{{:X}}}", c as u32).unwrap();
            }
        }
    }
}

pub(crate) struct Parens<'a, T>(pub(crate) &'a T);

impl<T: Compile> Compile for Parens<'_, T> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        buf.push_str("(?:");
        self.0.comp(options, state, buf)?;
        buf.push(')');
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompileState {
    pub(crate) next_idx: u32,
    pub(crate) used_names: HashMap<String, u32>,
}

impl CompileState {
    pub(crate) fn new() -> Self {
        CompileState {
            next_idx: 1,
            used_names: HashMap::new(),
        }
    }
}
