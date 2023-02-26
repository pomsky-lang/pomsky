use crate::Span;

use super::Rule;

#[derive(Clone)]
pub struct StmtExpr<'i> {
    pub stmt: Stmt<'i>,
    pub rule: Rule<'i>,
    pub span: Span,
}

#[derive(Clone)]
pub enum Stmt<'i> {
    Enable(BooleanSetting),
    Disable(BooleanSetting),
    Let(Let<'i>),
}

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone)]
pub struct Let<'i> {
    pub name: &'i str,
    pub rule: Rule<'i>,
    pub name_span: Span,
}

impl<'i> Let<'i> {
    pub fn new(name: &'i str, rule: Rule<'i>, name_span: Span) -> Self {
        Self { name, rule, name_span }
    }

    pub fn name(&self) -> &'i str {
        self.name
    }
}

impl<'i> StmtExpr<'i> {
    pub fn new(stmt: Stmt<'i>, rule: Rule<'i>, span: Span) -> Self {
        Self { stmt, rule, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        match &self.stmt {
            Stmt::Enable(setting) | Stmt::Disable(setting) => {
                buf.write(if matches!(&self.stmt, Stmt::Enable(_)) {
                    "enable "
                } else {
                    "disable "
                });
                setting.pretty_print(buf);
                buf.write(";\n");
            }
            Stmt::Let(r#let) => {
                buf.push_str("let ");
                buf.write(r#let.name);
                buf.push_str(" = ");
                r#let.rule.pretty_print(buf, true);
                buf.write(";\n");
                self.rule.pretty_print(buf, false);
            }
        }
    }
}
