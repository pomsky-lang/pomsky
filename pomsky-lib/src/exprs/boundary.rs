//! Implements _boundaries_. The analogues in the regex world are
//! [word boundaries](https://www.regular-expressions.info/wordboundaries.html) and
//! [anchors](https://www.regular-expressions.info/anchors.html).

use pomsky_syntax::exprs::{Boundary, BoundaryKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileError, CompileErrorKind},
    features::PomskyFeatures,
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
        if options.flavor == RegexFlavor::JavaScript
            && !state.ascii_only
            && matches!(self.kind, BoundaryKind::Word | BoundaryKind::NotWord)
        {
            Err(CompileErrorKind::JsWordBoundaryInUnicodeMode.at(self.span))
        } else {
            Ok(Regex::Boundary(self.kind))
        }
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        options.allowed_features.require(PomskyFeatures::BOUNDARIES, self.span)
    }
}

pub(crate) fn boundary_kind_codegen(bk: BoundaryKind, buf: &mut String) {
    match bk {
        BoundaryKind::Start => buf.push('^'),
        BoundaryKind::Word => buf.push_str("\\b"),
        BoundaryKind::NotWord => buf.push_str("\\B"),
        BoundaryKind::End => buf.push('$'),
    }
}
