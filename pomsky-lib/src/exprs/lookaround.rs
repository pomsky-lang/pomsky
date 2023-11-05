use pomsky_syntax::exprs::{Lookaround, LookaroundKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompatWarning, CompileErrorKind, CompileWarningKind},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Lookaround<'i> {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match options.flavor {
            RegexFlavor::JavaScript => {
                if let LookaroundKind::Behind | LookaroundKind::BehindNegative = self.kind {
                    state.diagnostics.push(
                        CompileWarningKind::Compat(CompatWarning::JsLookbehind)
                            .at(self.span)
                            .diagnostic(),
                    );
                }
            }
            RegexFlavor::Ruby if state.in_lookbehind => {
                if let LookaroundKind::Ahead | LookaroundKind::AheadNegative = self.kind {
                    return Err(CompileErrorKind::RubyLookaheadInLookbehind {
                        was_word_boundary: false,
                    }
                    .at(self.span));
                }
            }
            _ => (),
        }

        revert_on_drop!(state.in_lookbehind);
        if let LookaroundKind::Behind | LookaroundKind::BehindNegative = self.kind {
            state.in_lookbehind = true;
        }

        Ok(Regex::Lookaround(Box::new(RegexLookaround {
            content: self.rule.compile(options, &mut state)?,
            kind: self.kind,
        })))
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
