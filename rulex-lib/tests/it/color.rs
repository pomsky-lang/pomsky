use std::fmt;

use once_cell::sync::OnceCell;

pub(crate) enum Color<T> {
    Red(T),
    Green(T),
    Blue(T),
    Yellow(T),
    NoColor(T),
}

impl<T> Color<T> {
    pub(crate) fn color_if(self, condition: bool) -> Self {
        if condition {
            self
        } else {
            Color::NoColor(self.into_inner())
        }
    }

    pub(crate) fn inner(&self) -> &T {
        match self {
            Color::Red(inner)
            | Color::Green(inner)
            | Color::Blue(inner)
            | Color::Yellow(inner)
            | Color::NoColor(inner) => inner,
        }
    }

    pub(crate) fn into_inner(self) -> T {
        match self {
            Color::Red(inner)
            | Color::Green(inner)
            | Color::Blue(inner)
            | Color::Yellow(inner)
            | Color::NoColor(inner) => inner,
        }
    }

    fn get_markers(&self) -> (&'static str, &'static str) {
        const RED: &str = "\x1B[38;5;9m";
        const GREEN: &str = "\x1B[38;5;10m";
        const BLUE: &str = "\x1B[38;5;14m";
        const YELLOW: &str = "\x1B[38;5;11m";
        const RESET: &str = "\x1B[0m";

        match self {
            Color::Red(_) => (RED, RESET),
            Color::Green(_) => (GREEN, RESET),
            Color::Blue(_) => (BLUE, RESET),
            Color::Yellow(_) => (YELLOW, RESET),
            Color::NoColor(_) => ("", ""),
        }
    }
}

macro_rules! color {
    (~brace $e:expr) => {
        "{}"
    };
    ($id:ident if $cond:expr; $($e:expr),* $(,)?) => {
        color!($id; $($e),*).color_if($cond)
    };
    ($id:ident; $e:expr) => {
        $crate::color::Color::$id($e)
    };
    ($id:ident; $($e:expr),*) => {
        $crate::color::Color::$id(format!(concat!($(color!(~brace $e)),*), $($e),*))
    };
}

static ATTY: OnceCell<bool> = OnceCell::new();

impl<T: fmt::Display> fmt::Display for Color<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &atty = ATTY.get_or_init(|| atty::is(atty::Stream::Stdout));

        if atty {
            let (m1, m2) = self.get_markers();
            write!(f, "{m1}{}{m2}", self.inner())
        } else {
            self.inner().fmt(f)
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Color<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &atty =
            ATTY.get_or_init(|| atty::is(atty::Stream::Stderr) && atty::is(atty::Stream::Stdout));

        if atty {
            let (m1, m2) = self.get_markers();
            write!(f, "{m1}{:?}{m2}", self.inner())
        } else {
            self.inner().fmt(f)
        }
    }
}
