use std::collections::HashMap;

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
            compile_char(c, buf);
        }
        Ok(())
    }
}

pub(crate) fn compile_char(c: char, buf: &mut String) {
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
        c => buf.push(c),
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
