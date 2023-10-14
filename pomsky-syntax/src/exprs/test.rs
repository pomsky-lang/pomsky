use crate::Span;

use super::Literal;

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TestCapture<'i> {
    pub ident: CaptureIdent<'i>,
    pub ident_span: Span,
    pub literal: Literal<'i>,
}

#[derive(Debug, Clone, Copy)]
pub enum CaptureIdent<'i> {
    Name(&'i str),
    Index(u16),
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
