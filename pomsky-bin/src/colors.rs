use std::io::Write;

use atty::Stream;

/// All supported colors in our format strings
enum Color {
    /// Cyan text, surrounded by `%c.` `.%`
    Cyan,
    /// Green text, surrounded by `%g.` `.%`
    Green,
    /// Magenta text, surrounded by `%m.` `.%`
    Magenta,
    /// Red text, surrounded by `%red.` `.%`
    Red,
    /// Yellow text, surrounded by `%y.` `.%`
    Yellow,
    /// Cyan bold text, surrounded by `%C.` `.%`
    CyanBold,
    /// Green bold text, surrounded by `%G.` `.%`
    GreenBold,
    /// Magenta bold text, surrounded by `%M.` `.%`
    MagentaBold,
    /// Red bold text, surrounded by `%R.` `.%`
    RedBold,
    /// Yellow bold text, surrounded by `%Y.` `.%`
    YellowBold,
}

macro_rules! fprintln {
    (lit $lit:expr) => {{
        $crate::colors::fwriteln($lit, ::atty::Stream::Stdout);
    }};
    ($($args:tt)*) => {{
        $crate::colors::fwriteln(&format!($($args)*), ::atty::Stream::Stderr);
    }};
}

macro_rules! efprintln {
    (lit $lit:expr) => {{
        $crate::colors::fwriteln($lit, ::atty::Stream::Stderr);
    }};
    ($($args:tt)*) => {{
        $crate::colors::fwriteln(&format!($($args)*), ::atty::Stream::Stderr);
    }};
}

pub fn fwriteln(template: &str, stream: Stream) {
    match stream {
        Stream::Stdout => {
            let mut lock = std::io::stdout().lock();
            lock.write_all(format(template, stream).as_bytes()).unwrap();
            lock.write_all(b"\n").unwrap();
        }
        Stream::Stderr => {
            let mut lock = std::io::stderr().lock();
            lock.write_all(format(template, stream).as_bytes()).unwrap();
            lock.write_all(b"\n").unwrap();
        }
        Stream::Stdin => panic!("Can't write to stdin"),
    }
}

pub fn format(mut template: &str, stream: Stream) -> String {
    let supports_color = matches!(
        supports_color::on_cached(stream),
        Some(supports_color::ColorLevel { has_basic: true, .. })
    );
    let mut out = String::new();

    while let Some(next_idx) = template.find('%') {
        let (prev, remaining) = template.split_at(next_idx);
        out.push_str(prev);

        let color = match remaining[1..].chars().next() {
            Some('c') => Color::Cyan,
            Some('g') => Color::Green,
            Some('m') => Color::Magenta,
            Some('r') => Color::Red,
            Some('y') => Color::Yellow,
            Some('C') => Color::CyanBold,
            Some('G') => Color::GreenBold,
            Some('M') => Color::MagentaBold,
            Some('R') => Color::RedBold,
            Some('Y') => Color::YellowBold,
            _ => {
                out.push('%');
                template = &remaining[1..];
                continue;
            }
        };
        if !remaining[2..].starts_with('.') {
            out.push('%');
            template = &remaining[1..];
            continue;
        }
        template = &remaining[3..];

        if supports_color {
            out.push('\x1b');
            out.push_str(match color {
                Color::Cyan => "[36m",
                Color::Green => "[32m",
                Color::Magenta => "[35m",
                Color::Red => "[31m",
                Color::Yellow => "[33m",
                Color::CyanBold => "[36;1m",
                Color::GreenBold => "[32;1m",
                Color::MagentaBold => "[35;1m",
                Color::RedBold => "[31;1m",
                Color::YellowBold => "[33;1m",
            });
        }

        let closing_idx = template.find(".%").unwrap_or(template.len());
        let (prev, remaining) = template.split_at(closing_idx);

        out.push_str(prev);

        if supports_color {
            out.push_str("\x1b[0m");
        }

        template = remaining.strip_prefix(".%").unwrap_or(remaining);
    }

    out.push_str(template);
    out
}
