use std::collections::HashMap;

use crate::{error::CompileError, regex::Regex};

pub(crate) type CompileResult<'i> = Result<Regex<'i>, CompileError>;

#[derive(Debug, Clone)]
pub(crate) struct CompileState {
    pub(crate) next_idx: u32,
    pub(crate) used_names: HashMap<String, u32>,
    pub(crate) groups_count: u32,
}
