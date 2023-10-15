use std::collections::HashMap;

use pomsky_syntax::exprs::Rule;

use crate::{
    compile::{CompileResult, CompileState, ValidationState},
    diagnose::{CompileError, CompileErrorKind},
    options::CompileOptions,
};

use super::{codepoint::Codepoint, dot::Dot, grapheme::Grapheme, RuleExt};

impl<'i> RuleExt<'i> for Rule<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self {
            Rule::Literal(_)
            | Rule::CharClass(_)
            | Rule::Codepoint
            | Rule::Grapheme
            | Rule::Dot
            | Rule::Boundary(_)
            | Rule::Variable(_)
            | Rule::Regex(_)
            | Rule::Range(_)
            | Rule::Recursion(_) => {}
            Rule::Group(g) => g.get_capturing_groups(count, map, within_variable)?,
            Rule::Alternation(a) => a.get_capturing_groups(count, map, within_variable)?,
            Rule::Repetition(r) => r.get_capturing_groups(count, map, within_variable)?,
            Rule::Lookaround(l) => l.get_capturing_groups(count, map, within_variable)?,
            Rule::Reference(r) => {
                if within_variable {
                    return Err(CompileErrorKind::ReferenceInLet.at(r.span));
                }
            }
            Rule::StmtExpr(m) => m.get_capturing_groups(count, map, within_variable)?,
        }
        Ok(())
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match self {
            Rule::Literal(l) => l.compile(options, state),
            Rule::CharClass(c) => c.compile(options, state),
            Rule::Group(g) => g.compile(options, state),
            Rule::Grapheme => Grapheme {}.compile(options),
            Rule::Codepoint => Codepoint {}.compile(options),
            Rule::Dot => Dot {}.compile(options),
            Rule::Alternation(a) => a.compile(options, state),
            Rule::Repetition(r) => r.compile(options, state),
            Rule::Boundary(b) => b.compile(options, state),
            Rule::Lookaround(l) => l.compile(options, state),
            Rule::Variable(v) => v.compile(options, state).map_err(|mut e| {
                e.set_missing_span(v.span);
                e
            }),
            Rule::Reference(r) => r.compile(options, state),
            Rule::Range(r) => r.compile(options, state),
            Rule::Regex(r) => r.compile(options, state),
            Rule::StmtExpr(m) => m.compile(options, state),
            Rule::Recursion(r) => r.compile(options, state),
        }
    }

    fn validate(
        &self,
        options: &CompileOptions,
        state: &mut ValidationState,
    ) -> Result<(), CompileError> {
        match self {
            Rule::Literal(_) | Rule::CharClass(_) | Rule::Variable(_) | Rule::Codepoint => Ok(()),
            Rule::Grapheme => Grapheme {}.validate(options),
            Rule::Dot => Dot {}.validate(options),
            Rule::Group(g) => g.validate(options, state),
            Rule::Alternation(a) => a.validate(options, state),
            Rule::Repetition(r) => r.validate(options, state),
            Rule::Boundary(b) => b.validate(options, state),
            Rule::Lookaround(l) => l.validate(options, state),
            Rule::Reference(r) => r.validate(options, state),
            Rule::Range(r) => r.validate(options, state),
            Rule::Regex(r) => r.validate(options, state),
            Rule::Recursion(r) => r.validate(options, state),
            Rule::StmtExpr(s) => s.validate(options, state),
        }
    }
}
