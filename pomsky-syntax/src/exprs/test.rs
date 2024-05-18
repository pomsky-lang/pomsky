use crate::Span;

use super::Literal;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Test {
    pub cases: Vec<TestCase>,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum TestCase {
    Match(TestCaseMatch),
    MatchAll(TestCaseMatchAll),
    Reject(TestCaseReject),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct TestCaseMatch {
    pub literal: Literal,
    pub captures: Vec<TestCapture>,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct TestCaseMatchAll {
    pub literal: Literal,
    pub matches: Vec<TestCaseMatch>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct TestCaseReject {
    pub literal: Literal,
    pub as_substring: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct TestCapture {
    pub ident: CaptureIdent,
    pub ident_span: Span,
    pub literal: Literal,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum CaptureIdent {
    Name(String),
    Index(u16),
}

impl TestCase {
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

impl TestCaseMatch {
    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        self.literal.pretty_print(buf);

        if !self.captures.is_empty() {
            buf.start_indentation(" as {");

            let len = self.captures.len();
            for (i, capture) in self.captures.iter().enumerate() {
                match &capture.ident {
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
