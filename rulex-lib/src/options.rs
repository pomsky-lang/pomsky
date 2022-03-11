#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileOptions {
    pub parse_options: ParseOptions,
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
