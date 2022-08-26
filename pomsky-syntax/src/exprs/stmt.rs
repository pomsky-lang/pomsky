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
}

#[cfg(feature = "pretty-print")]
impl std::fmt::Debug for StmtExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DisplayDebug<T>(T);
        impl<T: std::fmt::Display> std::fmt::Debug for DisplayDebug<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        let mut x = f.debug_tuple("StmtExpr");
        let mut x = &mut x;
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => x = x.field(&DisplayDebug("enable lazy")),
            Stmt::Disable(BooleanSetting::Lazy) => x = x.field(&DisplayDebug("disable lazy")),
            Stmt::Let(r#let) => x = x.field(r#let),
        }
        x.field(&self.rule).finish()
    }
}

#[cfg(feature = "pretty-print")]
impl std::fmt::Debug for Let<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {:#?}", self.name, self.rule)
    }
}
