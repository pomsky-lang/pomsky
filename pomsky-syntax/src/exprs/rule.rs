use crate::Span;

use super::{
    Alternation, Boundary, CharClass, Group, Literal, Lookaround, Range, Recursion, Reference,
    Regex, Repetition, StmtExpr, Variable, intersection::Intersection, negation::Negation,
};

/// A parsed pomsky expression, which might contain more sub-expressions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Rule {
    /// A string literal
    Literal(Literal),
    /// A character class
    CharClass(CharClass),
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(Group),
    /// An alternation, i.e. a list of alternatives; at least one of them has to
    /// match.
    Alternation(Alternation),
    /// An intersection, i.e. a list of rules that all have to match at the same time.
    Intersection(Intersection),
    /// A repetition, i.e. a expression that must be repeated. The number of
    /// required repetitions is constrained by a lower and possibly an upper
    /// bound.
    Repetition(Box<Repetition>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(Boundary),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<Lookaround>),
    /// An variable that has been declared before.
    Variable(Variable),
    /// A backreference or forward reference.
    Reference(Reference),
    /// A range of integers
    Range(Range),
    /// An expression preceded by a modifier such as `enable lazy;`
    StmtExpr(Box<StmtExpr>),
    /// Negated expression
    Negation(Box<Negation>),
    /// A regex string, which is not escaped
    Regex(Regex),
    /// A regex string, which is not escaped
    Recursion(Recursion),

    /// A Unicode grapheme
    Grapheme,
    /// A Unicode code point
    Codepoint,
    /// The dot
    Dot,
}

impl Rule {
    /// Returns the span of this rule
    pub fn span(&self) -> Span {
        match self {
            Rule::Literal(l) => l.span,
            Rule::CharClass(c) => c.span,
            Rule::Group(g) => g.span,
            Rule::Alternation(a) => a.span,
            Rule::Intersection(i) => i.span,
            Rule::Repetition(r) => r.span,
            Rule::Boundary(b) => b.span,
            Rule::Lookaround(l) => l.span,
            Rule::Variable(v) => v.span,
            Rule::Reference(r) => r.span,
            Rule::Range(r) => r.span,
            Rule::StmtExpr(m) => m.span,
            Rule::Negation(n) => n.not_span.join(n.rule.span()),
            Rule::Regex(r) => r.span,
            Rule::Recursion(r) => r.span,
            Rule::Grapheme | Rule::Codepoint | Rule::Dot => Span::empty(),
        }
    }

    #[cfg(feature = "dbg")]
    pub(crate) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        match self {
            Rule::Literal(l) => l.pretty_print(buf),
            Rule::CharClass(c) => c.pretty_print(buf),
            Rule::Group(g) => g.pretty_print(buf, needs_parens),
            Rule::Alternation(a) => a.pretty_print(buf, needs_parens),
            Rule::Intersection(i) => i.pretty_print(buf, needs_parens),
            Rule::Repetition(r) => r.pretty_print(buf),
            Rule::Boundary(b) => b.pretty_print(buf),
            Rule::Lookaround(l) => l.pretty_print(buf, needs_parens),
            Rule::Variable(v) => v.pretty_print(buf),
            Rule::Reference(r) => r.pretty_print(buf),
            Rule::Range(r) => r.pretty_print(buf),
            Rule::StmtExpr(s) => s.pretty_print(buf),
            Rule::Negation(n) => n.pretty_print(buf, needs_parens),
            Rule::Regex(r) => r.pretty_print(buf),
            Rule::Recursion(_) => buf.push_str("recursion"),
            Rule::Grapheme => buf.push_str("Grapheme"),
            Rule::Codepoint => buf.push_str("Codepoint"),
            Rule::Dot => buf.push_str("."),
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = crate::PrettyPrinter::new();
        self.pretty_print(&mut buf, false);
        f.write_str(&buf.finish())
    }
}
