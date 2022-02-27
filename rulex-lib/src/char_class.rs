//! Implements _character classes_. The analogue in the regex world are
//! [character classes](https://www.regular-expressions.info/charclass.html),
//! [shorthand character classes](https://www.regular-expressions.info/shorthand.html),
//! [non-printable characters](https://www.regular-expressions.info/nonprint.html),
//! [Unicode categories/scripts/blocks](https://www.regular-expressions.info/unicode.html#category),
//! [extended grapheme clusters](https://www.regular-expressions.info/unicode.html#grapheme),
//! [POSIX classes](https://www.regular-expressions.info/posixbrackets.html#class) and the
//! [dot](https://www.regular-expressions.info/dot.html).
//!
//! All kinds of character classes mentioned above require `[` square brackets `]` in rulex.
//! A character class can be negated by putting the keyword `not` after the opening bracket. For
//! example, `[not .]` compiles to `\x0A`, the ASCII line break.
//!
//! ## Items
//!
//! A character class can contain multiple _items_, which can be
//!
//! - A code point (called _char_ for simplicity; I know this is incorrect, but this is the
//!   terminology used by most documentation about regexes)
//!
//! - A range of code points, e.g. `U+10-U+200`, which matches any code point `U+10 <= P <= U+200`
//!
//! - A named character class, which can be one of
//!   - a [shorthand character class](https://www.regular-expressions.info/shorthand.html),
//!     e.g. `[w]`, which matches any "word character" (letter, digit or underscore)
//!   - a [non-printable character](https://www.regular-expressions.info/nonprint.html),
//!     e.g. `[e]`, which matches U+001B, the _escape_ control character
//!   - a [POSIX class](https://www.regular-expressions.info/posixbrackets.html#class),
//!     e.g. `[alpha]`, which matches any ASCII alphabetic character.<br>
//!     __However__, POSIX classes are lowered to ranges, e.g. `[alpha]` = `[a-zA-Z]`.
//!   - a [Unicode category, script or block](https://www.regular-expressions.info/unicode.html#category),
//!     e.g. `[Letter]`, which matches any Unicode code point in the _Letter_ category.
//!
//! ### "Special" items
//!
//! There are also three special variants:
//!
//! - `[cp]` or `[codepoint]`, matching any code point
//! - `[.]` (the [dot](https://www.regular-expressions.info/dot.html)), matching any code point
//!   except the ASCII line break (`\n`)
//! - `[X]`, matching an
//!   [extended grapheme clusters](https://www.regular-expressions.info/unicode.html#grapheme)
//!
//! These must be treated specially, so a character class containing any of these can't contain
//! anything else:
//!
//! - Combining `[X]` with anything else would be equivalent to `[X]`
//! - Combining `[cp]` with anything other than `[X]` would be equivalent to `[cp]`
//! - Combining `[.]` with anything other than `[cp]` or `[X]` would be equivalent to `[.]`
//!
//! This demonstrates that there is no need to allow combinations including these special character
//! classes. The other reason is that they require special treatment when negating them (see below).
//!
//! ## Compilation
//!
//! When a character class contains only a single item (e.g. `[w]`), the character class is
//! "flattened":
//!
//! - `['a']` = `a`
//! - `[w]` = `\w`
//! - `[Letter]` = `\p{Letter}`
//! - `[cp]` = `[\S\s]`
//! - `[.]` = `.`
//! - `[X]` = `\X`
//!
//! When there is more than one item or a range (e.g. `['a'-'z' '!']`), a character class is
//! created:
//!
//! - `['a'-'z' '!']` = `[a-z!]`
//! - `[w e Punctuation]` = `[\w\e\p{Punctuation}]`
//!
//! ### Negation
//!
//! Negation is implemented as follows:
//!
//! - Ranges and chars such as `[not 'a'-'z' '!']` are negated with a negative character class,
//!   e.g. `[^a-z!]`.
//!
//! - Shorthand characters such as `[w]` are negated by making them uppercase: `[not w]` = `\W`
//!
//! - Non-printable characters such as `[e]` must be negated by wrapping them in a negated group:
//!   `[not e]` = `[^\e]`
//!
//! - Special classes:
//!   - `[not X]` = `[^\X]`
//!   - `[not .]` = `\x0A` (ASCII line break)
//!   - `[not cp]` = **ERROR**. This would result in an empty group, which is only allowed in
//!     JavaScript; instead we could return `[^\S\s]`, but this doesn't have a use case, since it
//!     matches nothing (it always fails).

use crate::{
    compile::{compile_char, compile_char_escaped, Compile, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
};

use crate::char_group::{CharGroup, GroupItem};

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
        use std::fmt::Write;

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
                    GroupItem::Char(c) => compile_char_escaped(c, buf, options.flavor),
                    GroupItem::Range { first, last } => {
                        buf.push('[');
                        compile_range_char(first, buf, options.flavor);
                        buf.push('-');
                        compile_range_char(last, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Named(name) => {
                        compile_named_class(name, buf, options.flavor, true)?;
                    }
                },
                (1, true) => match items[0] {
                    GroupItem::Char(c) => {
                        buf.push_str("[^");
                        compile_range_char(c, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Range { first, last } => {
                        buf.push_str("[^");
                        compile_range_char(first, buf, options.flavor);
                        buf.push('-');
                        compile_range_char(last, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Named(name) => {
                        compile_named_class_negative(name, buf, options.flavor)?;
                    }
                },
                (_, _) => {
                    buf.push('[');
                    if self.negative {
                        buf.push('^');
                    }
                    for item in items {
                        match *item {
                            GroupItem::Char(c) => compile_range_char(c, buf, options.flavor),
                            GroupItem::Range { first, last } => {
                                compile_range_char(first, buf, options.flavor);
                                buf.push('-');
                                compile_range_char(last, buf, options.flavor);
                            }
                            GroupItem::Named(name) => {
                                compile_named_class(name, buf, options.flavor, false)?;
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

fn compile_named_class(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
    is_single: bool,
) -> Result<(), CompileError> {
    match group {
        "R" => {
            if flavor == RegexFlavor::JavaScript {
                return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
            }
            buf.push_str("\\R");
        }
        "w" | "d" | "s" => {
            buf.push('\\');
            buf.push_str(group);
        }
        "h" | "v" => {
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) {
                buf.push('\\');
                buf.push_str(group);
            } else {
                if is_single {
                    buf.push('[');
                }
                if group == "h" {
                    buf.push_str(r#"\t\p{Zs}"#);
                } else {
                    buf.push_str(r#"\n\x0B\f\r\x85\u2028\u2029"#);
                }
                if is_single {
                    buf.push(']');
                }
            }
        }
        _ if group.starts_with(char::is_lowercase) => {
            return Err(CompileError::Other("Unknown shorthand character class"));
        }
        _ => {
            buf.push_str("\\p{");
            buf.push_str(group);
            buf.push('}');
        }
    }
    Ok(())
}

fn compile_named_class_negative(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
) -> Result<(), CompileError> {
    match group {
        "R" if flavor == RegexFlavor::JavaScript => {
            return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
        }
        "R" => buf.push_str("[^\\R]"),
        "w" | "d" | "s" => {
            buf.push('\\');
            buf.push_str(&group.to_uppercase());
        }
        "h" | "v" => {
            buf.push_str("[^");
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) {
                buf.push('\\');
                buf.push_str(group);
            } else if group == "h" {
                buf.push_str(r#"\t\p{Zs}"#);
            } else {
                buf.push_str(r#"\n\x0B\f\r\x85\u2028\u2029"#);
            }
            buf.push(']');
        }
        _ if group.starts_with(|c: char| c.is_ascii_lowercase()) => {
            return Err(CompileError::Other("Unknown shorthand character class"));
        }
        _ => {
            buf.push_str("\\P{");
            buf.push_str(group);
            buf.push('}');
        }
    }
    Ok(())
}

fn compile_range_char(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '-' => buf.push_str(r#"\-"#),
        ']' => buf.push_str(r#"\]"#),
        '^' => buf.push_str(r#"\^"#),
        c => compile_char(c, buf, flavor),
    }
}
