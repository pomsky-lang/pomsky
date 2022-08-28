use crate::Span;

use super::Rule;

/// A group, i.e. sequence of rules. A group is either capturing or
/// non-capturing.
///
/// If it is capturing, it must be wrapped in parentheses, and can have a name.
/// If it is non-capturing, the parentheses can be omitted in same cases.
#[derive(Clone)]
pub struct Group<'i> {
    pub parts: Vec<Rule<'i>>,
    pub capture: Option<Capture<'i>>,
    pub span: Span,
}

impl<'i> Group<'i> {
    pub fn new(parts: Vec<Rule<'i>>, capture: Option<Capture<'i>>, span: Span) -> Self {
        Group { parts, capture, span }
    }

    pub fn set_capture(&mut self, capture: Capture<'i>) {
        self.capture = Some(capture);
    }

    pub fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        let use_parens = self.capture.is_some() || needs_parens;

        if let Some(capture) = self.capture {
            buf.push(':');
            if let Some(name) = capture.name {
                buf.push_str(name);
            }
        }

        if self.parts.is_empty() {
            buf.push_str("()");
        } else {
            if use_parens {
                buf.start_indentation("(");
            }

            let len = self.parts.len();
            for (i, part) in self.parts.iter().enumerate() {
                let child_needs_parens = if len == 1 {
                    if use_parens {
                        false
                    } else {
                        needs_parens
                    }
                } else {
                    use Rule::*;
                    matches!(part, Lookaround(_) | StmtExpr(_) | Alternation(_) | Group(_))
                };
                part.pretty_print(buf, child_needs_parens);
                if i < len - 1 {
                    buf.write("\n");
                }
            }

            if use_parens {
                buf.end_indentation(")");
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capture<'i> {
    pub name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}
