use std::io::{self, Write};

use atty::Stream;

/// All supported colors in our formatting machinery
#[allow(unused)]
pub(crate) enum Color {
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

#[allow(non_upper_case_globals, dead_code)]
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
}

macro_rules! efprintln {
    ($($args:tt)*) => {{
        $crate::format::fwriteln(text![$($args)*], ::atty::Stream::Stderr);
    }};
}

pub(crate) fn fwriteln(segments: &[Segment], stream: Stream) {
    let supports_color = matches!(
        supports_color::on_cached(stream),
        Some(supports_color::ColorLevel { has_basic: true, .. })
    );

    match stream {
        Stream::Stdout => {
            let mut buf = std::io::stdout().lock();
            for segment in segments {
                let _ = segment.write(&mut buf, supports_color, 0);
            }
            let _ = buf.write_all(b"\n");
        }
        Stream::Stderr => {
            let mut buf = std::io::stderr().lock();
            for segment in segments {
                let _ = segment.write(&mut buf, supports_color, 0);
            }
            let _ = buf.write_all(b"\n");
        }
        Stream::Stdin => panic!("Can't write to stdin"),
    }
}

pub(crate) struct Help<'a>(pub(crate) &'a [HelpSection<'a>]);

impl Help<'_> {
    pub fn write(&self, buf: &mut impl Write, long: bool, colored: bool) -> io::Result<()> {
        for section in self.0 {
            section.write(buf, long, colored, 0, false)?;
        }
        Ok(())
    }
}

pub(crate) enum HelpSection<'a> {
    Short(&'a HelpSection<'a>),
    Long(&'a HelpSection<'a>),

    Text(&'a [Segment<'a>]),
    Name(&'a str, &'a [HelpSection<'a>]),
    Table(TableMode, &'a [(&'a str, &'a [HelpSection<'a>])]),
}

impl HelpSection<'_> {
    fn write(
        &self,
        buf: &mut impl Write,
        long: bool,
        colored: bool,
        indent: usize,
        same_line: bool,
    ) -> io::Result<bool> {
        match *self {
            HelpSection::Short(section) => {
                if !long {
                    return section.write(buf, long, colored, indent, same_line);
                }
            }
            HelpSection::Long(section) => {
                if long {
                    return section.write(buf, long, colored, indent, same_line);
                }
            }
            HelpSection::Text(segments) => {
                if !same_line {
                    buf.write_all(
                        &b"                                                  "[..indent],
                    )?;
                }
                for segment in segments {
                    segment.write(&mut *buf, colored, indent)?;
                }
                buf.write_all(b"\n")?;
                return Ok(true);
            }
            HelpSection::Name(name, sections) => {
                buf.write_all(
                    &b"\n                                                  "[..indent + 1],
                )?;

                if colored {
                    buf.write_all(b"\x1b[33m")?; // yellow
                    buf.write_all(name.as_bytes())?;
                    buf.write_all(b"\x1b[0m")?;
                } else {
                    buf.write_all(name.as_bytes())?;
                }
                let new_indent = indent + 4;
                buf.write_all(b":\n")?;

                let mut line_written = false;
                for section in sections {
                    if line_written {
                        buf.write_all(
                            &b"                                                  "[..new_indent],
                        )?;
                    }
                    line_written |= section.write(buf, long, colored, new_indent, false)?;
                }
                return Ok(line_written);
            }
            HelpSection::Table(style, rows) => {
                let mut is_small = match style {
                    TableMode::Compact => true,
                    TableMode::Auto => !long,
                };
                let col_width = if is_small {
                    // we don't care about Unicode here, help is only available in English
                    rows.iter().map(|&(col1, _)| col1.len()).max().unwrap_or(0) + 2
                } else {
                    0
                };
                if col_width + indent > 50 {
                    is_small = false;
                }
                let new_indent = if is_small { indent + col_width } else { indent + 8 };

                for (i, &(key, value)) in rows.iter().enumerate() {
                    if !long && value.iter().all(|section| matches!(section, HelpSection::Long(_)))
                    {
                        continue;
                    }

                    buf.write_all(
                        &b"                                                  "[..indent],
                    )?;

                    if colored {
                        buf.write_all(b"\x1b[32m")?; // green
                        buf.write_all(key.as_bytes())?;
                        buf.write_all(b"\x1b[0m")?;
                    } else {
                        buf.write_all(key.as_bytes())?;
                    }

                    if is_small {
                        buf.write_all(
                            &b"                                                  "
                                [..col_width - key.len()],
                        )?;
                    } else {
                        buf.write_all(b"\n")?;
                    }

                    let mut line_written = false;
                    for section in value {
                        line_written |= section.write(
                            buf,
                            long,
                            colored,
                            new_indent,
                            is_small && !line_written,
                        )?;
                    }

                    if !is_small && i + 1 < rows.len() {
                        buf.write_all(b"\n")?;
                    }
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Copy, Clone)]
pub enum TableMode {
    Compact,
    Auto,
}

pub(crate) struct Segment<'a> {
    pub(crate) style: Option<Color>,
    pub(crate) text: &'a str,
    pub(crate) ticks: bool,
}

impl<'a> Segment<'a> {
    pub(crate) const fn new(text: &'a str) -> Self {
        Segment { style: None, text, ticks: false }
    }

    pub(super) fn write(
        &self,
        buf: &mut impl Write,
        colored: bool,
        indent: usize,
    ) -> io::Result<()> {
        if let Some(color) = &self.style {
            if colored {
                buf.write_all(match *color {
                    Color::Cyan => b"\x1b[36m",
                    Color::Green => b"\x1b[32m",
                    Color::Magenta => b"\x1b[35m",
                    Color::Red => b"\x1b[31m",
                    Color::Yellow => b"\x1b[33m",
                    Color::CyanBold => b"\x1b[36;1m",
                    Color::GreenBold => b"\x1b[32;1m",
                    Color::MagentaBold => b"\x1b[35;1m",
                    Color::RedBold => b"\x1b[31;1m",
                    Color::YellowBold => b"\x1b[33;1m",
                })?;
            } else if self.ticks {
                buf.write_all(b"`")?;
            }
        }

        let mut is_first_line = true;
        for line in self.text.lines() {
            if !is_first_line {
                buf.write_all(
                    &b"\n                                                  "[..indent + 1],
                )?;
            }
            buf.write_all(line.as_bytes())?;
            is_first_line = false;
        }

        if self.style.is_some() {
            if colored {
                buf.write_all(b"\x1b[0m")?;
            } else if self.ticks {
                buf.write_all(b"`")?;
            }
        }

        Ok(())
    }
}

macro_rules! text_impl {
    // c:"text"
    ([$color_id:ident : $lit:literal $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment {
            style: Some($crate::format::Color::$color_id), text: $lit, ticks: true
        })
    };
    // c:{expr}
    ([$color_id:ident : {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment {
            style: Some($crate::format::Color::$color_id), text: $ex, ticks: true
        })
    };
    // c!"text"
    ([$color_id:ident ! $lit:literal $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment {
            style: Some($crate::format::Color::$color_id), text: $lit, ticks: false
        })
    };
    // c!{expr}
    ([$color_id:ident ! {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment {
            style: Some($crate::format::Color::$color_id), text: $ex, ticks: false
        })
    };
    // "text"
    ([$lit:literal $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment::new($lit))
    };
    // {expr}
    ([{$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        text_impl!([$($rest)*] $($done)*, $crate::format::Segment::new($ex))
    };
    ([], $($done:tt)*) => {
        &[$($done)*]
    };
}

macro_rules! sections_impl {
    // ["text"]
    (@[ [$($text:tt)*] $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::format::HelpSection::Text(text![ $($text)* ])
        )
    };
    // table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ table $mode:ident { $( $key:literal => { $($inner:tt)* } )* } $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::format::HelpSection::Table($crate::format::TableMode::$mode, &[$(
                ( $key, sections!( $($inner)* ) ),
            )*])
        )
    };
    // "NAME" {...}
    (@[ $name:literal { $($inner:tt)* } $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::format::HelpSection::Name($name, sections!( $($inner)* ))
        )
    };
    // Short ["text"]
    (@[ $wrapper:ident [$($text:tt)* ] $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::format::HelpSection::$wrapper(
                &$crate::format::HelpSection::Text(text![ $($text)* ])
            )
        )
    };
    // Short table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ $wrapper:ident table $mode:ident { $( $key:literal => { $($inner:tt)* } )* } $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::format::HelpSection::$wrapper(
                &$crate::format::HelpSection::Table($crate::format::TableMode::$mode, &[$(
                    ( $key, sections!( $($inner)* ) ),
                )*])
            )
        )
    };
    // Short "NAME" {...}
    (@[ $wrapper:ident $name:literal { $($inner:tt)* } $($rest:tt)* ] $($done:tt)*) => {
        sections_impl!(
            @[$($rest)*] $($done)*,
            $crate::format::HelpSection::$wrapper(
                &$crate::format::HelpSection::Name($name, sections!( $($inner)* ))
            )
        )
    };

    (@[], $($done:tt)*) => {
        &[ $($done)* ]
    };
}

/// Macro to declare a list of text segments. A segment can be written as
///
/// - `"text"` (a string literal)
/// - `{expr}` (an expression that evaluates to a string slice)
///
/// Each segment can be preceded by one of
///
/// - `c:`, where `c` is a [`Color`] variant; the segment is printed in color if
///   supported, otherwise it is wrapped in backticks
/// - `c!`, where `c` is a [`Color`] variant; the segment is printed in color if
///   supported, otherwise no formatting is applied
///
/// Each color can be abbreviated with its first letter (cyan -> c, green -> g,
/// magenta -> m, red -> r, yellow -> y); use an uppercase letter to make it
/// bold (bold cyan -> C, etc.)
///
/// Segments are _not_ separated with commas, for example:
///
/// ```
/// // "warning" is yellow and bold, "world" is cyan, or wrapped in backticks
/// let _segments = text!(Y!"warning" ": hello" c:"world");
///
/// // the value of the FOO environment variable is printed in magenta
/// let _segments = text!("FOO is " m!{env!("FOO")});
/// ```
macro_rules! text {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        text_impl!([ $($rest)* ])
    };
}

/// Macro to declare a list of help sections. This can be passed to [`Help`] to
/// print it:
///
/// ```
/// const HELP: Help = Help(sections!(
///     ["this is my help text."]
/// ));
///
/// fn print_help() -> {
///     HELP.write(
///         &mut std::io::stdout().lock(),
///         false,  // don't show long help
///         supports_color(), // use color if available
///     );
/// }
/// ```
///
/// There are three kinds of sections:
///
/// 1. Normal sections, wrapped in square brackets. Refer to the
///    [`text` macro][text] for the syntax. Example:
///
///    ```
///    ["test" c:"cyan" R!"bold red"]
///    ```
///
///    Each section is terminated by a line break.
///
/// 2. Named sections. Example:
///
///    ```
///    "USAGE" {
///        ["section 1"]
///        ["section 2"]
///    }
///    ```
///
///    Named sections are always preceded by a blank line. Child sections are
///    indented with 4 spaces.
///
/// 3. Tables. Example:
///
///    ```
///    table Auto {
///        "argument 1" => {
///            ["help for argument 1"]
///        }
///        "argument 2" => {
///            ["help for argument 2"]
///            ["and some more help!"]
///        }
///    }
///    ```
///
///    With short help, this is rendered as
///
///    ```text
///    argument 1   help for argument 1
///    argument 2   help for argument 2
///                 and some more help!
///    ```
///
///    With long help, this is rendered as
///
///    ```text
///    argument 1
///            help for argument 1
///
///    argument 2
///            help for argument 2
///            and some more help!
///    ```
///
///    The argument name (left column) must be a string literal. It is displayed
///    in color.
///
///    The `table` keyword must be followed by either `Auto` or `Compact`. If
///    `Compact` is used, then the compact format is used for both the short and
///    long help. If `Auto` is used, the compact format is used for short help
///    and the longer format is used for long help.
///
/// Beyond that, each section can be preceded by `Short` or `Long`. By default,
/// a section is included in the long and short help. With the `Short` modifier,
/// it is _only_ shown in the short help, and sections preceded by `Long` only
/// appear in the long help. Example:
///
/// ```
/// sections!(
///     Short ["Short help text"]
///     Long ["This is more detailed help text"]
///     ["This is shown either way"]
///
///     table Auto {
///         "argument 1" => {
///             ["description"]
///             Long ["Further details only shown in long help"]
///         }
///         "argument 2" => {
///             Long ["This argument isn't shown in the short help"]
///         }
///     }
///
///     // table only shown in long help:
///     Long table Compact {}
///
///     Long "MORE DETAILS" {
///         ["named section only shown in long help"]
///     }
/// );
/// ```
macro_rules! sections {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        sections_impl!(@[ $($rest)* ])
    };
}
