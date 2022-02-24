use std::collections::HashMap;
use std::fmt::Write;

use crate::{error::CompileError, options::CompileOptions};

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
        _options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        for c in self.chars() {
            compile_char_escaped(c, buf);
        }
        Ok(())
    }
}

pub(crate) fn compile_char_escaped(c: char, buf: &mut String) {
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
        c => compile_char(c, buf),
    }
}

pub(crate) fn compile_char(c: char, buf: &mut String) {
    match c {
        '\t' => buf.push_str("\\t"),
        '\r' => buf.push_str("\\r"),
        '\n' => buf.push_str("\\n"),
        '\x07' => buf.push_str("\\a"),
        '\x1B' => buf.push_str("\\e"),
        '\x0C' => buf.push_str("\\f"),
        ' ' => buf.push(' '),
        c if c.len_utf16() == 2 => {
            write!(buf, "\\u{{{:X}}}", c as u32).unwrap();
        }
        c if c.is_ascii_graphic() || c.is_alphanumeric() => buf.push(c),
        c if c.is_ascii() => {
            write!(buf, "\\x{:02X}", c as u8).unwrap();
        }
        c => {
            write!(buf, "\\u{:04X}", c as u32).unwrap();
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
