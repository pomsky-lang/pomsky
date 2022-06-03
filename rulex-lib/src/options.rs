use crate::features::RulexFeatures;

#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    pub max_range_size: u8,
    pub allowed_features: RulexFeatures,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self { max_range_size: 6, allowed_features: Default::default() }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileOptions {
    pub flavor: RegexFlavor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegexFlavor {
    Pcre,
    Python,
    Java,
    JavaScript,
    DotNet,
    Ruby,
    Rust,
}

impl Default for RegexFlavor {
    fn default() -> Self {
        RegexFlavor::Pcre
    }
}
