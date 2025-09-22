use pomsky_syntax::exprs::{Lookaround, LookaroundKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::CompileErrorKind,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::Compile;

impl Compile for Lookaround {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        match options.flavor {
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

        let content = self.rule.compile(options, &mut state)?;
        let lookaround = RegexLookaround::new(content, self.kind, options.flavor)
            .map_err(|e| e.at(self.span))?;

        Ok(Regex::Lookaround(Box::new(lookaround)))
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexLookaround {
    pub(crate) content: Regex,
    pub(crate) kind: LookaroundKind,
}

impl RegexLookaround {
    pub(crate) fn new(
        content: Regex,
        kind: LookaroundKind,
        flavor: RegexFlavor,
    ) -> Result<Self, CompileErrorKind> {
        if let LookaroundKind::Behind | LookaroundKind::BehindNegative = kind {
            match flavor {
                RegexFlavor::Python => {
                    content.validate_in_lookbehind_py()?;
                }
                RegexFlavor::Pcre => {
                    content.validate_in_lookbehind_pcre()?;
                }
                RegexFlavor::Java => {
                    content.validate_in_lookbehind_java()?;
                }
                _ => {}
            }
        }

        Ok(RegexLookaround { content, kind })
    }

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
