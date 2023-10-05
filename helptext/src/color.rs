/// All supported colors in the formatting machinery
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Cyan text; alias: c
    Cyan,
    /// Green text; alias: g
    Green,
    /// Magenta text; alias: m
    Magenta,
    /// Red text; alias: r
    Red,
    /// Yellow text; alias: y
    Yellow,
    /// Cyan bold text; alias: C
    CyanBold,
    /// Green bold text; alias: G
    GreenBold,
    /// Magenta bold text; alias: M
    MagentaBold,
    /// Red bold text; alias: R
    RedBold,
    /// Yellow bold text; alias: Y
    YellowBold,
}

#[allow(non_upper_case_globals)]
impl Color {
    pub const c: Color = Color::Cyan;
    pub const g: Color = Color::Green;
    pub const m: Color = Color::Magenta;
    pub const r: Color = Color::Red;
    pub const y: Color = Color::Yellow;

    pub const C: Color = Color::CyanBold;
    pub const G: Color = Color::GreenBold;
    pub const M: Color = Color::MagentaBold;
    pub const R: Color = Color::RedBold;
    pub const Y: Color = Color::YellowBold;

    pub(crate) const ANSI_RESET: &'static str = "\x1b[0m";
    pub(crate) const ANSI_C: &'static str = "\x1b[36m";
    pub(crate) const ANSI_G: &'static str = "\x1b[32m";
    pub(crate) const ANSI_M: &'static str = "\x1b[35m";
    pub(crate) const ANSI_R: &'static str = "\x1b[31m";
    pub(crate) const ANSI_Y: &'static str = "\x1b[33m";
    pub(crate) const ANSI_CB: &'static str = "\x1b[36;1m";
    pub(crate) const ANSI_GB: &'static str = "\x1b[32;1m";
    pub(crate) const ANSI_MB: &'static str = "\x1b[35;1m";
    pub(crate) const ANSI_RB: &'static str = "\x1b[31;1m";
    pub(crate) const ANSI_YB: &'static str = "\x1b[33;1m";
}

impl Color {
    pub fn ansi_code(self) -> &'static str {
        match self {
            Color::Cyan => Self::ANSI_C,
            Color::Green => Self::ANSI_G,
            Color::Magenta => Self::ANSI_M,
            Color::Red => Self::ANSI_R,
            Color::Yellow => Self::ANSI_Y,
            Color::CyanBold => Self::ANSI_CB,
            Color::GreenBold => Self::ANSI_GB,
            Color::MagentaBold => Self::ANSI_MB,
            Color::RedBold => Self::ANSI_RB,
            Color::YellowBold => Self::ANSI_YB,
        }
    }
}
