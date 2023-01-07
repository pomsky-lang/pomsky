/// A regex feature, which might not be supported in every regex flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Feature {
    /// Named capturing groups, e.g. `(?<name>group)`
    NamedCaptureGroups,
    /// Atomic groups, e.g. `(?>group)`
    AtomicGroups,
    /// Lookahead or lookbehind, e.g. `(?=lookahead)`
    Lookaround,
    /// A single grapheme cluster, `\X`
    Grapheme,
    /// Unicode blocks, e.g. `\p{InBasic_Latin}`
    UnicodeBlock,
    /// Unicode properties, e.g. `\p{Whitespace}`
    UnicodeProp,
    /// Backreferences, e.g. `\4`
    Backreference,
    /// Forward references. They're like backreferences, but refer to a group
    /// that syntactically appears _after_ the reference
    ForwardReference,
    /// A numeric reference relative to the current position, e.g. `\k<-2>`.
    ///
    /// Note that this enum variant is currently unused, because relative
    /// references are converted to absolute references by Pomsky.
    // TODO: maybe remove in next major version
    RelativeReference,
    /// A relative reference with a relative index of 0 or higher, e.g. `\k<-0>`
    /// or `\k<+3>`. These aren't supported in any regex engine that I know
    /// of.
    ///
    /// Note that this enum variant is currently unused, because relative
    /// references are converted to absolute references by Pomsky.
    // TODO: maybe remove in next major version
    NonNegativeRelativeReference,
    /// Negative `\w` shorthand, i.e. `[\W]`. This is not supported in
    /// JavaScript when polyfilling Unicode support for `\w` and `\d`.
    NegativeShorthandW,
}

impl Feature {
    pub(super) fn name(self) -> &'static str {
        match self {
            Feature::NamedCaptureGroups => "named capturing groups",
            Feature::AtomicGroups => "atomic groups",
            Feature::Lookaround => "lookahead/behind",
            Feature::Grapheme => "grapheme cluster matcher (\\X)",
            Feature::UnicodeBlock => "Unicode blocks (\\p{InBlock})",
            Feature::UnicodeProp => "Unicode properties (\\p{Property})",
            Feature::Backreference => "backreference",
            Feature::ForwardReference => "forward reference",
            Feature::RelativeReference => "relative backreference",
            Feature::NonNegativeRelativeReference => "non-negative relative backreference",
            Feature::NegativeShorthandW => "negative `\\w` shorthand in character class",
        }
    }
}
