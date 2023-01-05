use std::io::{self, Write};

use crate::Color;

/// A structured help message.
///
/// Refer to the [crate-level documentation](index.html) for help.
#[derive(Debug, Clone)]
pub struct Help<'a>(pub &'a [HelpSection<'a>]);

impl Help<'_> {
    pub fn write(&self, buf: &mut impl Write, long: bool, colored: bool) -> io::Result<()> {
        for section in self.0 {
            section.write(buf, long, colored, 0, false)?;
        }
        Ok(())
    }
}

/// Part of a help message. Should be created with the
/// [`sections`](crate::sections) macro.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum HelpSection<'a> {
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
                    buf.write_all(Color::ANSI_Y.as_bytes())?;
                    buf.write_all(name.as_bytes())?;
                    buf.write_all(Color::ANSI_RESET.as_bytes())?;
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
                        buf.write_all(Color::ANSI_G.as_bytes())?;
                        buf.write_all(key.as_bytes())?;
                        buf.write_all(Color::ANSI_RESET.as_bytes())?;
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

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum TableMode {
    Compact,
    Auto,
}

#[derive(Debug, Clone)]
pub struct Segment<'a> {
    pub style: Option<Color>,
    pub text: &'a str,
    pub ticks: bool,
}

impl<'a> Segment<'a> {
    pub const fn new(text: &'a str) -> Self {
        Segment { style: None, text, ticks: false }
    }

    pub fn write(&self, buf: &mut impl Write, colored: bool, indent: usize) -> io::Result<()> {
        if let Some(color) = &self.style {
            if colored {
                buf.write_all(color.ansi_code().as_bytes())?;
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
                buf.write_all(Color::ANSI_RESET.as_bytes())?;
            } else if self.ticks {
                buf.write_all(b"`")?;
            }
        }

        Ok(())
    }
}
