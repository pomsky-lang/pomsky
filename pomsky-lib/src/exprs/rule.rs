use std::collections::HashMap;

use pomsky_syntax::exprs::Rule;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind},
    options::CompileOptions,
};

use super::{grapheme::Grapheme, RuleExt};

impl<'i> RuleExt<'i> for Rule<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self {
            Rule::Literal(_) => {}
            Rule::CharClass(_) => {}
            Rule::Grapheme => {}
            Rule::Group(g) => g.get_capturing_groups(count, map, within_variable)?,
            Rule::Alternation(a) => a.get_capturing_groups(count, map, within_variable)?,
            Rule::Repetition(r) => r.get_capturing_groups(count, map, within_variable)?,
            Rule::Boundary(_) => {}
            Rule::Lookaround(l) => l.get_capturing_groups(count, map, within_variable)?,
            Rule::Variable(_) => {}
            Rule::Reference(r) => {
                if within_variable {
                    return Err(CompileErrorKind::ReferenceInLet.at(r.span));
                }
            }
            Rule::Regex(_) => {}
            Rule::Range(_) => {}
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
        }
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        match self {
            Rule::Literal(_) => {}
            Rule::CharClass(_) => {}
            Rule::Grapheme => Grapheme {}.validate(options)?,
            Rule::Group(g) => g.validate(options)?,
            Rule::Alternation(a) => a.validate(options)?,
            Rule::Repetition(r) => r.validate(options)?,
            Rule::Boundary(b) => b.validate(options)?,
            Rule::Lookaround(l) => l.validate(options)?,
            Rule::Variable(_) => {}
            Rule::Reference(r) => r.validate(options)?,
            Rule::Range(r) => r.validate(options)?,
            Rule::Regex(r) => r.validate(options)?,
            Rule::StmtExpr(s) => s.validate(options)?,
        }

        Ok(())
    }
}
