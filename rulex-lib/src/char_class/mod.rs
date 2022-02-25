use crate::{
    compile::{compile_char, compile_char_escaped, Compile, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
};

use inner::CharClassInner;

mod char_range;
mod inner;

#[derive(Clone, PartialEq, Eq)]
pub enum CharClass<'i> {
    CodePoint,
    Grapheme,
    Dot,
    Empty,
    Inner(CharClassInner<'i>),
    Negated(CharClassInner<'i>),
}

impl<'i> CharClass<'i> {
    pub fn negate(self) -> Self {
        match self {
            CharClass::CodePoint | CharClass::Grapheme => CharClass::Empty,
            CharClass::Dot => CharClass::from_char('\n'),
            CharClass::Empty => CharClass::CodePoint,
            CharClass::Inner(i) => CharClass::Negated(i),
            CharClass::Negated(n) => CharClass::Inner(n),
        }
    }

    pub fn is_negated(&self) -> bool {
        matches!(self, CharClass::Negated(_))
    }

    pub fn union(self, rhs: Self) -> Result<Self, (Self, Self)> {
        Ok(match (self, rhs) {
            (CharClass::Empty, c) | (c, CharClass::Empty) => c,
            (CharClass::CodePoint, _) | (_, CharClass::CodePoint) => CharClass::CodePoint,
            (CharClass::Grapheme, _) | (_, CharClass::Grapheme) => CharClass::Grapheme,
            (CharClass::Dot, c) | (c, CharClass::Dot) => {
                if c.includes_newline() {
                    CharClass::CodePoint
                } else {
                    CharClass::Dot
                }
            }
            (CharClass::Inner(i1), CharClass::Inner(ref mut i2)) => CharClass::Inner(i1.union(i2)),
            tuple @ ((CharClass::Negated(_), CharClass::Negated(_))
            | (CharClass::Inner(_), CharClass::Negated(_))
            | (CharClass::Negated(_), CharClass::Inner(_))) => return Err(tuple),
        })
    }

    pub fn try_from_range(first: char, last: char) -> Option<Self> {
        let mut new = CharClassInner::default();
        if first <= last {
            new.add_range(first, last);
            Some(CharClass::Inner(new))
        } else {
            None
        }
    }

    pub fn from_chars(chars: &str) -> Self {
        let mut inner = CharClassInner::default();
        inner.add_ranges(chars);
        CharClass::Inner(inner)
    }

    pub fn from_char(c: char) -> Self {
        let mut inner = CharClassInner::default();
        inner.add_range(c, c);
        CharClass::Inner(inner)
    }

    pub fn from_group_name(name: &'i str) -> Self {
        match name {
            "codepoint" | "cp" => CharClass::CodePoint,
            "X" => CharClass::Grapheme,
            "." => CharClass::Dot,
            _ => {
                let mut inner = CharClassInner::default();
                inner.add_named(name);
                CharClass::Inner(inner)
            }
        }
    }

    fn includes_newline(&self) -> bool {
        match self {
            CharClass::CodePoint | CharClass::Grapheme => true,
            CharClass::Dot | CharClass::Empty => false,
            CharClass::Inner(n) => n.includes_newline(),
            CharClass::Negated(n) => !n.includes_newline(),
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for CharClass<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CharClass::CodePoint => f.write_str("CharClass(<codepoint>)"),
            CharClass::Grapheme => f.write_str("CharClass(<X>)"),
            CharClass::Dot => f.write_str("CharClass(<.>)"),
            CharClass::Empty => f.write_str("CharClass(empty)"),
            CharClass::Inner(i) | CharClass::Negated(i) => {
                let mut items = Vec::with_capacity(i.groups.len() + i.ranges.len());
                for &part in &i.groups {
                    items.push(format!("<{part}>"));
                }
                for range in &i.ranges {
                    items.push(format!("{range:?}"));
                }
                write!(f, "CharClass({})", items.join(" | "))?;
                if let CharClass::Negated(_) = self {
                    f.write_str(" negated")?;
                }
                Ok(())
            }
        }
    }
}

impl Compile for CharClass<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> crate::compile::CompileResult {
        match self {
            CharClass::Empty => {
                buf.push_str("[^\\S\\s]");
            }
            CharClass::CodePoint => {
                buf.push_str("[\\S\\s]");
            }
            CharClass::Grapheme => {
                if options.flavor == RegexFlavor::JavaScript {
                    return Err(CompileError::Unsupported(Feature::Grapheme, options.flavor));
                }
                buf.push_str("\\X");
            }
            CharClass::Dot => {
                buf.push('.');
            }
            CharClass::Inner(i) => {
                if i.groups.len() == 1 && i.ranges.is_empty() {
                    compile_named_range(i.groups[0], false, true, buf, options.flavor)?;
                } else if let Some(c) = i.get_single_char() {
                    compile_char_escaped(c, buf);
                } else {
                    buf.push('[');
                    for &range in &i.groups {
                        compile_named_range(range, false, false, buf, options.flavor)?;
                    }
                    for &range in &i.ranges {
                        let range = range.0;
                        compile_range_char(range.first, buf);
                        if range.last != range.first {
                            buf.push('-');
                            compile_range_char(range.last, buf);
                        }
                    }
                    buf.push(']');
                }
            }
            CharClass::Negated(n) => {
                if n.groups.len() == 1 && n.ranges.is_empty() {
                    compile_named_range(n.groups[0], true, true, buf, options.flavor)?;
                } else if let Some(c) = n.get_single_char() {
                    buf.push_str("[^");
                    compile_range_char(c, buf);
                    buf.push(']');
                } else {
                    buf.push_str("[^");
                    for &range in &n.groups {
                        compile_named_range(range, false, false, buf, options.flavor)?;
                    }
                    for &range in &n.ranges {
                        let range = range.0;
                        compile_range_char(range.first, buf);
                        if range.last != range.first {
                            buf.push('-');
                            compile_range_char(range.last, buf);
                        }
                    }
                    buf.push(']');
                }
            }
        }
        Ok(())
    }
}

fn compile_named_range(
    group: &str,
    negated: bool,
    single: bool,
    buf: &mut String,
    flavor: RegexFlavor,
) -> Result<(), CompileError> {
    if matches!(group, "R" | "X") {
        if flavor == RegexFlavor::JavaScript {
            return Err(CompileError::Unsupported(
                if group == "R" {
                    Feature::UnicodeLineBreak
                } else {
                    Feature::Grapheme
                },
                flavor,
            ));
        }
        if negated {
            if !single {
                return Err(CompileError::Other("Cannot negate <R>"));
            }
            buf.push_str("[^\\");
            buf.push_str(group);
            buf.push(']');
        } else {
            buf.push('\\');
            buf.push_str(group);
        }
    } else if group.starts_with(char::is_lowercase) {
        buf.push('\\');
        if negated {
            buf.push_str(&group.to_ascii_uppercase());
        } else {
            buf.push_str(group);
        }
    } else {
        if negated {
            buf.push_str("\\P{");
        } else {
            buf.push_str("\\p{");
        }
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
