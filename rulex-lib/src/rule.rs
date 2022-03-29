use std::collections::HashMap;

use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind},
    grapheme::Grapheme,
    group::Group,
    literal::Literal,
    lookaround::Lookaround,
    options::CompileOptions,
    range::Range,
    reference::Reference,
    repetition::Repetition,
    span::Span,
    stmt::StmtExpr,
    var::Variable,
};

/// A parsed rulex expression, which might contain more sub-expressions.
#[derive(Clone)]
#[non_exhaustive]
pub(crate) enum Rule<'i> {
    /// A string literal
    Literal(Literal<'i>),
    /// A character class
    CharClass(CharClass),
    /// A Unicode grapheme
    Grapheme(Grapheme),
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(Group<'i>),
    /// An alternation, i.e. a list of alternatives; at least one of them has to match.
    Alternation(Alternation<'i>),
    /// A repetition, i.e. a expression that must be repeated. The number of required repetitions is
    /// constrained by a lower and possibly an upper bound.
    Repetition(Box<Repetition<'i>>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(Boundary),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<Lookaround<'i>>),
    /// An variable that has been declared before.
    Variable(Variable<'i>),
    /// A backreference or forward reference.
    Reference(Reference<'i>),
    /// A range of integers
    Range(Range),
    /// An expression preceded by a modifier such as `enable lazy;`
    StmtExpr(Box<StmtExpr<'i>>),
}

impl<'i> Rule<'i> {
    pub(crate) fn span(&self) -> Span {
        match self {
            Rule::Literal(l) => l.span,
            Rule::CharClass(c) => c.span,
            Rule::Grapheme(g) => g.span,
            Rule::Group(g) => g.span,
            Rule::Alternation(a) => a.span,
            Rule::Repetition(r) => r.span,
            Rule::Boundary(b) => b.span,
            Rule::Lookaround(l) => l.span,
            Rule::Variable(v) => v.span,
            Rule::Reference(r) => r.span,
            Rule::Range(r) => r.span,
            Rule::StmtExpr(m) => m.span,
        }
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self {
            Rule::Literal(_) => {}
            Rule::CharClass(_) => {}
            Rule::Grapheme(_) => {}
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
            Rule::Range(_) => {}
            Rule::StmtExpr(m) => m.get_capturing_groups(count, map, within_variable)?,
        }
        Ok(())
    }

    pub(crate) fn comp<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match self {
            Rule::Literal(l) => l.compile(),
            Rule::CharClass(c) => c.compile(options),
            Rule::Group(g) => g.compile(options, state),
            Rule::Grapheme(g) => g.compile(options),
            Rule::Alternation(a) => a.compile(options, state),
            Rule::Repetition(r) => r.compile(options, state),
            Rule::Boundary(b) => b.compile(),
            Rule::Lookaround(l) => l.compile(options, state),
            Rule::Variable(v) => v.compile(options, state),
            Rule::Reference(r) => r.compile(options, state),
            Rule::Range(r) => r.compile(),
            Rule::StmtExpr(m) => m.compile(options, state),
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rule<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Rule::Literal(arg0) => arg0.fmt(f),
            Rule::CharClass(arg0) => arg0.fmt(f),
            Rule::Grapheme(arg0) => arg0.fmt(f),
            Rule::Group(arg0) => arg0.fmt(f),
            Rule::Alternation(arg0) => arg0.fmt(f),
            Rule::Repetition(arg0) => arg0.fmt(f),
            Rule::Boundary(arg0) => arg0.fmt(f),
            Rule::Lookaround(arg0) => arg0.fmt(f),
            Rule::Variable(arg0) => arg0.fmt(f),
            Rule::Reference(arg0) => arg0.fmt(f),
            Rule::Range(arg0) => arg0.fmt(f),
            Rule::StmtExpr(arg0) => arg0.fmt(f),
        }
    }
}
