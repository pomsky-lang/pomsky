use std::collections::HashMap;

use pomsky_syntax::exprs::{Lookaround, LookaroundKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Lookaround<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        if options.flavor == RegexFlavor::Rust {
            return Err(
                CompileErrorKind::Unsupported(Feature::Lookaround, options.flavor).at(self.span)
            );
        }

        Ok(Regex::Lookaround(Box::new(RegexLookaround {
            content: self.rule.compile(options, state)?,
            kind: self.kind,
        })))
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        let feature = match self.kind {
            LookaroundKind::Ahead | LookaroundKind::AheadNegative => PomskyFeatures::LOOKAHEAD,
            LookaroundKind::Behind | LookaroundKind::BehindNegative => PomskyFeatures::LOOKBEHIND,
        };
        options.allowed_features.require(feature, self.span)
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexLookaround<'i> {
    pub(crate) content: Regex<'i>,
    pub(crate) kind: LookaroundKind,
}

impl<'i> RegexLookaround<'i> {
    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        buf.push_str(match self.kind {
            LookaroundKind::Ahead => "(?=",
            LookaroundKind::Behind => "(?<=",
            LookaroundKind::AheadNegative => "(?!",
            LookaroundKind::BehindNegative => "(?<!",
        });
        self.content.codegen(buf, flavor);
        buf.push(')');
    }
}
