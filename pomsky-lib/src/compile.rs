use std::collections::{HashMap, HashSet};

use pomsky_syntax::exprs::Rule;

use crate::{
    diagnose::{CompileError, Diagnostic},
    exprs::repetition::RegexQuantifier,
    regex::Regex,
};

pub(crate) type CompileResult<'i> = Result<Regex<'i>, CompileError>;

#[derive(Clone)]
pub(crate) struct CompileState<'c, 'i> {
    pub(crate) next_idx: u32,
    pub(crate) used_names_vec: Vec<Option<String>>,
    pub(crate) used_names: HashMap<String, u32>,
    pub(crate) groups_count: u32,
    pub(crate) has_named: bool,

    pub(crate) default_quantifier: RegexQuantifier,
    pub(crate) variables: Vec<(&'i str, &'c Rule<'i>)>,
    pub(crate) current_vars: HashSet<usize>,

    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl<'c, 'i> CompileState<'c, 'i> {
    pub(crate) fn new(
        default_quantifier: RegexQuantifier,
        used_names: HashMap<String, u32>,
        groups_count: u32,
        variables: Vec<(&'i str, &'c Rule<'i>)>,
    ) -> Self {
        // needed for Ruby: In Ruby, backreferences to named groups have to be named as
        // well
        let mut used_names_vec = vec![None; groups_count as usize + 1];
        let mut has_named = false;
        for (name, &index) in &used_names {
            used_names_vec[index as usize] = Some(name.clone());
            has_named = true;
        }

        CompileState {
            next_idx: 1,
            used_names_vec,
            used_names,
            groups_count,
            has_named,

            default_quantifier,
            variables,
            current_vars: Default::default(),

            diagnostics: vec![],
        }
    }
}
