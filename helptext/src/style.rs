/// All supported styles in the formatting machinery
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
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

    /// Underlined text; alias: u
    Underline,
    /// Underlined bold text; alias: U
    UnderlineBold,
}

#[allow(non_upper_case_globals)]
impl Style {
    pub const c: Style = Style::Cyan;
    pub const g: Style = Style::Green;
    pub const m: Style = Style::Magenta;
    pub const r: Style = Style::Red;
    pub const y: Style = Style::Yellow;
    pub const u: Style = Style::Underline;

    pub const C: Style = Style::CyanBold;
    pub const G: Style = Style::GreenBold;
    pub const M: Style = Style::MagentaBold;
    pub const R: Style = Style::RedBold;
    pub const Y: Style = Style::YellowBold;
    pub const U: Style = Style::UnderlineBold;

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

    pub(crate) const ANSI_U: &'static str = "\x1b[4m";
    pub(crate) const ANSI_UB: &'static str = "\x1b[4;1m";
}

impl Style {
    pub fn ansi_code(self) -> &'static str {
        match self {
            Style::Cyan => Self::ANSI_C,
            Style::Green => Self::ANSI_G,
            Style::Magenta => Self::ANSI_M,
            Style::Red => Self::ANSI_R,
            Style::Yellow => Self::ANSI_Y,
            Style::CyanBold => Self::ANSI_CB,
            Style::GreenBold => Self::ANSI_GB,
            Style::MagentaBold => Self::ANSI_MB,
            Style::RedBold => Self::ANSI_RB,
            Style::YellowBold => Self::ANSI_YB,

            Style::Underline => Self::ANSI_U,
            Style::UnderlineBold => Self::ANSI_UB,
        }
    }
}
