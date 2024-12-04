//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

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
pub struct Alternation {
    pub rules: Vec<Rule>,
    pub(crate) span: Span,
}

impl Alternation {
    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        if needs_parens {
            buf.start_indentation("(");
        }

        let len = self.rules.len();
        for (i, rule) in self.rules.iter().enumerate() {
            let needs_parens =
                matches!(rule, Rule::Alternation(_) | Rule::Lookaround(_) | Rule::StmtExpr(_));

            buf.push_str("| ");
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
