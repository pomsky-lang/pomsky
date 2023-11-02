use std::collections::HashMap;

use pomsky_syntax::exprs::{self, Capture, GroupKind};

use crate::{
    diagnose::{CompileError, CompileErrorKind},
    visitor::RuleVisitor,
};

#[derive(Default)]
pub(crate) struct CapturingGroupsCollector {
    pub(crate) count_named: u32,
    pub(crate) count_numbered: u32,
    pub(crate) names: HashMap<String, CapturingGroupIndex>,
    pub(crate) variable_nesting: u32,
}

#[derive(Clone)]
pub(crate) struct CapturingGroupIndex {
    pub(crate) from_named: u32,
    pub(crate) absolute: u32,
}

impl CapturingGroupsCollector {
    pub(crate) fn new() -> Self {
        CapturingGroupsCollector::default()
    }
}

impl RuleVisitor<CompileError> for CapturingGroupsCollector {
    fn down(&mut self, kind: crate::visitor::NestingKind) {
        if let crate::visitor::NestingKind::Let = kind {
            self.variable_nesting += 1;
        }
    }

    fn up(&mut self, kind: crate::visitor::NestingKind) {
        if let crate::visitor::NestingKind::Let = kind {
            self.variable_nesting -= 1;
        }
    }

    fn visit_group(&mut self, group: &exprs::Group) -> Result<(), CompileError> {
        match group.kind {
            GroupKind::Capturing(Capture { name: Some(name) }) => {
                if self.variable_nesting > 0 {
                    return Err(CompileErrorKind::CaptureInLet.at(group.span));
                }

                if self.names.contains_key(name) {
                    return Err(
                        CompileErrorKind::NameUsedMultipleTimes(name.to_string()).at(group.span)
                    );
                }

                self.count_named += 1;
                let index = CapturingGroupIndex {
                    from_named: self.count_named,
                    absolute: self.count_named + self.count_numbered,
                };
                self.names.insert(name.to_string(), index);
            }
            GroupKind::Capturing(Capture { name: None }) => {
                if self.variable_nesting > 0 {
                    return Err(CompileErrorKind::CaptureInLet.at(group.span));
                }

                self.count_numbered += 1;
            }
            _ => {}
        }

        Ok(())
    }

    fn visit_reference(&mut self, reference: &exprs::Reference) -> Result<(), CompileError> {
        if self.variable_nesting > 0 {
            return Err(CompileErrorKind::ReferenceInLet.at(reference.span));
        }
        Ok(())
    }
}
