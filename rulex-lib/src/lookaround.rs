use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Lookaround<'i> {
    kind: LookaroundKind,
    rule: Rulex<'i>,
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
    pub fn new(rule: Rulex<'i>, kind: LookaroundKind) -> Self {
        Lookaround { rule, kind }
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
            return Err(CompileError::Unsupported(
                Feature::Lookaround,
                options.flavor,
            ));
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
