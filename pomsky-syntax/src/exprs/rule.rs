use crate::{error::ParseErrorKind, Span};

use super::{
    Alternation, Boundary, CharClass, Group, Literal, Lookaround, Range, Reference, Repetition,
    StmtExpr, Variable,
};

/// A parsed pomsky expression, which might contain more sub-expressions.
#[derive(Clone)]
pub enum Rule<'i> {
    /// A string literal
    Literal(Literal<'i>),
    /// A character class
    CharClass(CharClass),
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(Group<'i>),
    /// An alternation, i.e. a list of alternatives; at least one of them has to
    /// match.
    Alternation(Alternation<'i>),
    /// A repetition, i.e. a expression that must be repeated. The number of
    /// required repetitions is constrained by a lower and possibly an upper
    /// bound.
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

    /// A Unicode grapheme
    Grapheme,
}

impl<'i> Rule<'i> {
    /// Returns the span of this rule
    pub fn span(&self) -> Span {
        match self {
            Rule::Literal(l) => l.span,
            Rule::CharClass(c) => c.span,
            Rule::Group(g) => g.span,
            Rule::Alternation(a) => a.span,
            Rule::Repetition(r) => r.span,
            Rule::Boundary(b) => b.span,
            Rule::Lookaround(l) => l.span,
            Rule::Variable(v) => v.span,
            Rule::Reference(r) => r.span,
            Rule::Range(r) => r.span,
            Rule::StmtExpr(m) => m.span,
            Rule::Grapheme => Span::empty(),
        }
    }

    pub(crate) fn negate(&mut self) -> Result<(), ParseErrorKind> {
        match self {
            Rule::Literal(_)
            | Rule::Group(_)
            | Rule::Alternation(_)
            | Rule::Variable(_)
            | Rule::Reference(_)
            | Rule::Range(_)
            | Rule::StmtExpr(_)
            | Rule::Grapheme => Err(ParseErrorKind::UnallowedNot),

            Rule::CharClass(c) => c.negate(),
            Rule::Repetition(r) => r.rule.negate(),
            Rule::Boundary(b) => b.negate(),
            Rule::Lookaround(l) => l.negate(),
        }
    }
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Rule<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Rule::Literal(arg0) => arg0.fmt(f),
            Rule::CharClass(arg0) => arg0.fmt(f),
            Rule::Group(arg0) => arg0.fmt(f),
            Rule::Alternation(arg0) => arg0.fmt(f),
            Rule::Repetition(arg0) => arg0.fmt(f),
            Rule::Boundary(arg0) => arg0.fmt(f),
            Rule::Lookaround(arg0) => arg0.fmt(f),
            Rule::Variable(arg0) => arg0.fmt(f),
            Rule::Reference(arg0) => arg0.fmt(f),
            Rule::Range(arg0) => arg0.fmt(f),
            Rule::StmtExpr(arg0) => arg0.fmt(f),
            Rule::Grapheme => f.write_str("Grapheme"),
        }
    }
}
