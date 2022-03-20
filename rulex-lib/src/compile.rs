use std::collections::HashMap;

use crate::{
    error::{CompileError, CompileErrorKind},
    options::CompileOptions,
    span::Span,
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

pub(crate) trait Transform {
    fn transform(&mut self, options: CompileOptions, state: &mut TransformState) -> CompileResult;
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

    pub(crate) unknown_references: Vec<(String, Span)>,
    pub(crate) unknown_groups: Vec<(u32, Span)>,
}

impl CompileState {
    pub(crate) fn new() -> Self {
        CompileState {
            next_idx: 1,
            used_names: HashMap::new(),
            unknown_references: vec![],
            unknown_groups: vec![],
        }
    }

    pub(crate) fn check_validity(self) -> Result<(), CompileError> {
        for (group, span) in self.unknown_groups {
            if group >= self.next_idx {
                return Err(CompileErrorKind::UnknownReferenceNumber(group as i32).at(span));
            }
        }
        if let Some((reference, span)) = self.unknown_references.into_iter().next() {
            return Err(CompileErrorKind::UnknownReferenceName(reference).at(span));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TransformState {
    pub(crate) next_idx: u32,
    pub(crate) capturing_groups: u32,
}

impl TransformState {
    pub(crate) fn new(capturing_groups: u32) -> Self {
        Self { next_idx: 1, capturing_groups }
    }
}
