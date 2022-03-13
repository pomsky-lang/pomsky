//! Implements _character classes_. The analogue in the regex world are
//! [character classes](https://www.regular-expressions.info/charclass.html),
//! [shorthand character classes](https://www.regular-expressions.info/shorthand.html),
//! [non-printable characters](https://www.regular-expressions.info/nonprint.html),
//! [Unicode categories/scripts/blocks](https://www.regular-expressions.info/unicode.html#category),
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
//!     identifier except `R` as Unicode class.
//!
//! ### "Special" items
//!
//! There are also three special variants:
//!
//! - `[cp]` or `[codepoint]`, matching a code point
//! - `[.]` (the [dot](https://www.regular-expressions.info/dot.html)), matching any code point
//!   except the ASCII line break (`\n`)
//!
//! A character class containing `cp` or `.` can't contain anything else. Note that:
//!
//! - combining `[cp]` with anything else would be equivalent to `[cp]`
//! - combining `[.]` with anything other than `[cp]` or `[n]` would be equivalent to `[.]`
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
//!   - `![cp]` is an error, as this would result in an empty group, which is only allowed in
//!     JavaScript; instead we could return `[^\S\s]`, but this doesn't have a use case, since it
//!     matches nothing (it always fails).
//!
//! - `w`, `s`, `d` and Unicode categories/scripts/blocks can be negated individually _within a
//!   character class_, e.g. `[s !s]` = `[\s\S]` (equivalent to `[cp]`),
//!   `![!Latin 'a']` = `[^\P{Latin}a]`.
//!
//!   When a negated character class only contains 1 item, which is also negated, the class is
//!   removed and the negations cancel each other out: `![!w]` = `\w`, `![!L]` = `\p{L}`.

use crate::{
    compile::{Compile, CompileState},
    error::{CompileError, CompileErrorKind, Feature},
    literal::{compile_char, compile_char_esc},
    options::{CompileOptions, RegexFlavor},
    span::Span,
};

pub(crate) use char_group::{CharGroup, GroupItem};

use self::char_group::GroupName;

mod ascii;
mod char_group;
mod unicode;

/// A _character class_. Refer to the [module-level documentation](self) for details.
#[derive(Clone, PartialEq, Eq)]
pub struct CharClass {
    negative: bool,
    inner: CharGroup,
    pub(crate) span: Span,
}

impl CharClass {
    pub(crate) fn new(inner: CharGroup, span: Span) -> Self {
        CharClass { inner, span, negative: false }
    }

    /// Makes a positive character class negative and vice versa.
    pub fn negate(&mut self) {
        self.negative = !self.negative;
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for CharClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use std::fmt::Write;

        f.write_str("CharClass(")?;

        if self.negative {
            f.write_str("not ")?;
        }

        match &self.inner {
            CharGroup::Dot => f.write_str(".")?,
            CharGroup::CodePoint => f.write_str("codepoint")?,
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

impl Compile for CharClass {
    fn comp(
        &self,
        options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> crate::compile::CompileResult {
        let span = self.span;
        match &self.inner {
            CharGroup::Dot => {
                buf.push_str(if self.negative { "\\n" } else { "." });
            }
            CharGroup::CodePoint => {
                if self.negative {
                    return Err(CompileErrorKind::EmptyClassNegated.at(span));
                }
                buf.push_str("[\\S\\s]");
            }
            CharGroup::Items(items) => match (items.len(), self.negative) {
                (0, _) => return Err(CompileErrorKind::EmptyClass.at(span)),
                (1, false) => match items[0] {
                    GroupItem::Char(c) => compile_char_esc(c, buf, options.flavor),
                    GroupItem::Range { first, last } => {
                        buf.push('[');
                        compile_char_esc_in_class(first, buf, options.flavor);
                        buf.push('-');
                        compile_char_esc_in_class(last, buf, options.flavor);
                        buf.push(']');
                    }
                    GroupItem::Named { name, negative } => {
                        if negative {
                            compile_named_class_negative(name, buf, options.flavor, span)?;
                        } else {
                            compile_named_class(name, negative, buf, options.flavor, true, span)?;
                        }
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
                    GroupItem::Named { name, negative } => {
                        if negative {
                            compile_named_class(name, false, buf, options.flavor, true, span)?;
                        } else {
                            compile_named_class_negative(name, buf, options.flavor, span)?;
                        }
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
                            GroupItem::Named { name, negative } => {
                                compile_named_class(
                                    name,
                                    negative,
                                    buf,
                                    options.flavor,
                                    false,
                                    span,
                                )?;
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
    group: GroupName,
    negative: bool,
    buf: &mut String,
    flavor: RegexFlavor,
    is_single: bool,
    span: Span,
) -> Result<(), CompileError> {
    match group {
        GroupName::Word if negative => buf.push_str("\\W"),
        GroupName::Word => buf.push_str("\\w"),

        GroupName::HorizSpace | GroupName::VertSpace | GroupName::LineBreak if negative => {
            let s = group.as_str().to_string();
            return Err(CompileErrorKind::UnsupportedNegatedClass(s).at(span));
        }

        GroupName::HorizSpace | GroupName::VertSpace => {
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) {
                buf.push_str(if group == GroupName::HorizSpace { "\\h" } else { "\\v" });
            } else {
                if is_single {
                    buf.push('[');
                }
                if group == GroupName::HorizSpace {
                    buf.push_str(r#"\t\p{Zs}"#);
                } else {
                    buf.push_str(r#"\n\x0B\f\r\x85\u2028\u2029"#);
                }
                if is_single {
                    buf.push(']');
                }
            }
        }
        GroupName::LineBreak if flavor == RegexFlavor::JavaScript => {
            return Err(CompileErrorKind::Unsupported(Feature::UnicodeLineBreak, flavor).at(span));
        }
        GroupName::LineBreak => buf.push_str("\\R"),
        _ => {
            if negative {
                buf.push_str("\\P{");
            } else {
                buf.push_str("\\p{");
            }
            match group {
                GroupName::Category(c) => buf.push_str(c.as_str()),
                GroupName::Script(s) => buf.push_str(s.as_str()),
                GroupName::CodeBlock(c) => match flavor {
                    RegexFlavor::DotNet => {
                        buf.push_str("Is");
                        buf.push_str(&c.as_str().replace('_', ""));
                    }
                    RegexFlavor::Java => {
                        buf.push_str("In");
                        buf.push_str(&c.as_str().replace('-', ""));
                    }
                    RegexFlavor::Ruby => {
                        buf.push_str("In");
                        buf.push_str(c.as_str());
                    }
                    _ => {
                        return Err(
                            CompileErrorKind::Unsupported(Feature::UnicodeBlock, flavor).at(span)
                        )
                    }
                },
                GroupName::OtherProperties(o) => {
                    if flavor == RegexFlavor::Pcre {
                        return Err(
                            CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span)
                        );
                    }
                    buf.push_str(o.as_str());
                }
                _ => unreachable!("The group is neither a Category, nor a Script, nor a CodeBlock"),
            }
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
    group: GroupName,
    buf: &mut String,
    flavor: RegexFlavor,
    span: Span,
) -> Result<(), CompileError> {
    match group {
        GroupName::Word => buf.push_str("\\W"),
        GroupName::HorizSpace | GroupName::VertSpace => {
            buf.push_str("[^");
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) {
                buf.push_str(if group == GroupName::HorizSpace { "\\h" } else { "\\v" });
            } else if group == GroupName::HorizSpace {
                buf.push_str(r#"\t\p{Zs}"#);
            } else {
                buf.push_str(r#"\n\x0B\f\r\x85\u2028\u2029"#);
            }
            buf.push(']');
        }
        GroupName::LineBreak if flavor == RegexFlavor::JavaScript => {
            return Err(CompileErrorKind::Unsupported(Feature::UnicodeLineBreak, flavor).at(span));
        }
        GroupName::LineBreak => buf.push_str("[^\\R]"),

        GroupName::Category(c) => {
            buf.push_str("\\P{");
            buf.push_str(c.as_str());
            buf.push('}');
        }
        GroupName::Script(s) => {
            buf.push_str("\\P{");
            buf.push_str(s.as_str());
            buf.push('}');
        }
        GroupName::CodeBlock(c) => {
            buf.push_str("\\P{");
            match flavor {
                RegexFlavor::DotNet => {
                    buf.push_str("Is");
                    buf.push_str(&c.as_str().replace('_', ""));
                }
                RegexFlavor::Java => {
                    buf.push_str("In");
                    buf.push_str(&c.as_str().replace('-', ""));
                }
                RegexFlavor::Ruby => {
                    buf.push_str("In");
                    buf.push_str(c.as_str());
                }
                _ => {
                    return Err(
                        CompileErrorKind::Unsupported(Feature::UnicodeBlock, flavor).at(span)
                    )
                }
            }
            buf.push('}');
        }
        GroupName::OtherProperties(o) => {
            if flavor == RegexFlavor::Pcre {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            }
            buf.push_str("\\P{");
            buf.push_str(o.as_str());
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
