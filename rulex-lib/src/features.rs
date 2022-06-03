use std::fmt;

use crate::{
    error::{ParseError, ParseErrorKind, UnsupportedError},
    span::Span,
};

#[derive(Copy, Clone)]
pub struct RulexFeatures {
    bits: u16,
}

impl fmt::Debug for RulexFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RulexFeatures")
            .field("grapheme", &self.supports(Self::GRAPHEME))
            .field("numbered_groups", &self.supports(Self::NUMBERED_GROUPS))
            .field("named_groups", &self.supports(Self::NAMED_GROUPS))
            .field("references", &self.supports(Self::REFERENCES))
            .field("lazy_mode", &self.supports(Self::LAZY_MODE))
            .field("ranges", &self.supports(Self::RANGES))
            .field("variables", &self.supports(Self::VARIABLES))
            .field("lookahead", &self.supports(Self::LOOKAHEAD))
            .field("lookbehind", &self.supports(Self::LOOKBEHIND))
            .field("boundaries", &self.supports(Self::BOUNDARIES))
            .finish()
    }
}

impl Default for RulexFeatures {
    fn default() -> Self {
        Self {
            bits: Self::GRAPHEME
                | Self::NUMBERED_GROUPS
                | Self::NAMED_GROUPS
                | Self::REFERENCES
                | Self::LAZY_MODE
                | Self::RANGES
                | Self::VARIABLES
                | Self::LOOKAHEAD
                | Self::LOOKBEHIND
                | Self::BOUNDARIES,
        }
    }
}

impl RulexFeatures {
    pub(crate) const GRAPHEME: u16 = 1 << 0;
    pub(crate) const NUMBERED_GROUPS: u16 = 1 << 1;
    pub(crate) const NAMED_GROUPS: u16 = 1 << 2;
    pub(crate) const REFERENCES: u16 = 1 << 3;
    pub(crate) const LAZY_MODE: u16 = 1 << 4;
    pub(crate) const RANGES: u16 = 1 << 5;
    pub(crate) const VARIABLES: u16 = 1 << 6;
    pub(crate) const LOOKAHEAD: u16 = 1 << 7;
    pub(crate) const LOOKBEHIND: u16 = 1 << 8;
    pub(crate) const BOUNDARIES: u16 = 1 << 9;

    fn set_bit(&mut self, bit: u16, support: bool) {
        if support {
            self.bits |= bit;
        } else {
            self.bits &= bit ^ 0xFF_FF_u16;
        }
    }

    fn supports(&self, bit: u16) -> bool {
        (self.bits & bit) != 0
    }

    pub(super) fn require(&self, feature: u16, span: Span) -> Result<(), ParseError> {
        if self.supports(feature) {
            Ok(())
        } else {
            Err(ParseErrorKind::Unsupported(match feature {
                Self::GRAPHEME => UnsupportedError::Grapheme,
                Self::NUMBERED_GROUPS => UnsupportedError::NumberedGroups,
                Self::NAMED_GROUPS => UnsupportedError::NamedGroups,
                Self::REFERENCES => UnsupportedError::References,
                Self::LAZY_MODE => UnsupportedError::LazyMode,
                Self::RANGES => UnsupportedError::Ranges,
                Self::VARIABLES => UnsupportedError::Variables,
                Self::LOOKAHEAD => UnsupportedError::Lookahead,
                Self::LOOKBEHIND => UnsupportedError::Lookbehind,
                Self::BOUNDARIES => UnsupportedError::Boundaries,
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
}
