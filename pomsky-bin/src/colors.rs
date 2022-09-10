macro_rules! red_bold {
    ($stream:ident, $e:expr) => {
        $e.if_supports_color(atty::Stream::$stream, |t| t.style(Style::new().bright_red().bold()))
    };
}

macro_rules! yellow_bold {
    ($stream:ident, $e:expr) => {
        $e.if_supports_color(atty::Stream::$stream, |t| t.style(Style::new().yellow().bold()))
    };
}

macro_rules! cyan_bold {
    ($stream:ident, $e:expr) => {
        $e.if_supports_color(atty::Stream::$stream, |t| t.style(Style::new().cyan().bold()))
    };
}

macro_rules! yellow {
    ($stream:expr, $e:expr) => {
        $e.if_supports_color($stream, |t| t.yellow())
    };
}

macro_rules! green {
    ($stream:expr, $e:expr) => {
        $e.if_supports_color($stream, |t| t.green())
    };
}
