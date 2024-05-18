//! Contains pomsky features that can be individually enabled and disabled.

use std::fmt;

use pomsky_syntax::Span;

use crate::diagnose::{CompileError, CompileErrorKind, UnsupportedError};

/// A set of enabled pomsky features. By default, all features are enabled.
/// You can disabled specific features with
///
/// ```
/// use pomsky::features::PomskyFeatures;
///
/// let allowed_features = PomskyFeatures::default()
///     .grapheme(false)
///     .variables(false);
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct PomskyFeatures {
    bits: u16,
}

impl fmt::Debug for PomskyFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PomskyFeatures")
            .field("grapheme", &self.supports(Self::GRAPHEME))
            .field("numbered_groups", &self.supports(Self::NUMBERED_GROUPS))
            .field("named_groups", &self.supports(Self::NAMED_GROUPS))
            .field("atomic_groups", &self.supports(Self::ATOMIC_GROUPS))
            .field("references", &self.supports(Self::REFERENCES))
            .field("lazy_mode", &self.supports(Self::LAZY_MODE))
            .field("ascii_mode", &self.supports(Self::ASCII_MODE))
            .field("ranges", &self.supports(Self::RANGES))
            .field("variables", &self.supports(Self::VARIABLES))
            .field("lookahead", &self.supports(Self::LOOKAHEAD))
            .field("lookbehind", &self.supports(Self::LOOKBEHIND))
            .field("boundaries", &self.supports(Self::BOUNDARIES))
            .field("regexes", &self.supports(Self::REGEXES))
            .field("dot", &self.supports(Self::DOT))
            .field("recursion", &self.supports(Self::RECURSION))
            .finish()
    }
}

impl Default for PomskyFeatures {
    fn default() -> Self {
        Self {
            bits: Self::GRAPHEME
                | Self::NUMBERED_GROUPS
                | Self::NAMED_GROUPS
                | Self::REFERENCES
                | Self::LAZY_MODE
                | Self::ASCII_MODE
                | Self::RANGES
                | Self::VARIABLES
                | Self::LOOKAHEAD
                | Self::LOOKBEHIND
                | Self::BOUNDARIES
                | Self::ATOMIC_GROUPS
                | Self::REGEXES
                | Self::DOT
                | Self::RECURSION,
        }
    }
}

impl PomskyFeatures {
    pub(crate) const GRAPHEME: u16 = 1 << 0;
    pub(crate) const NUMBERED_GROUPS: u16 = 1 << 1;
    pub(crate) const NAMED_GROUPS: u16 = 1 << 2;
    pub(crate) const REFERENCES: u16 = 1 << 3;
    pub(crate) const LAZY_MODE: u16 = 1 << 4;
    pub(crate) const ASCII_MODE: u16 = 1 << 5;
    pub(crate) const RANGES: u16 = 1 << 6;
    pub(crate) const VARIABLES: u16 = 1 << 7;
    pub(crate) const LOOKAHEAD: u16 = 1 << 8;
    pub(crate) const LOOKBEHIND: u16 = 1 << 9;
    pub(crate) const BOUNDARIES: u16 = 1 << 10;
    pub(crate) const ATOMIC_GROUPS: u16 = 1 << 11;
    pub(crate) const REGEXES: u16 = 1 << 12;
    pub(crate) const DOT: u16 = 1 << 13;
    pub(crate) const RECURSION: u16 = 1 << 14;

    /// Creates an empty set of features. With this set, all optional features
    /// are disabled.
    ///
    /// Use `PomskyFeatures::default()` instead if you want features to be
    /// enabled by default.
    #[must_use]
    pub fn new() -> Self {
        PomskyFeatures { bits: 0 }
    }

    fn set_bit(&mut self, bit: u16, support: bool) {
        if support {
            self.bits |= bit;
        } else {
            self.bits &= bit ^ 0xFF_FF_u16;
        }
    }

    fn supports(self, bit: u16) -> bool {
        (self.bits & bit) != 0
    }

    pub(super) fn require(self, feature: u16, span: Span) -> Result<(), CompileError> {
        if self.supports(feature) {
            Ok(())
        } else {
            Err(CompileErrorKind::UnsupportedPomskySyntax(match feature {
                Self::GRAPHEME => UnsupportedError::Grapheme,
                Self::NUMBERED_GROUPS => UnsupportedError::NumberedGroups,
                Self::NAMED_GROUPS => UnsupportedError::NamedGroups,
                Self::REFERENCES => UnsupportedError::References,
                Self::LAZY_MODE => UnsupportedError::LazyMode,
                Self::ASCII_MODE => UnsupportedError::AsciiMode,
                Self::RANGES => UnsupportedError::Ranges,
                Self::VARIABLES => UnsupportedError::Variables,
                Self::LOOKAHEAD => UnsupportedError::Lookahead,
                Self::LOOKBEHIND => UnsupportedError::Lookbehind,
                Self::BOUNDARIES => UnsupportedError::Boundaries,
                Self::ATOMIC_GROUPS => UnsupportedError::AtomicGroups,
                Self::REGEXES => UnsupportedError::Regexes,
                Self::DOT => UnsupportedError::Dot,
                Self::RECURSION => UnsupportedError::Recursion,
                _ => panic!("Unknown feature `0x{feature:0x}`"),
            })
            .at(span))
        }
    }

    /// Set support for `Grapheme`
    pub fn grapheme(&mut self, support: bool) -> Self {
        self.set_bit(Self::GRAPHEME, support);
        *self
    }

    /// Set support for numbered groups, e.g. `:('test')`
    pub fn numbered_groups(&mut self, support: bool) -> Self {
        self.set_bit(Self::NUMBERED_GROUPS, support);
        *self
    }

    /// Set support for named groups, e.g. `:test('!')`
    pub fn named_groups(&mut self, support: bool) -> Self {
        self.set_bit(Self::NAMED_GROUPS, support);
        *self
    }

    /// Set support for atomic groups, e.g. `atomic ('if' | 'else' | 'while' |
    /// 'for')`
    pub fn atomic_groups(&mut self, support: bool) -> Self {
        self.set_bit(Self::ATOMIC_GROUPS, support);
        *self
    }

    /// Set support for references, e.g. `::-1` or `:foo() ::foo`
    pub fn references(&mut self, support: bool) -> Self {
        self.set_bit(Self::REFERENCES, support);
        *self
    }

    /// Set support for lazy mode, i.e. `enable lazy;`
    pub fn lazy_mode(&mut self, support: bool) -> Self {
        self.set_bit(Self::LAZY_MODE, support);
        *self
    }

    /// Set support for ascii mode, i.e. `disable unicode;`
    pub fn ascii_mode(&mut self, support: bool) -> Self {
        self.set_bit(Self::ASCII_MODE, support);
        *self
    }

    /// Set support for ranges, e.g. `range '1'-'255'`
    pub fn ranges(&mut self, support: bool) -> Self {
        self.set_bit(Self::RANGES, support);
        *self
    }

    /// Set support for variables, e.g. `let x = 'hello' 'world'?;`
    pub fn variables(&mut self, support: bool) -> Self {
        self.set_bit(Self::VARIABLES, support);
        *self
    }

    /// Set support for lookahead, e.g. `>> 'test'`
    pub fn lookahead(&mut self, support: bool) -> Self {
        self.set_bit(Self::LOOKAHEAD, support);
        *self
    }

    /// Set support for lookbehind, e.g. `<< 'test'`
    pub fn lookbehind(&mut self, support: bool) -> Self {
        self.set_bit(Self::LOOKBEHIND, support);
        *self
    }

    /// Set support for boundaries, i.e. `%` and `!%`
    pub fn boundaries(&mut self, support: bool) -> Self {
        self.set_bit(Self::BOUNDARIES, support);
        *self
    }

    /// Set support for raw regular expressions, e.g. `regex
    /// '.[\p{Alpha}&&[^test]]'`
    pub fn regexes(&mut self, support: bool) -> Self {
        self.set_bit(Self::REGEXES, support);
        *self
    }

    /// Set support for the dot, i.e. `.`
    pub fn dot(&mut self, support: bool) -> Self {
        self.set_bit(Self::DOT, support);
        *self
    }

    /// Set support for recursion
    pub fn recursion(&mut self, support: bool) -> Self {
        self.set_bit(Self::RECURSION, support);
        *self
    }
}

#[test]
fn test_toggles() {
    let features = PomskyFeatures::new()
        .grapheme(true)
        .numbered_groups(true)
        .named_groups(true)
        .atomic_groups(true)
        .references(true)
        .lazy_mode(true)
        .ascii_mode(true)
        .ranges(true)
        .variables(true)
        .lookahead(true)
        .lookbehind(true)
        .boundaries(true)
        .regexes(true)
        .dot(true)
        .recursion(true);

    assert_eq!(features.bits, PomskyFeatures::default().bits);
}
