use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    span::Span,
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Lookaround<'i> {
    kind: LookaroundKind,
    rule: Rulex<'i>,
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
pub enum LookaroundKind {
    Ahead,
    Behind,
    AheadNegative,
    BehindNegative,
}

impl<'i> Lookaround<'i> {
    pub fn new(rule: Rulex<'i>, kind: LookaroundKind, span: Span) -> Self {
        Lookaround { rule, kind, span }
    }
}

impl Compile for Lookaround<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        if options.flavor == RegexFlavor::Rust {
            return Err(
                CompileErrorKind::Unsupported(Feature::Lookaround, options.flavor).at(self.span)
            );
        }

        buf.push_str(match self.kind {
            LookaroundKind::Ahead => "(?=",
            LookaroundKind::Behind => "(?<=",
            LookaroundKind::AheadNegative => "(?!",
            LookaroundKind::BehindNegative => "(?<!",
        });
        self.rule.comp(options, state, buf)?;
        buf.push(')');
        Ok(())
    }
}
