use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    compile::{Compile, CompileResult, CompileState, Transform, TransformState},
    error::{CompileError, ParseError},
    grapheme::Grapheme,
    group::Group,
    literal::Literal,
    lookaround::Lookaround,
    options::{CompileOptions, ParseOptions},
    range::Range,
    reference::Reference,
    repetition::Repetition,
    span::Span,
};

/// A parsed rulex expression, which might contain more sub-expressions.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rulex<'i> {
    /// A string literal
    Literal(Literal<'i>),
    /// A character class
    CharClass(CharClass),
    /// A Unicode grapheme
    Grapheme(Grapheme),
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(Group<'i>),
    /// An alternation, i.e. a list of alternatives; at least one of them has to match.
    Alternation(Alternation<'i>),
    /// A repetition, i.e. a expression that must be repeated. The number of required repetitions is
    /// constrained by a lower and possibly an upper bound.
    Repetition(Box<Repetition<'i>>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(Boundary),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<Lookaround<'i>>),
    /// A backreference or forward reference.
    Reference(Reference<'i>),
    /// A range of integers
    Range(Range),
}

impl<'i> Rulex<'i> {
    pub fn parse(input: &'i str, _options: ParseOptions) -> Result<Self, ParseError> {
        crate::parse::parse(input)
    }

    pub fn compile(mut self, options: CompileOptions) -> Result<String, CompileError> {
        let capturing_groups = self.count_capturing_groups();
        let mut transform_state = TransformState::new(capturing_groups);
        self.transform(options, &mut transform_state)?;

        let mut buf = String::new();
        let mut state = CompileState::new();
        self.comp(options, &mut state, &mut buf)?;
        state.check_validity()?;
        Ok(buf)
    }

    pub fn parse_and_compile(input: &str, options: CompileOptions) -> Result<String, CompileError> {
        let parsed = Rulex::parse(input, options.parse_options)?;
        parsed.compile(options)
    }

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        match self {
            Rulex::Literal(l) => l.needs_parens_before_repetition(),
            Rulex::Group(g) => g.needs_parens_before_repetition(),
            Rulex::Alternation(_) | Rulex::Range(_) => true,
            Rulex::CharClass(_)
            | Rulex::Grapheme(_)
            | Rulex::Repetition(_)
            | Rulex::Boundary(_)
            | Rulex::Lookaround(_)
            | Rulex::Reference(_) => false,
        }
    }

    pub(crate) fn needs_parens_in_group(&self) -> bool {
        match self {
            Rulex::Alternation(_) => true,
            Rulex::Literal(_)
            | Rulex::Group(_)
            | Rulex::CharClass(_)
            | Rulex::Grapheme(_)
            | Rulex::Repetition(_)
            | Rulex::Boundary(_)
            | Rulex::Lookaround(_)
            | Rulex::Reference(_)
            | Rulex::Range(_) => false,
        }
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Rulex::Literal(l) => l.span,
            Rulex::CharClass(c) => c.span,
            Rulex::Grapheme(g) => g.span,
            Rulex::Group(g) => g.span,
            Rulex::Alternation(a) => a.span,
            Rulex::Repetition(r) => r.span,
            Rulex::Boundary(b) => b.span,
            Rulex::Lookaround(l) => l.span,
            Rulex::Reference(r) => r.span,
            Rulex::Range(r) => r.span,
        }
    }

    pub(crate) fn count_capturing_groups(&self) -> u32 {
        match self {
            Rulex::Literal(_) => 0,
            Rulex::CharClass(_) => 0,
            Rulex::Grapheme(_) => 0,
            Rulex::Group(g) => (if g.is_capturing() { 1 } else { 0 }) + g.count_capturing_groups(),
            Rulex::Alternation(a) => a.count_capturing_groups(),
            Rulex::Repetition(r) => r.count_capturing_groups(),
            Rulex::Boundary(_) => 0,
            Rulex::Lookaround(l) => l.count_capturing_groups(),
            Rulex::Reference(_) => 0,
            Rulex::Range(_) => 0,
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rulex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Rulex::Literal(arg0) => arg0.fmt(f),
            Rulex::CharClass(arg0) => arg0.fmt(f),
            Rulex::Grapheme(arg0) => arg0.fmt(f),
            Rulex::Group(arg0) => arg0.fmt(f),
            Rulex::Alternation(arg0) => arg0.fmt(f),
            Rulex::Repetition(arg0) => arg0.fmt(f),
            Rulex::Boundary(arg0) => arg0.fmt(f),
            Rulex::Lookaround(arg0) => arg0.fmt(f),
            Rulex::Reference(arg0) => arg0.fmt(f),
            Rulex::Range(arg0) => arg0.fmt(f),
        }
    }
}

impl Transform for Rulex<'_> {
    fn transform(&mut self, options: CompileOptions, state: &mut TransformState) -> CompileResult {
        match self {
            Rulex::Literal(l) => l.transform(options, state),
            Rulex::CharClass(c) => c.transform(options, state),
            Rulex::Grapheme(g) => g.transform(options, state),
            Rulex::Group(g) => g.transform(options, state),
            Rulex::Alternation(a) => a.transform(options, state),
            Rulex::Repetition(r) => r.transform(options, state),
            Rulex::Boundary(b) => b.transform(options, state),
            Rulex::Lookaround(l) => l.transform(options, state),
            Rulex::Reference(r) => r.transform(options, state),
            Rulex::Range(r) => {
                *self = r.transform(options, state)?;
                Ok(())
            }
        }
    }
}

impl Compile for Rulex<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        match self {
            Rulex::Literal(l) => l.comp(options, state, buf),
            Rulex::CharClass(c) => c.comp(options, state, buf),
            Rulex::Group(g) => g.comp(options, state, buf),
            Rulex::Grapheme(g) => g.comp(options, state, buf),
            Rulex::Alternation(a) => a.comp(options, state, buf),
            Rulex::Repetition(r) => r.comp(options, state, buf),
            Rulex::Boundary(b) => b.comp(options, state, buf),
            Rulex::Lookaround(l) => l.comp(options, state, buf),
            Rulex::Reference(r) => r.comp(options, state, buf),
            Rulex::Range(_) => {
                panic!("Tried to compile a range that hasn't been transformed yet")
            }
        }
    }
}
