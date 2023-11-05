//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use pomsky_syntax::exprs::{Boundary, BoundaryKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::CompileErrorKind,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Boundary {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        use BoundaryKind::*;

        if options.flavor == RegexFlavor::JavaScript
            && !state.ascii_only
            && matches!(self.kind, Word | NotWord | WordStart | WordEnd)
        {
            Err(CompileErrorKind::JsWordBoundaryInUnicodeMode.at(self.span))
        } else if options.flavor == RegexFlavor::Ruby && state.in_lookbehind {
            Err(CompileErrorKind::RubyLookaheadInLookbehind { was_word_boundary: true }
                .at(self.span))
        } else {
            Ok(Regex::Boundary(self.kind))
        }
    }
}

pub(crate) fn boundary_kind_codegen(bk: BoundaryKind, buf: &mut String, flavor: RegexFlavor) {
    match bk {
        BoundaryKind::Start => buf.push('^'),
        BoundaryKind::End => buf.push('$'),

        BoundaryKind::Word => buf.push_str(r"\b"),
        BoundaryKind::NotWord => buf.push_str(r"\B"),

        BoundaryKind::WordStart => buf.push_str(match flavor {
            RegexFlavor::Pcre => "[[:<:]]",
            RegexFlavor::Rust => r"\<",
            _ => r"(?<!\w)(?=\w)",
        }),
        BoundaryKind::WordEnd => buf.push_str(match flavor {
            RegexFlavor::Pcre => "[[:>:]]",
            RegexFlavor::Rust => r"\>",
            _ => r"(?<=\w)(?!\w)",
        }),
    }
}
