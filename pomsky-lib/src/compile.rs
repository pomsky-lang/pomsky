use std::collections::{HashMap, HashSet};

use pomsky_syntax::exprs::Rule;

use crate::{
    capturing_groups::{CapturingGroupIndex, CapturingGroupsCollector},
    diagnose::{CompileError, Diagnostic},
    regex::Regex,
};

pub(crate) type CompileResult = Result<Regex, CompileError>;

#[derive(Clone)]
pub(crate) struct CompileState<'i> {
    pub(crate) next_idx: u32,
    pub(crate) used_names_vec: Vec<Option<String>>,
    pub(crate) used_names: HashMap<String, CapturingGroupIndex>,
    pub(crate) groups_count: u32,
    pub(crate) numbered_groups_count: u32,
    pub(crate) in_lookbehind: bool,

    pub(crate) variables: Vec<(&'i str, &'i Rule)>,
    pub(crate) current_vars: HashSet<usize>,

    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl<'i> CompileState<'i> {
    pub(crate) fn new(
        capt_groups: CapturingGroupsCollector,
        variables: Vec<(&'i str, &'i Rule)>,
    ) -> Self {
        let used_names = capt_groups.names;
        let groups_count = capt_groups.count_named + capt_groups.count_numbered;

        // needed for Ruby: In Ruby, backreferences to named groups have to be named as
        // well
        let mut used_names_vec = vec![None; groups_count as usize + 1];
        for (name, index) in &used_names {
            used_names_vec[index.absolute as usize] = Some(name.clone());
        }

        CompileState {
            next_idx: 1,
            used_names_vec,
            used_names,
            groups_count,
            numbered_groups_count: capt_groups.count_numbered,
            in_lookbehind: false,

            variables,
            current_vars: Default::default(),

            diagnostics: vec![],
        }
    }

    pub(crate) fn has_named_groups(&self) -> bool {
        self.numbered_groups_count < self.groups_count
    }

    pub(crate) fn has_numbered_groups(&self) -> bool {
        self.numbered_groups_count > 0
    }
}
