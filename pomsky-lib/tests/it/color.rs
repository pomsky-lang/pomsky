use std::{fmt, io::IsTerminal, sync::OnceLock};

pub(crate) enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

pub(crate) struct Colored<T> {
    pub(crate) inner: T,
    color: Option<Color>,
}

/// A `Display` wrapper for two values to format after another.
pub(crate) struct D2<A, B>(pub(crate) A, pub(crate) B);

impl<A: fmt::Display, B: fmt::Display> fmt::Display for D2<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)?;
        self.1.fmt(f)
    }
}

pub(crate) mod prelude {
    use super::{Color, Colored};

    pub(crate) fn red<T>(inner: T) -> Colored<T> {
        (Color::Red, inner).into()
    }

    pub(crate) fn green<T>(inner: T) -> Colored<T> {
        (Color::Green, inner).into()
    }

    pub(crate) fn blue<T>(inner: T) -> Colored<T> {
        (Color::Blue, inner).into()
    }

    pub(crate) fn yellow<T>(inner: T) -> Colored<T> {
        (Color::Yellow, inner).into()
    }

    pub(crate) fn no_color<T>(inner: T) -> Colored<T> {
        Colored { color: None, inner }
    }
}

impl<T> Colored<T> {
    fn get_markers(&self) -> (&'static str, &'static str) {
        const RED: &str = "\x1B[38;5;9m";
        const GREEN: &str = "\x1B[38;5;10m";
        const BLUE: &str = "\x1B[38;5;14m";
        const YELLOW: &str = "\x1B[38;5;11m";
        const RESET: &str = "\x1B[0m";

        match self.color {
            None => ("", ""),
            Some(Color::Red) => (RED, RESET),
            Some(Color::Green) => (GREEN, RESET),
            Some(Color::Blue) => (BLUE, RESET),
            Some(Color::Yellow) => (YELLOW, RESET),
        }
    }
}

impl<T> From<(Color, T)> for Colored<T> {
    fn from((color, inner): (Color, T)) -> Self {
        Colored { inner, color: Some(color) }
    }
}

fn stdout_is_terminal() -> bool {
    static IS_TERMINAL: OnceLock<bool> = OnceLock::new();

    *IS_TERMINAL.get_or_init(|| std::io::stdout().is_terminal())
}

impl<T: fmt::Display> fmt::Display for Colored<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if stdout_is_terminal() {
            let (m1, m2) = self.get_markers();
            write!(f, "{m1}{}{m2}", self.inner)
        } else {
            self.inner.fmt(f)
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Colored<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if stdout_is_terminal() {
            let (m1, m2) = self.get_markers();
            write!(f, "{m1}{:?}{m2}", self.inner)
        } else {
            self.inner.fmt(f)
        }
    }
}
