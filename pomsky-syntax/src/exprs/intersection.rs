//! Implements intersection: `'alt1' & 'alt2' & 'alt3'`. This is not a common feature,
//! and only makes sense in certain scenarios.

use crate::Span;

use super::Rule;

/// An [alternation](https://www.regular-expressions.info/alternation.html).
/// This is a list of alternatives. Each alternative is a [`Rule`].
///
/// If an alternative consists of multiple expressions (e.g. `'a' | 'b' 'c'`),
/// that alternative is a [`Rule::Group`]. Note that a group's parentheses are
/// removed when compiling to a regex if they aren't required. In other words,
/// `'a' | ('b' 'c')` compiles to `a|bc`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Intersection {
    pub rules: Vec<Rule>,
    pub span: Span,
}

impl Intersection {
    pub(crate) fn new_expr(rules: Vec<Rule>, start_span: Span) -> Option<Rule> {
        rules
            .into_iter()
            .reduce(|a, b| match (a, b) {
                (Rule::Intersection(mut a), Rule::Intersection(b)) => {
                    a.span = a.span.join(b.span);
                    a.rules.extend(b.rules);
                    Rule::Intersection(a)
                }
                (Rule::Intersection(mut a), b) => {
                    a.span = a.span.join(b.span());
                    a.rules.push(b);
                    Rule::Intersection(a)
                }
                (a, b) => {
                    let span = a.span().join(b.span());
                    Rule::Intersection(Intersection { rules: vec![a, b], span })
                }
            })
            .map(|mut rule| {
                if let Rule::Intersection(i) = &mut rule {
                    i.span = i.span.join(start_span)
                }
                rule
            })
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        if needs_parens {
            buf.start_indentation("(");
        }

        let len = self.rules.len();
        for (i, rule) in self.rules.iter().enumerate() {
            let needs_parens = matches!(
                rule,
                Rule::Intersection(_)
                    | Rule::Alternation(_)
                    | Rule::Lookaround(_)
                    | Rule::StmtExpr(_)
            );

            buf.push_str("& ");
            buf.increase_indentation(2);
            rule.pretty_print(buf, needs_parens);
            buf.decrease_indentation(2);
            if i < len - 1 {
                buf.write("\n");
            }
        }

        if needs_parens {
            buf.end_indentation(")");
        }
    }
}
