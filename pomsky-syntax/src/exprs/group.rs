use crate::Span;

use super::Rule;

/// A group, i.e. sequence of rules. A group is either capturing or
/// non-capturing.
///
/// If it is capturing, it must be wrapped in parentheses, and can have a name.
/// If it is non-capturing, the parentheses can be omitted in same cases.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Group {
    pub parts: Vec<Rule>,
    pub kind: GroupKind,
    pub span: Span,
}

impl Group {
    pub fn new(parts: Vec<Rule>, kind: GroupKind, span: Span) -> Self {
        Group { parts, kind, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter, needs_parens: bool) {
        let use_parens =
            matches!(self.kind, GroupKind::Capturing(_) | GroupKind::Atomic) || needs_parens;

        match &self.kind {
            GroupKind::Capturing(capture) => {
                buf.push(':');
                if let Some(name) = &capture.name {
                    buf.push_str(name);
                }
            }
            GroupKind::Atomic => {
                buf.push_str("atomic ");
            }
            GroupKind::Normal | GroupKind::Implicit => {}
        }

        if self.parts.is_empty() {
            buf.push_str("()");
        } else {
            if self.kind != GroupKind::Implicit {
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

            if self.kind != GroupKind::Implicit {
                buf.end_indentation(")");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum GroupKind {
    /// A (possibly named) capturing group e.g. `:foo`
    Capturing(Capture),
    /// An atomic group
    Atomic,
    /// A normal group with a set of parentheses
    Normal,
    /// An implicit group, with no parentheses
    Implicit,
}

impl GroupKind {
    pub fn is_normal(&self) -> bool {
        matches!(self, GroupKind::Normal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub name: Option<String>,
}

impl Capture {
    pub fn new(name: Option<&str>) -> Self {
        Capture { name: name.map(str::to_string) }
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for Capture {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        if u.arbitrary()? {
            Ok(Capture { name: Some(super::arbitrary::Ident::create(u)?) })
        } else {
            Ok(Capture { name: None })
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}
