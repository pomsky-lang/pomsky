//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use std::borrow::Cow;

use crate::Span;

use super::{Literal, Rule};

/// An [alternation](https://www.regular-expressions.info/alternation.html).
/// This is a list of alternatives. Each alternative is a [`Rule`].
///
/// If an alternative consists of multiple expressions (e.g. `'a' | 'b' 'c'`),
/// that alternative is a [`Rule::Group`]. Note that a group's parentheses are
/// removed when compiling to a regex if they aren't required. In other words,
/// `'a' | ('b' 'c')` compiles to `a|bc`.
#[derive(Clone)]
pub struct Alternation<'i> {
    pub rules: Vec<Rule<'i>>,
    pub(crate) span: Span,
}

impl<'i> Alternation<'i> {
    pub(crate) fn new_expr(rules: Vec<Rule<'i>>) -> Rule<'i> {
        rules
            .into_iter()
            .reduce(|a, b| match (a, b) {
                (Rule::Alternation(mut a), Rule::Alternation(b)) => {
                    a.span = a.span.join(b.span);
                    a.rules.extend(b.rules);
                    Rule::Alternation(a)
                }
                (Rule::Alternation(mut a), b) => {
                    a.span = a.span.join(b.span());
                    a.rules.push(b);
                    Rule::Alternation(a)
                }
                (a, b) => {
                    let span = a.span().join(b.span());
                    Rule::Alternation(Alternation { rules: vec![a, b], span })
                }
            })
            .unwrap_or_else(|| Rule::Literal(Literal::new(Cow::Borrowed(""), Span::default())))
    }
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Alternation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("Alternation");
        let mut d = &mut d;
        for rule in &self.rules {
            d = d.field(rule);
        }
        d.finish()
    }
}
