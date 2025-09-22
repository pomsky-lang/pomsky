use pomsky_syntax::exprs::Rule;

use crate::{
    compile::{CompileResult, CompileState},
    options::CompileOptions,
    regex::Regex,
};

use super::{
    Compile, char_class::check_char_class_empty, codepoint::Codepoint, dot::Dot, grapheme::Grapheme,
};

impl Compile for Rule {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        match self {
            Rule::Literal(l) => l.compile(options, state),
            Rule::CharClass(c) => c.compile(options, state),
            Rule::Group(g) => g.compile(options, state),
            Rule::Grapheme => Grapheme {}.compile(options),
            Rule::Codepoint => Codepoint {}.compile(options),
            Rule::Dot => Dot {}.compile(options),
            Rule::Alternation(a) => a.compile(options, state),
            Rule::Intersection(a) => a.compile(options, state),
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
            Rule::Negation(n) => {
                let span = n.rule.span();
                let regex = n
                    .rule
                    .compile(options, state)
                    .and_then(|r| r.negate(n.not_span, options.flavor))?;
                if let Regex::CharSet(char_set) = &regex {
                    check_char_class_empty(char_set, span)?;
                }
                Ok(regex)
            }
        }
    }
}
