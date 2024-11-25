use std::{fmt, io::IsTerminal, sync::OnceLock};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

#[derive(Clone, Copy)]
pub(crate) struct Colored<T> {
    pub(crate) inner: T,
    color: ColorOption,
}

#[derive(Clone, Copy)]
enum ColorOption {
    Fg(Color),
    Bg(Color),
    None,
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
        Colored { color: super::ColorOption::None, inner }
    }
}

impl<T> Colored<T> {
    fn get_markers(&self) -> (&'static str, &'static str) {
        const RED: &str = "\x1B[38;5;9m";
        const GREEN: &str = "\x1B[38;5;10m";
        const BLUE: &str = "\x1B[38;5;14m";
        const YELLOW: &str = "\x1B[38;5;11m";
        const RED_BG: &str = "\x1B[48;2;150;0;0m";
        const GREEN_BG: &str = "\x1B[48;2;0;100;0m";
        const BLUE_BG: &str = "\x1B[48;2;0;80;80m";
        const YELLOW_BG: &str = "\x1B[48;2;80;80;0m";
        const RESET: &str = "\x1B[0m";

        match self.color {
            ColorOption::None => ("", ""),
            ColorOption::Fg(Color::Red) => (RED, RESET),
            ColorOption::Fg(Color::Green) => (GREEN, RESET),
            ColorOption::Fg(Color::Blue) => (BLUE, RESET),
            ColorOption::Fg(Color::Yellow) => (YELLOW, RESET),
            ColorOption::Bg(Color::Red) => (RED_BG, RESET),
            ColorOption::Bg(Color::Green) => (GREEN_BG, RESET),
            ColorOption::Bg(Color::Blue) => (BLUE_BG, RESET),
            ColorOption::Bg(Color::Yellow) => (YELLOW_BG, RESET),
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Colored<U> {
        Colored { inner: f(self.inner), color: self.color }
    }

    pub fn bg(self) -> Colored<T> {
        let color = match self.color {
            ColorOption::Fg(color) | ColorOption::Bg(color) => ColorOption::Bg(color),
            ColorOption::None => ColorOption::None,
        };
        Colored { color, ..self }
    }

    pub fn iff(self, condition: bool) -> Colored<T> {
        if condition {
            self
        } else {
            Colored { inner: self.inner, color: ColorOption::None }
        }
    }
}

impl<T> From<(Color, T)> for Colored<T> {
    fn from((color, inner): (Color, T)) -> Self {
        Colored { inner, color: ColorOption::Fg(color) }
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
