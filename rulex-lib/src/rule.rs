use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    compile::{Compile, CompileState},
    error::{CompileError, ParseError},
    grapheme::Grapheme,
    group::Group,
    lookaround::Lookaround,
    options::{CompileOptions, ParseOptions},
    repetition::Repetition,
};

/// A parsed rulex expression, which might contain more sub-expressions.
#[derive(Clone, PartialEq, Eq)]
pub enum Rulex<'i> {
    /// A string literal
    Literal(&'i str),
    /// A character class
    CharClass(CharClass<'i>),
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
}

impl<'i> Rulex<'i> {
    pub fn parse(input: &'i str, _options: ParseOptions) -> Result<Self, ParseError> {
        crate::parse::parse(input)
    }

    pub fn compile(&self, options: CompileOptions) -> Result<String, CompileError> {
        let mut buf = String::new();
        self.comp(options, &mut CompileState::new(), &mut buf)?;
        Ok(buf)
    }

    pub fn parse_and_compile(input: &str, options: CompileOptions) -> Result<String, CompileError> {
        let parsed = Rulex::parse(input, options.parse_options)?;
        let mut buf = String::new();
        parsed.comp(options, &mut CompileState::new(), &mut buf)?;
        Ok(buf)
    }

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        match self {
            Rulex::Literal(l) => l.chars().nth(1).is_some(),
            Rulex::Alternation(_) => true,
            Rulex::Group(g) => g.needs_parens_before_repetition(),
            Rulex::CharClass(_)
            | Rulex::Grapheme(_)
            | Rulex::Repetition(_)
            | Rulex::Boundary(_)
            | Rulex::Lookaround(_) => false,
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
            | Rulex::Lookaround(_) => false,
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rulex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(arg0) => arg0.fmt(f),
            Self::CharClass(arg0) => arg0.fmt(f),
            Rulex::Grapheme(arg0) => arg0.fmt(f),
            Self::Group(arg0) => arg0.fmt(f),
            Self::Alternation(arg0) => arg0.fmt(f),
            Self::Repetition(arg0) => arg0.fmt(f),
            Self::Boundary(arg0) => arg0.fmt(f),
            Self::Lookaround(arg0) => arg0.fmt(f),
        }
    }
}

impl Compile for Rulex<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut crate::compile::CompileState,
        buf: &mut String,
    ) -> crate::compile::CompileResult {
        match self {
            Rulex::Literal(l) => l.comp(options, state, buf),
            Rulex::CharClass(c) => c.comp(options, state, buf),
            Rulex::Group(g) => g.comp(options, state, buf),
            Rulex::Grapheme(g) => g.comp(options, state, buf),
            Rulex::Alternation(a) => a.comp(options, state, buf),
            Rulex::Repetition(r) => r.comp(options, state, buf),
            Rulex::Boundary(b) => b.comp(options, state, buf),
            Rulex::Lookaround(l) => l.comp(options, state, buf),
        }
    }
}
