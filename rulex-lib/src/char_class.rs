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
//! example, `![.]` compiles to `\n`.
//!
//! ## Items
//!
//! A character class can contain multiple _items_, which can be
//!
//! - A __code point__, e.g. `['a']` or `[U+107]`
//!
//!   - This includes
//!     [non-printable characters](https://www.regular-expressions.info/nonprint.html).\
//!     Supported are `[n]`, `[r]`, `[t]`, `[a]`, `[e]` and `[f]`.
//!
//! - A __range of code points__. For example, `[U+10 - U+200]` matches any code point P where
//!   `U+10 ≤ P ≤ U+200`
//!
//! - A __named character class__, which can be one of
//!
//!   - a [shorthand character class](https://www.regular-expressions.info/shorthand.html).\
//!     Supported are `[w]`, `[d]`, `[s]`, `[h]`, `[v]` and `[R]`.
//!
//!   - a [POSIX class](https://www.regular-expressions.info/posixbrackets.html#class).\
//!     Supported are `[alnum]`, `[alpha]`, `[ascii]`, `[blank]`, `[cntrl]`, `[digit]`, `[graph]`,
//!     `[lower]`, `[print]`, `[punct]`, `[space]`, `[upper]`, `[word]` and `[xdigit]`.\
//!     _Note_: POSIX classes are not Unicode aware!\
//!     _Note_: They're converted to ranges, e.g. `[alpha]` = `[a-zA-Z]`.
//!
//!   - a [Unicode category, script or block](https://www.regular-expressions.info/unicode.html#category).\
//!     For example: `[Letter]` compiles to `\p{Letter}`. Rulex currently treats any uppercase
//!     identifier except `X` and `R` as Unicode class.
//!
//! ### "Special" items
//!
//! There are also three special variants:
//!
//! - `[cp]` or `[codepoint]`, matching a code point
//! - `[.]` (the [dot](https://www.regular-expressions.info/dot.html)), matching any code point
//!   except the ASCII line break (`\n`)
//! - `[X]`, matching an
//!   [extended grapheme cluster](https://www.regular-expressions.info/unicode.html#grapheme)
//!
//! A character class containing any of these can't contain anything else. Note that:
//!
//! - combining `[X]` with anything else would be equivalent to `[X]`
//! - combining `[cp]` with anything other than `[X]` would be equivalent to `[cp]`
//! - combining `[.]` with anything other than `[cp]` or `[X]` would be equivalent to `[.]`
//!
//! They also require special treatment when negating them (see below).
//!
//! ## Compilation
//!
//! When a character class contains only a single item (e.g. `[w]`), the character class is
//! "flattened":
//!
//! - `['a']` = `a`
//! - `[w]` = `\w`
//! - `[Letter]` = `\p{Letter}`
//! - `[.]` = `.`
//! - `[X]` = `\X`
//!
//! The exception is `[cp]`, which compiles to `[\S\s]`.
//!
//! When there is more than one item or a range (e.g. `['a'-'z' '!']`), a regex character class is
//! created:
//!
//! - `['a'-'z' '!']` = `[a-z!]`
//! - `[w e Punctuation]` = `[\w\e\p{Punctuation}]`
//!
//! ### Negation
//!
//! Negation is implemented as follows:
//!
//! - Ranges and chars such as `!['a'-'z' '!' e]` are wrapped in a negative character class,
//!   e.g. `[^a-z!\e]`.
//!
//! - The `h`, `v` and `R` shorthands are also wrapped in a negative character class.
//!
//! - The `w`, `d` and `s` shorthands are negated by making them uppercase (`![w]` = `\W`),
//!   except when there is more than one item in the class (`![w '-']` = `[^\w\-]`)
//!
//! - Special classes:
//!   - `![.]` = `\n`
//!   - `![X]` or `![cp]` is an error, as this would result in an empty group, which is only
//!     allowed in JavaScript; instead we could return `[^\S\s]`, but this doesn't have a use case,
//!     since it matches nothing (it always fails).
//!
//! I'm considering a possibility to negate only part of a character class, e.g. `[s !s]`
//! (which is equivalent to `[cp]`) or `![!Latin 'a']` = `[^\P{Latin}a]`.
//!
//! I'm also considering making `X` an expression that isn't wrapped in brackets. After all, it is
//! the only character class that can match more than 1 code point.

use crate::{
    compile::{compile_char, compile_char_esc, Compile, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
};

use crate::char_group::{CharGroup, GroupItem};

/// A _character class_. Refer to the [module-level documentation](self) for details.
#[derive(Clone, PartialEq, Eq)]
pub struct CharClass<'i> {
    negative: bool,
    inner: CharGroup<'i>,
}

impl<'i> CharClass<'i> {
    /// Makes a positive character class negative and vice versa.
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
                    GroupItem::Char(c) => compile_char_esc(c, buf, options.flavor),
                    GroupItem::Range { first, last } => {
                        buf.push('[');
                        compile_char_esc_in_class(first, buf, options.flavor);
                        buf.push('-');
                        compile_char_esc_in_class(last, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Named(name) => {
                        compile_named_class(name, buf, options.flavor, true)?;
                    }
                },
                (1, true) => match items[0] {
                    GroupItem::Char(c) => {
                        buf.push_str("[^");
                        compile_char_esc_in_class(c, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Range { first, last } => {
                        buf.push_str("[^");
                        compile_char_esc_in_class(first, buf, options.flavor);
                        buf.push('-');
                        compile_char_esc_in_class(last, buf, options.flavor);
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
                            GroupItem::Char(c) => compile_char_esc_in_class(c, buf, options.flavor),
                            GroupItem::Range { first, last } => {
                                compile_char_esc_in_class(first, buf, options.flavor);
                                buf.push('-');
                                compile_char_esc_in_class(last, buf, options.flavor);
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

/// Compiles a shorthand character class or Unicode category/script/block.
///
/// Refer to the [module-level documentation](self) for details about named character classes.
///
/// The last argument is important. Set `is_single` to `true` if no square brackets are printed
/// outside of this function call:
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
/// compile_named_class("h", buf, flavor, true);
///
/// // or:
///
/// buf.push('[');
/// compile_named_class("h", buf, flavor, false);
/// buf.push(']');
/// ````
fn compile_named_class(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
    is_single: bool,
) -> Result<(), CompileError> {
    match group {
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
        "R" if flavor == RegexFlavor::JavaScript => {
            return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
        }
        "R" => buf.push_str("\\R"),
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

/// Compiles a negated shorthand character class or Unicode category/script/block.
///
/// Refer to the [module-level documentation](self) for details about named character classes
/// and negation.
fn compile_named_class_negative(
    group: &str,
    buf: &mut String,
    flavor: RegexFlavor,
) -> Result<(), CompileError> {
    match group {
        "w" => buf.push_str("\\W"),
        "d" => buf.push_str("\\D"),
        "s" => buf.push_str("\\S"),
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
        "R" if flavor == RegexFlavor::JavaScript => {
            return Err(CompileError::Unsupported(Feature::UnicodeLineBreak, flavor));
        }
        "R" => buf.push_str("[^\\R]"),
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

/// Write a char to the output buffer with proper escaping. Assumes the char is inside a
/// character class.
fn compile_char_esc_in_class(c: char, buf: &mut String, flavor: RegexFlavor) {
    match c {
        '\\' => buf.push_str(r#"\\"#),
        '-' => buf.push_str(r#"\-"#),
        ']' => buf.push_str(r#"\]"#),
        '^' => buf.push_str(r#"\^"#),
        c => compile_char(c, buf, flavor),
    }
}
