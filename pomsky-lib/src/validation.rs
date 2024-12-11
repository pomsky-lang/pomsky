use pomsky_syntax::{exprs, Span};

use crate::{
    diagnose::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures as Feat,
    options::{CompileOptions, RegexFlavor},
    visitor::{NestingKind, RuleVisitor},
};

#[derive(Clone)]
pub(crate) struct Validator {
    pub(crate) options: CompileOptions,
    pub(crate) layer: u32,
}

impl Validator {
    pub(crate) fn new(options: CompileOptions) -> Self {
        Validator { options, layer: 0 }
    }

    fn require(&self, feature: u16, span: Span) -> Result<(), CompileError> {
        self.options.allowed_features.require(feature, span)
    }

    fn flavor(&self) -> RegexFlavor {
        self.options.flavor
    }
}

impl RuleVisitor<CompileError> for Validator {
    fn down(&mut self, kind: NestingKind) {
        if !matches!(kind, NestingKind::StmtExpr) {
            self.layer += 1;
        }
    }

    fn up(&mut self, kind: NestingKind) {
        if !matches!(kind, NestingKind::StmtExpr) {
            self.layer -= 1;
        }
    }

    fn visit_repetition(&mut self, repetition: &exprs::Repetition) -> Result<(), CompileError> {
        if let (RegexFlavor::RE2, Some(1001..)) = (self.flavor(), repetition.kind.upper_bound) {
            return Err(CompileErrorKind::Unsupported(Feature::RepetitionAbove1000, self.flavor())
                .at(repetition.span));
        }
        Ok(())
    }

    fn visit_intersection(&mut self, int: &exprs::Intersection) -> Result<(), CompileError> {
        self.require(Feat::INTERSECTION, int.span)
    }

    fn visit_group(&mut self, group: &exprs::Group) -> Result<(), CompileError> {
        match &group.kind {
            exprs::GroupKind::Atomic => {
                self.require(Feat::ATOMIC_GROUPS, group.span)?;

                if let RegexFlavor::JavaScript | RegexFlavor::Rust | RegexFlavor::RE2 =
                    self.flavor()
                {
                    return Err(CompileErrorKind::Unsupported(
                        Feature::AtomicGroups,
                        self.flavor(),
                    )
                    .at(group.span));
                }
            }
            exprs::GroupKind::Capturing(c) => {
                let feature = match &c.name {
                    Some(_) => Feat::NAMED_GROUPS,
                    None => Feat::NUMBERED_GROUPS,
                };

                self.require(feature, group.span)?;
            }
            _ => (),
        }

        Ok(())
    }

    fn visit_boundary(&mut self, boundary: &exprs::Boundary) -> Result<(), CompileError> {
        self.require(Feat::BOUNDARIES, boundary.span)
    }

    fn visit_lookaround(&mut self, lookaround: &exprs::Lookaround) -> Result<(), CompileError> {
        use exprs::LookaroundKind;
        let feature = match lookaround.kind {
            LookaroundKind::Ahead | LookaroundKind::AheadNegative => Feat::LOOKAHEAD,
            LookaroundKind::Behind | LookaroundKind::BehindNegative => Feat::LOOKBEHIND,
        };
        self.require(feature, lookaround.span)?;

        if let flavor @ (RegexFlavor::Rust | RegexFlavor::RE2) = self.flavor() {
            Err(CompileErrorKind::Unsupported(Feature::Lookaround, flavor).at(lookaround.span))
        } else {
            Ok(())
        }
    }

    fn visit_reference(&mut self, reference: &exprs::Reference) -> Result<(), CompileError> {
        self.require(Feat::REFERENCES, reference.span)
    }

    fn visit_range(&mut self, range: &exprs::Range) -> Result<(), CompileError> {
        self.require(Feat::RANGES, range.span)?;

        if range.end.len() <= self.options.max_range_size as usize {
            Ok(())
        } else {
            Err(CompileErrorKind::RangeIsTooBig(self.options.max_range_size).at(range.span))
        }
    }

    fn visit_statement(&mut self, statement: &exprs::Stmt) -> Result<(), CompileError> {
        use exprs::{BooleanSetting as BS, Stmt};
        match statement {
            Stmt::Enable(BS::Lazy, span) => self.require(Feat::LAZY_MODE, *span),
            Stmt::Disable(BS::Unicode, span) => self.require(Feat::ASCII_MODE, *span),
            Stmt::Let(l) => self.require(Feat::VARIABLES, l.name_span),
            Stmt::Test(t) if self.layer > 0 => Err(CompileErrorKind::NestedTest.at(t.span)),
            _ => Ok(()),
        }
    }

    fn visit_regex(&mut self, regex: &exprs::Regex) -> Result<(), CompileError> {
        self.require(Feat::REGEXES, regex.span)
    }

    fn visit_recursion(&mut self, recursion: &exprs::Recursion) -> Result<(), CompileError> {
        self.require(Feat::RECURSION, recursion.span)?;

        if let RegexFlavor::Pcre | RegexFlavor::Ruby = self.flavor() {
            Ok(())
        } else {
            Err(CompileErrorKind::Unsupported(Feature::Recursion, self.flavor()).at(recursion.span))
        }
    }

    fn visit_grapheme(&mut self) -> Result<(), CompileError> {
        self.require(Feat::GRAPHEME, Span::empty())
    }

    fn visit_dot(&mut self) -> Result<(), CompileError> {
        self.require(Feat::DOT, Span::empty())
    }
}
