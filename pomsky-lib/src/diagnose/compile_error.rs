use pomsky_syntax::{
    Span,
    diagnose::{ParseError, ParseErrorKind},
};

use crate::{exprs::char_class::RegexCharSetItem, options::RegexFlavor};

use super::{Diagnostic, Feature};

/// An error that can occur during parsing or compiling
#[derive(Debug, Clone)]
pub(crate) struct CompileError {
    pub(super) kind: CompileErrorKind,
    pub(super) span: Span,
}

impl CompileError {
    pub(crate) fn set_missing_span(&mut self, span: Span) {
        if self.span.is_empty() {
            self.span = span;
        }
    }

    /// Create a [Diagnostic] from this error.
    #[must_use]
    pub fn diagnostic(&self, source_code: &str) -> Diagnostic {
        Diagnostic::from_compile_error(self, source_code)
    }
}

impl std::error::Error for CompileError {}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{}\n  at {start}..{end}", self.kind)
        } else {
            self.kind.fmt(f)
        }
    }
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        CompileError { kind: CompileErrorKind::ParseError(e.kind), span: e.span }
    }
}

/// An error kind (without span) that can occur during parsing or compiling
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum CompileErrorKind {
    ParseError(ParseErrorKind),
    Unsupported(Feature, RegexFlavor),
    UnsupportedPomskySyntax(UnsupportedError),
    HugeReference,
    UnknownReferenceNumber(i32),
    UnknownReferenceName {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },
    NameUsedMultipleTimes(String),
    EmptyClassNegated {
        group1: RegexCharSetItem,
        group2: RegexCharSetItem,
    },
    // [!v]
    // TODO: This should be allowed when there's only one shorthand in the character set
    NegatedHorizVertSpace,
    IllegalNegation {
        kind: IllegalNegationKind,
    },
    CaptureInLet,
    ReferenceInLet,
    RelativeRefZero,
    UnknownVariable {
        found: Box<str>,
        #[cfg(feature = "suggestions")]
        similar: Option<Box<str>>,
    },
    RecursiveVariable,
    RangeIsTooBig(u8),
    NegativeShorthandInAsciiMode,
    UnicodeInAsciiMode,
    DotNetNumberedRefWithMixedGroups,
    RubyLookaheadInLookbehind {
        was_word_boundary: bool,
    },
    UnsupportedInLookbehind {
        flavor: RegexFlavor,
        feature: Feature,
    },
    LookbehindNotConstantLength {
        flavor: RegexFlavor,
    },
    NestedTest,
    InfiniteRecursion,
    BadIntersection,
    EmptyIntersection,
}

impl CompileErrorKind {
    pub(crate) fn at(self, span: Span) -> CompileError {
        CompileError { kind: self, span }
    }

    pub(crate) fn unsupported_specific_prop_in(flavor: RegexFlavor) -> CompileErrorKind {
        CompileErrorKind::Unsupported(Feature::SpecificUnicodeProp, flavor)
    }
}

impl std::error::Error for CompileErrorKind {}

impl core::fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompileErrorKind::ParseError(kind) => write!(f, "Parse error: {kind}"),
            CompileErrorKind::Unsupported(feature, flavor) => match feature {
                Feature::SpecificUnicodeProp => write!(
                    f,
                    "This Unicode property is not supported in the `{flavor:?}` regex flavor"
                ),
                Feature::LargeCodePointInCharClass(c) => write!(
                    f,
                    "Code point {c:?} is too large. Code points above U+FFFF \
                    may not appear in character classes in the `{flavor:?}` flavor"
                ),
                Feature::UnicodeWordBoundaries => write!(
                    f,
                    "In the `{flavor:?}` flavor, word boundaries may only be used when Unicode \
                    is disabled"
                ),
                Feature::ShorthandW => write!(
                    f,
                    "In the `{flavor:?}` flavor, `word` can only be used when Unicode is disabled"
                ),
                Feature::NegativeShorthandW => write!(
                    f,
                    "In the `{flavor:?}` flavor, `word` can only be negated in a character class \
                    when Unicode is disabled"
                ),
                Feature::NegativeShorthandS => write!(
                    f,
                    "In the `{flavor:?}` flavor, `space` can only be negated in a character class \
                    when Unicode is disabled"
                ),
                _ => write!(
                    f,
                    "Unsupported feature `{}` in the `{flavor:?}` regex flavor",
                    feature.name(),
                ),
            },
            CompileErrorKind::UnsupportedPomskySyntax(inner) => inner.fmt(f),
            CompileErrorKind::HugeReference => {
                write!(f, "Group references this large aren't supported")
            }
            CompileErrorKind::UnknownReferenceNumber(group) => {
                write!(f, "Reference to unknown group. There is no group number {group}")
            }
            CompileErrorKind::UnknownReferenceName { found, .. } => {
                write!(f, "Reference to unknown group. There is no group named `{found}`")
            }
            CompileErrorKind::NameUsedMultipleTimes(name) => {
                write!(f, "Group name `{name}` used multiple times")
            }
            CompileErrorKind::EmptyClassNegated { .. } => {
                write!(f, "This negated character class matches nothing")
            }
            CompileErrorKind::NegatedHorizVertSpace => {
                write!(f, "horiz_space and vert_space can't be negated within a character class")
            }
            CompileErrorKind::IllegalNegation { kind } => kind.fmt(f),
            CompileErrorKind::CaptureInLet => {
                write!(f, "Capturing groups within `let` statements are currently not supported")
            }
            CompileErrorKind::ReferenceInLet => {
                write!(f, "References within `let` statements are currently not supported")
            }
            CompileErrorKind::RelativeRefZero => {
                write!(f, "Relative references can't be 0")
            }
            CompileErrorKind::UnknownVariable { found, .. } => {
                write!(f, "Variable `{found}` doesn't exist")
            }
            CompileErrorKind::RecursiveVariable => write!(f, "Variables can't be used recursively"),
            CompileErrorKind::RangeIsTooBig(digits) => {
                write!(f, "Range is too big, it isn't allowed to contain more than {digits} digits")
            }
            CompileErrorKind::NegativeShorthandInAsciiMode => {
                write!(f, "Shorthands can't be negated when Unicode is disabled")
            }
            CompileErrorKind::UnicodeInAsciiMode => {
                write!(f, "Unicode properties can't be used when Unicode is disabled")
            }
            CompileErrorKind::DotNetNumberedRefWithMixedGroups => write!(
                f,
                "In the .NET flavor, numeric references are forbidden when there are both named \
                and unnamed capturing groups. This is because .NET counts named and unnamed \
                capturing groups separately, which is inconsistent with other flavors."
            ),
            CompileErrorKind::RubyLookaheadInLookbehind { was_word_boundary: false } => {
                write!(f, "In the Ruby flavor, lookahead is not allowed within lookbehind")
            }
            CompileErrorKind::RubyLookaheadInLookbehind { was_word_boundary: true } => write!(
                f,
                "In the Ruby flavor, `<` and `>` word boundaries are not allowed within lookbehind"
            ),
            CompileErrorKind::NestedTest => {
                write!(f, "Unit tests may only appear at the top level of the expression")
            }
            CompileErrorKind::UnsupportedInLookbehind { flavor, feature } => {
                write!(
                    f,
                    "Feature `{feature:?}` is not supported within lookbehinds in the {flavor:?} flavor"
                )
            }
            CompileErrorKind::LookbehindNotConstantLength { flavor } => match flavor {
                RegexFlavor::Pcre | RegexFlavor::Python => write!(
                    f,
                    "In the {flavor:?} flavor, lookbehinds must have a {} length",
                    if flavor == &RegexFlavor::Pcre { "bounded" } else { "constant" }
                ),
                _ => write!(f, "This kind of lookbehind is not supported in the {flavor:?} flavor"),
            },
            CompileErrorKind::InfiniteRecursion => write!(f, "This recursion never terminates"),
            CompileErrorKind::BadIntersection => write!(
                f,
                "Intersecting these expressions is not supported. Only character sets \
                can be intersected."
            ),
            CompileErrorKind::EmptyIntersection => {
                write!(f, "Intersection of expressions that do not overlap")
            }
        }
    }
}

/// An error that indicates that an unsupported feature was used.
///
/// See [`crate::features::PomskyFeatures`] for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum UnsupportedError {
    Grapheme,
    NumberedGroups,
    NamedGroups,
    AtomicGroups,
    References,
    LazyMode,
    AsciiMode,
    Ranges,
    Variables,
    Lookahead,
    Lookbehind,
    Boundaries,
    Regexes,
    Dot,
    Recursion,
    Intersection,
}

impl std::error::Error for UnsupportedError {}

impl core::fmt::Display for UnsupportedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = match self {
            UnsupportedError::Grapheme => "Grapheme isn't supported",
            UnsupportedError::NumberedGroups => "Numbered capturing groups aren't supported",
            UnsupportedError::NamedGroups => "Named capturing groups aren't supported",
            UnsupportedError::AtomicGroups => "Atomic groups aren't supported",
            UnsupportedError::References => "References aren't supported",
            UnsupportedError::LazyMode => "Lazy mode isn't supported",
            UnsupportedError::AsciiMode => "Disabling Unicode isn't supported",
            UnsupportedError::Ranges => "Ranges aren't supported",
            UnsupportedError::Variables => "Variables aren't supported",
            UnsupportedError::Lookahead => "Lookahead isn't supported",
            UnsupportedError::Lookbehind => "Lookbehind isn't supported",
            UnsupportedError::Boundaries => "Word boundaries aren't supported",
            UnsupportedError::Regexes => "Unescaped regexes aren't supported",
            UnsupportedError::Dot => "The dot isn't supported",
            UnsupportedError::Recursion => "Recursion isn't supported",
            UnsupportedError::Intersection => "Intersection isn't supported",
        };

        f.write_str(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum IllegalNegationKind {
    Literal(String),
    DotNetChar(char),
    Unescaped,
    Grapheme,
    Dot,
    Group,
    Alternation,
    Repetition,
    Reference,
    Recursion,
    Boundary,
}

impl core::fmt::Display for IllegalNegationKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            IllegalNegationKind::Literal(s) => {
                return write!(f, "String literal {s:?} can't be negated");
            }
            &IllegalNegationKind::DotNetChar(c) => {
                return write!(
                    f,
                    "Code point {c:?} (U+{:X}) can't be negated in the .NET flavor, because it is \
                    above U+FFFF, and is therefore incorrectly treated as two code points by .NET.",
                    c as u32
                );
            }
            IllegalNegationKind::Unescaped => "An inline regex",
            IllegalNegationKind::Grapheme => "A grapheme",
            IllegalNegationKind::Dot => "The dot",
            IllegalNegationKind::Group => "This group",
            IllegalNegationKind::Alternation => "This alternation",
            IllegalNegationKind::Repetition => "A repetition",
            IllegalNegationKind::Reference => "A reference",
            IllegalNegationKind::Recursion => "Recursion",
            IllegalNegationKind::Boundary => "This boundary",
        };
        f.write_str(s)?;
        f.write_str(" can't be negated")
    }
}
