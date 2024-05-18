use crate::Span;

use super::{test::Test, Rule};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct StmtExpr {
    pub stmt: Stmt,
    pub rule: Rule,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Stmt {
    Enable(BooleanSetting, Span),
    Disable(BooleanSetting, Span),
    Let(Let),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum BooleanSetting {
    Lazy,
    Unicode,
}

impl BooleanSetting {
    #[cfg(feature = "dbg")]
    fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        match self {
            BooleanSetting::Lazy => buf.write("lazy"),
            BooleanSetting::Unicode => buf.write("unicode"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub rule: Rule,
    pub name_span: Span,
}

impl Let {
    pub fn new(name: &str, rule: Rule, name_span: Span) -> Self {
        Self { name: name.to_string(), rule, name_span }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "arbitrary")]
impl arbitrary::Arbitrary<'_> for Let {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let name = super::arbitrary::Ident::create(u)?;
        Ok(Let { name, rule: Rule::arbitrary(u)?, name_span: Span::arbitrary(u)? })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::recursion_guard(depth, |depth| {
            arbitrary::size_hint::and(
                super::arbitrary::Ident::size_hint(depth),
                Rule::size_hint(depth),
            )
        })
    }
}

impl StmtExpr {
    pub fn new(stmt: Stmt, rule: Rule, span: Span) -> Self {
        Self { stmt, rule, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        match &self.stmt {
            Stmt::Enable(setting, _) | Stmt::Disable(setting, _) => {
                buf.write(if matches!(&self.stmt, Stmt::Enable(..)) {
                    "enable "
                } else {
                    "disable "
                });
                setting.pretty_print(buf);
                buf.write(";\n");
                self.rule.pretty_print(buf, false);
            }
            Stmt::Let(r#let) => {
                buf.push_str("let ");
                buf.write(&r#let.name);
                buf.push_str(" = ");
                r#let.rule.pretty_print(buf, true);
                buf.write(";\n");
                self.rule.pretty_print(buf, false);
            }
            Stmt::Test(test) => {
                buf.push_str("test ");
                buf.start_indentation("{");

                let len = test.cases.len();
                for (i, test_case) in test.cases.iter().enumerate() {
                    test_case.pretty_print(buf);
                    if i < len - 1 {
                        buf.write("\n");
                    }
                }
                buf.end_indentation("}");
                buf.write("\n");
                self.rule.pretty_print(buf, false);
            }
        }
    }
}
