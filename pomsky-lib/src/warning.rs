//! Compilation warnings

use std::fmt;

/// A warning emitted during compilation
#[derive(Clone)]
pub enum CompileWarning {
    /// Compatibility warning
    Compat(CompatWarning),
}

/// A compatibility warning: Indicates that something might not be supported
/// everywhere
#[derive(Debug, Clone, Copy)]
pub enum CompatWarning {
    /// Lookbehind encountered targeting the JS flavor
    JsLookbehind,
}

impl fmt::Display for CompatWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatWarning::JsLookbehind => {
                f.write_str("Lookbehind is not supported in all browsers, e.g. Safari")
            }
        }
    }
}
