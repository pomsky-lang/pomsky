use crate::Span;

use super::{Literal, Rule};

#[derive(Clone)]
pub struct StmtExpr<'i> {
    pub stmt: Stmt<'i>,
    pub rule: Rule<'i>,
    pub span: Span,
}

#[derive(Clone)]
pub enum Stmt<'i> {
    Enable(BooleanSetting, Span),
    Disable(BooleanSetting, Span),
    Let(Let<'i>),
    Test(Test<'i>),
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

#[derive(Clone)]
pub struct Test<'i> {
    pub cases: Vec<TestCase<'i>>,
    pub span: Span,
}

#[derive(Clone)]
pub enum TestCase<'i> {
    Match(TestCaseMatch<'i>),
    MatchAll(TestCaseMatchAll<'i>),
    Reject(TestCaseReject<'i>),
}

#[derive(Clone)]
pub struct TestCaseMatch<'i> {
    pub literal: Literal<'i>,
    pub captures: Vec<TestCapture<'i>>,
    pub span: Span,
}

#[derive(Clone)]
pub struct TestCaseMatchAll<'i> {
    pub literal: Literal<'i>,
    pub matches: Vec<TestCaseMatch<'i>>,
}

#[derive(Clone)]
pub struct TestCaseReject<'i> {
    pub literal: Literal<'i>,
    pub as_substring: bool,
}

#[derive(Clone)]
pub struct TestCapture<'i> {
    pub ident: CaptureIdent<'i>,
    pub ident_span: Span,
    pub literal: Literal<'i>,
}

#[derive(Clone, Copy)]
pub enum CaptureIdent<'i> {
    Name(&'i str),
    Index(u16),
}

impl<'i> StmtExpr<'i> {
    pub fn new(stmt: Stmt<'i>, rule: Rule<'i>, span: Span) -> Self {
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
                buf.write(r#let.name);
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

impl<'i> TestCase<'i> {
    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        match self {
            TestCase::Match(match_) => {
                buf.push_str("match ");
                match_.pretty_print(buf);
                buf.push(';');
            }
            TestCase::MatchAll(match_all) => {
                buf.push_str("match ");
                let len = match_all.matches.len();
                buf.increase_indentation(if len == 0 { 3 } else { 6 });

                for (i, match_) in match_all.matches.iter().enumerate() {
                    match_.pretty_print(buf);
                    if i < len - 1 {
                        buf.push(',');
                    } else {
                        buf.decrease_indentation(3);
                    }
                    buf.write("\n");
                }
                buf.push_str("in ");
                match_all.literal.pretty_print(buf);
                buf.decrease_indentation(3);
                buf.push(';');
            }
            TestCase::Reject(reject) => {
                buf.push_str("reject ");
                if reject.as_substring {
                    buf.push_str("in ");
                }
                reject.literal.pretty_print(buf);
                buf.push(';');
            }
        }
    }
}

impl<'i> TestCaseMatch<'i> {
    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        self.literal.pretty_print(buf);

        if !self.captures.is_empty() {
            buf.start_indentation(" as {");

            let len = self.captures.len();
            for (i, capture) in self.captures.iter().enumerate() {
                match capture.ident {
                    CaptureIdent::Name(name) => buf.push_str(name),
                    CaptureIdent::Index(idx) => buf.write_fmt(idx),
                }
                buf.push_str(": ");
                capture.literal.pretty_print(buf);
                buf.push(',');
                if i < len - 1 {
                    buf.write("\n");
                }
            }
            buf.end_indentation("}");
        }
    }
}
