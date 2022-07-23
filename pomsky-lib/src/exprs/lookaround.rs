use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind, Feature, ParseError, ParseErrorKind},
    features::PomskyFeatures,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    regex::Regex,
    span::Span,
};

use super::Rule;

#[derive(Clone)]
pub(crate) struct Lookaround<'i> {
    kind: LookaroundKind,
    rule: Rule<'i>,
    pub(crate) span: Span,
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Lookaround<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Lookaround ")?;
        f.write_str(match self.kind {
            LookaroundKind::Ahead => ">> ",
            LookaroundKind::Behind => "<< ",
            LookaroundKind::AheadNegative => "!>> ",
            LookaroundKind::BehindNegative => "!<< ",
        })?;
        self.rule.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum LookaroundKind {
    Ahead,
    Behind,
    AheadNegative,
    BehindNegative,
}

impl<'i> Lookaround<'i> {
    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    pub(crate) fn new(rule: Rule<'i>, kind: LookaroundKind, span: Span) -> Self {
        Lookaround { rule, kind, span }
    }

    pub(crate) fn negate(&mut self) -> Result<(), ParseErrorKind> {
        match self.kind {
            LookaroundKind::AheadNegative | LookaroundKind::BehindNegative => {
                Err(ParseErrorKind::UnallowedDoubleNot)
            }
            LookaroundKind::Ahead => {
                self.kind = LookaroundKind::AheadNegative;
                Ok(())
            }
            LookaroundKind::Behind => {
                self.kind = LookaroundKind::BehindNegative;
                Ok(())
            }
        }
    }

    pub(crate) fn compile<'c>(
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
            content: self.rule.comp(options, state)?,
            kind: self.kind,
        })))
    }

    pub(crate) fn validate(&self, options: &ParseOptions) -> Result<(), ParseError> {
        let feature = match self.kind {
            LookaroundKind::Ahead => PomskyFeatures::LOOKAHEAD,
            LookaroundKind::Behind => PomskyFeatures::LOOKBEHIND,
            LookaroundKind::AheadNegative => PomskyFeatures::LOOKAHEAD,
            LookaroundKind::BehindNegative => PomskyFeatures::LOOKBEHIND,
        };
        options.allowed_features.require(feature, self.span)
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexLookaround<'i> {
    content: Regex<'i>,
    kind: LookaroundKind,
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
