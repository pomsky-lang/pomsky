//! Contains parser and compiler options passed to rulex.

use crate::features::RulexFeatures;

/// Options passed to the rulex parser
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// The maximum number of digits in a `range` expression. Defaults to 6.
    ///
    /// Note that if you increase this number, the time needed to compile a
    /// rulex may increase exponentially. If you parse untrusted input, this
    /// can be used for a DoS attack.
    pub max_range_size: u8,

    /// Allowed rulex features. By default, all features are allowed.
    pub allowed_features: RulexFeatures,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self { max_range_size: 6, allowed_features: Default::default() }
    }
}

/// Options passed to the rulex compiler
#[derive(Debug, Clone, Copy, Default)]
pub struct CompileOptions {
    /// The targeted regex flavor. Rulex makes sure that the emitted regex is
    /// compatible with this flavor.
    pub flavor: RegexFlavor,
}

/// A regex flavor is a regex engine or a set of regex engines that are similar
/// enough that they can be treated the same for the purpose of writing regexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegexFlavor {
    /// PCRE and PCRE2
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
}

impl Default for RegexFlavor {
    fn default() -> Self {
        RegexFlavor::Pcre
    }
}
