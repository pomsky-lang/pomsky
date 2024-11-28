/// A regex feature, which might not be supported in every regex flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Feature {
    /// Atomic groups, e.g. `(?>group)`
    AtomicGroups,
    /// Lookahead or lookbehind, e.g. `(?=lookahead)`
    Lookaround,
    /// A single grapheme cluster, `\X`
    Grapheme,
    /// Unicode scripts, e.g. `\p{Latin}`
    UnicodeScript,
    /// Unicode blocks, e.g. `\p{InBasic_Latin}`
    UnicodeBlock,
    /// Unicode properties, e.g. `\p{Whitespace}`
    UnicodeProp,
    /// A specific Unicode properties is not supported, even though most are,
    /// e.g. `\p{Bidi_Mirrored}` in Ruby
    SpecificUnicodeProp,
    /// Backreferences, e.g. `\4`
    Backreference,
    /// Forward references. They're like backreferences, but refer to a group
    /// that syntactically appears _after_ the reference
    ForwardReference,
    /// Negative `\w` shorthand, i.e. `[\W]`. This is not supported in
    /// JavaScript when polyfilling Unicode support for `\w` and `\d`.
    NegativeShorthandW,
    /// Negative `\s` shorthand, i.e. `[\S]`. This is not supported in
    /// RE2 when polyfilling Unicode support for `\s`.
    NegativeShorthandS,
    /// `\w` shorthand. This is not supported in RE2: It cannot polyfill Unicode
    /// support because `\p{Alphabetic}` is not supported
    ShorthandW,
    /// Having backreferences to both named and numbered groups is not supported
    /// in Ruby.
    MixedReferences,
    /// Lookarounds can't be repeated in Ruby, even when wrapped in a group
    RepeatedAssertion,
    /// Code points above U+FFFF in char classes
    LargeCodePointInCharClass(char),
    /// Recursion is only supported in PCRE and Ruby at the moment
    Recursion,
    /// Word boundaries in Unicode mode
    UnicodeWordBoundaries,
    /// Word start and word end is not supported in RE2
    WordStartEnd,
    /// Unicode script extensions, e.g. `[scx:Greek]`
    ScriptExtensions,
}

impl Feature {
    pub(super) fn name(self) -> &'static str {
        match self {
            Feature::AtomicGroups => "atomic groups",
            Feature::Lookaround => "lookahead/behind",
            Feature::Grapheme => "grapheme cluster matcher (\\X)",
            Feature::UnicodeScript => "Unicode scripts (\\p{Script})",
            Feature::UnicodeBlock => "Unicode blocks (\\p{InBlock})",
            Feature::UnicodeProp => "Unicode properties (\\p{Property})",
            Feature::SpecificUnicodeProp => "This particular Unicode property",
            Feature::Backreference => "backreference",
            Feature::ForwardReference => "forward reference",
            Feature::NegativeShorthandW => "negative \\w shorthand in character class",
            Feature::NegativeShorthandS => "negative \\s shorthand in character class",
            Feature::ShorthandW => "\\w shorthand",
            Feature::MixedReferences => "references to both named and numbered groups",
            Feature::RepeatedAssertion => "single repeated assertion",
            Feature::LargeCodePointInCharClass(_) => "code points above U+FFFF in char classes",
            Feature::Recursion => "recursion",
            Feature::UnicodeWordBoundaries => "word boundaries in Unicode mode",
            Feature::WordStartEnd => "word start and word end",
            Feature::ScriptExtensions => "Unicode script extensions",
        }
    }
}
