use std::fmt::Write;

use crate::{
    compile::{compile_char, compile_char_escaped, Compile, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
};

pub(crate) use self::group::{CharGroup, GroupItem};

mod group;

#[derive(Clone, PartialEq, Eq)]
pub struct CharClass<'i> {
    negative: bool,
    inner: CharGroup<'i>,
}

impl<'i> CharClass<'i> {
    pub fn negate(&mut self) {
        self.negative = !self.negative;
    }
}

impl<'i> From<CharGroup<'i>> for CharClass<'i> {
    fn from(inner: CharGroup<'i>) -> Self {
        CharClass {
            inner,
            negative: false,
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for CharClass<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CharClass(")?;

        if self.negative {
            f.write_str("not ")?;
        }

        match &self.inner {
            CharGroup::Dot => f.write_str(".")?,
            CharGroup::CodePoint => f.write_str("codepoint")?,
            CharGroup::X => f.write_str("X")?,
            CharGroup::Items(items) => {
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_char(' ')?;
                    }
                    item.fmt(f)?;
                }
            }
        }
        f.write_char(')')
    }
}

impl Compile for CharClass<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> crate::compile::CompileResult {
        match &self.inner {
            CharGroup::Dot => {
                buf.push_str(if self.negative { "\\n" } else { "." });
            }
            CharGroup::CodePoint => {
                if self.negative {
                    return Err(CompileError::EmptyClassNegated);
                }
                buf.push_str("[\\S\\s]");
            }
            CharGroup::X => {
                if self.negative {
                    return Err(CompileError::EmptyClassNegated);
                }
                if options.flavor == RegexFlavor::JavaScript {
                    return Err(CompileError::Unsupported(Feature::Grapheme, options.flavor));
                }
                buf.push_str("\\X");
            }
            CharGroup::Items(items) => match (items.len(), self.negative) {
                (0, _) => return Err(CompileError::EmptyClass),
                (1, false) => match items[0] {
                    GroupItem::Char(c) => compile_char_escaped(c, buf),
                    GroupItem::Range { first, last } => {
                        buf.push('[');
                        compile_range_char(first, buf);
                        buf.push('-');
                        compile_range_char(last, buf);
                        buf.push(']');
                    }
                    GroupItem::Named(name) => {
                        compile_named_range(name, buf, options.flavor)?;
                    }
                },
                (1, true) => match items[0] {
                    GroupItem::Char(c) => {
                        buf.push_str("[^");
                        compile_range_char(c, buf);
                        buf.push(']');
                    }
                    GroupItem::Range { first, last } => {
                        buf.push_str("[^");
                        compile_range_char(first, buf);
                        buf.push('-');
                        compile_range_char(last, buf);
                        buf.push(']');
                    }
                    GroupItem::Named(name) => {
                        compile_named_range_negative(name, buf, options.flavor)?;
                    }
                },
                (_, _) => {
                    buf.push('[');
                    if self.negative {
                        buf.push('^');
                    }
                    for item in items {
                        match *item {
                            GroupItem::Char(c) => compile_range_char(c, buf),
                            GroupItem::Range { first, last } => {
                                compile_range_char(first, buf);
                                buf.push('-');
                                compile_range_char(last, buf);
                            }
                            GroupItem::Named(name) => {
                                compile_named_range(name, buf, options.flavor)?;
                            }
                        }
                    }
                    buf.push(']');
                }
            },
        }
        Ok(())
    }
}

fn compile_named_range(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
) -> Result<(), CompileError> {
    if group == "R" {
        if flavor == RegexFlavor::JavaScript {
            return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
        }
        buf.push_str("\\R");
    } else if group.starts_with(char::is_lowercase) {
        buf.push('\\');
        buf.push_str(group);
    } else {
        buf.push_str("\\p{");
        buf.push_str(group);
        buf.push('}');
    }
    Ok(())
}

fn compile_named_range_negative(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
) -> Result<(), CompileError> {
    if group == "R" {
        if flavor == RegexFlavor::JavaScript {
            return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
        }
        buf.push_str("[^\\R]");
    } else if group.starts_with(char::is_lowercase) {
        buf.push('\\');
        buf.push_str(&group.to_uppercase());
    } else {
        buf.push_str("\\P{");
        buf.push_str(group);
        buf.push('}');
    }
    Ok(())
}

fn compile_range_char(c: char, buf: &mut String) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '-' => buf.push_str(r#"\-"#),
        ']' => buf.push_str(r#"\]"#),
        '^' => buf.push_str(r#"\^"#),
        c => compile_char(c, buf),
    }
}
