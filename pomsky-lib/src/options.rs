//! Contains parser and compiler options passed to pomsky.

use crate::features::PomskyFeatures;

/// Options passed to the pomsky compiler
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct CompileOptions {
    /// The targeted regex flavor. Pomsky makes sure that the emitted regex is
    /// compatible with this flavor.
    pub flavor: RegexFlavor,

    /// The maximum number of digits in a `range` expression. Defaults to 6.
    ///
    /// Note that if you increase this number, the time needed to compile a
    /// pomsky expression may increase exponentially. If you parse untrusted
    /// input, this can be used for a DoS attack.
    pub max_range_size: u8,

    /// Allowed pomsky features. By default, all features are allowed.
    pub allowed_features: PomskyFeatures,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            flavor: RegexFlavor::default(),
            max_range_size: 6,
            allowed_features: PomskyFeatures::default(),
        }
    }
}

/// A regex flavor is a regex engine or a set of regex engines that are similar
/// enough that they can be treated the same for the purpose of writing regexes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub enum RegexFlavor {
    /// PCRE and PCRE2
    #[default]
    Pcre,
    /// Python's `re` module
    Python,
    /// The `java.util.regex.Pattern` class
    Java,
    /// JavaScript (ECMAScript) built-in regular expressions
    JavaScript,
    /// .NET `Regex` class from the namespace `System.Text.RegularExpressions`
    DotNet,
    /// Ruby built-in regular expressions
    Ruby,
    /// The Rust `regex` crate
    Rust,
    /// The RE2 library
    RE2,
}
