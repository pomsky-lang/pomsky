//! Implements _character classes_. The analogue in the regex world are
//! [character classes](https://www.regular-expressions.info/charclass.html),
//! [shorthand character classes](https://www.regular-expressions.info/shorthand.html),
//! [non-printable characters](https://www.regular-expressions.info/nonprint.html),
//! [Unicode categories/scripts/blocks](https://www.regular-expressions.info/unicode.html#category),
//! [POSIX classes](https://www.regular-expressions.info/posixbrackets.html#class) and the
//! [dot](https://www.regular-expressions.info/dot.html).
//!
//! All kinds of character classes mentioned above require `[` square brackets
//! `]` in Pomsky. A character class can be negated by putting the keyword `not`
//! after the opening bracket. For example, `![.]` compiles to `\n`.
//!
//! ## Items
//!
//! A character class can contain multiple _items_, which can be
//!
//! - A __code point__, e.g. `['a']` or `[U+107]`
//!
//!   - This includes [non-printable characters](https://www.regular-expressions.info/nonprint.html).\
//!     Supported are `[n]`, `[r]`, `[t]`, `[a]`, `[e]` and `[f]`.
//!
//! - A __range of code points__. For example, `[U+10 - U+200]` matches any code
//!   point P where `U+10 ≤ P ≤ U+200`
//!
//! - A __named character class__, which can be one of
//!
//!   - a [shorthand character class](https://www.regular-expressions.info/shorthand.html).\
//!     Supported are `[w]`, `[d]`, `[s]`, `[h]`, `[v]` and `[R]`.
//!
//!   - a [POSIX class](https://www.regular-expressions.info/posixbrackets.html#class).\
//!     Supported are `[ascii_alnum]`, `[ascii_alpha]`, `[ascii]`,
//!     `[ascii_blank]`, `[ascii_cntrl]`, `[ascii_digit]`, `[ascii_graph]`,
//!     `[ascii_lower]`, `[ascii_print]`, `[ascii_punct]`, ´ `[ascii_space]`,
//!     `[ascii_upper]`, `[ascii_word]` and `[ascii_xdigit]`.\ _Note_: POSIX
//!     classes are not Unicode aware!\ _Note_: They're converted to ranges,
//!     e.g. `[ascii_alpha]` = `[a-zA-Z]`.
//!
//!   - a [Unicode category, script or block](https://www.regular-expressions.info/unicode.html#category).\
//!     For example: `[Letter]` compiles to `\p{Letter}`. Pomsky currently
//!     treats any uppercase identifier except `R` as Unicode class.
//!
//! ### "Special" items
//!
//! There are also three special variants:
//!
//! - `[cp]` or `[codepoint]`, matching a code point
//! - `[.]` (the [dot](https://www.regular-expressions.info/dot.html)), matching
//!   any code point except the ASCII line break (`\n`)
//!
//! A character class containing `cp` or `.` can't contain anything else. Note
//! that:
//!
//! - combining `[cp]` with anything else would be equivalent to `[cp]`
//! - combining `[.]` with anything other than `[cp]` or `[n]` would be
//!   equivalent to `[.]`
//!
//! They also require special treatment when negating them (see below).
//!
//! ## Compilation
//!
//! When a character class contains only a single item (e.g. `[w]`), the
//! character class is "flattened":
//!
//! - `['a']` = `a`
//! - `[w]` = `\w`
//! - `[Letter]` = `\p{Letter}`
//! - `[.]` = `.`
//!
//! The exception is `[cp]`, which compiles to `[\S\s]`.
//!
//! When there is more than one item or a range (e.g. `['a'-'z' '!']`), a regex
//! character class is created:
//!
//! - `['a'-'z' '!']` = `[a-z!]`
//! - `[w e Punctuation]` = `[\w\e\p{Punctuation}]`
//!
//! ### Negation
//!
//! Negation is implemented as follows:
//!
//! - Ranges and chars such as `!['a'-'z' '!' e]` are wrapped in a negative
//!   character class, e.g. `[^a-z!\e]`.
//!
//! - The `h`, `v` and `R` shorthands are also wrapped in a negative character
//!   class.
//!
//! - The `w`, `d` and `s` shorthands are negated by making them uppercase
//!   (`![w]` = `\W`), except when there is more than one item in the class
//!   (`![w '-']` = `[^\w\-]`)
//!
//! - Special classes:
//!   - `![.]` = `\n`
//!   - `![cp]` is an error, as this would result in an empty group, which is
//!     only allowed in JavaScript; instead we could return `[^\S\s]`, but this
//!     doesn't have a use case, since it matches nothing (it always fails).
//!
//! - `w`, `s`, `d` and Unicode categories/scripts/blocks can be negated
//!   individually _within a character class_, e.g. `[s !s]` = `[\s\S]`
//!   (equivalent to `[cp]`), `![!Latin 'a']` = `[^\P{Latin}a]`.
//!
//!   When a negated character class only contains 1 item, which is also
//! negated, the class is   removed and the negations cancel each other out:
//! `![!w]` = `\w`, `![!L]` = `\p{L}`.

use std::borrow::Cow;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind, Feature},
    exprs::literal,
    options::{CompileOptions, RegexFlavor},
    regex::{Regex, RegexProperty, RegexShorthand},
};

use pomsky_syntax::{
    exprs::{Category, CharClass, CharGroup, GroupItem, GroupName, OtherProperties},
    Span,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for CharClass {
    fn compile(&self, options: CompileOptions, _: &mut CompileState<'_, 'i>) -> CompileResult<'i> {
        let span = self.span;
        match &self.inner {
            CharGroup::Dot => {
                Ok(if self.negative { Regex::Literal(Cow::Borrowed("\\n")) } else { Regex::Dot })
            }
            CharGroup::CodePoint => {
                if self.negative {
                    return Err(CompileErrorKind::EmptyClassNegated.at(span));
                }
                Ok(Regex::CharSet(RegexCharSet {
                    negative: false,
                    items: vec![
                        RegexCharSetItem::Shorthand(RegexShorthand::Space),
                        RegexCharSetItem::Shorthand(RegexShorthand::NotSpace),
                    ],
                }))
            }
            CharGroup::Items(items) => match (items.len(), self.negative) {
                (0, _) => Err(CompileErrorKind::EmptyClass.at(span)),
                (1, false) => match items[0] {
                    GroupItem::Char(c) => Ok(Regex::Char(c)),
                    GroupItem::Range { first, last } => Ok(Regex::CharSet(RegexCharSet {
                        negative: false,
                        items: vec![RegexCharSetItem::Range { first, last }],
                    })),
                    GroupItem::Named { name, negative } => {
                        named_class_to_regex(name, negative, options.flavor, span)
                    }
                },
                (1, true) => match items[0] {
                    GroupItem::Char(c) => Ok(Regex::CharSet(RegexCharSet {
                        negative: true,
                        items: vec![RegexCharSetItem::Char(c)],
                    })),
                    GroupItem::Range { first, last } => Ok(Regex::CharSet(RegexCharSet {
                        negative: true,
                        items: vec![RegexCharSetItem::Range { first, last }],
                    })),
                    GroupItem::Named { name, negative } => {
                        named_class_to_regex(name, !negative, options.flavor, span)
                    }
                },
                (_, negative) => {
                    let mut buf = Vec::new();
                    for item in items {
                        match *item {
                            GroupItem::Char(c) => buf.push(RegexCharSetItem::Char(c)),
                            GroupItem::Range { first, last } => {
                                buf.push(RegexCharSetItem::Range { first, last });
                            }
                            GroupItem::Named { name, negative } => {
                                named_class_to_regex_class_items(
                                    name,
                                    negative,
                                    options.flavor,
                                    span,
                                    &mut buf,
                                )?;
                            }
                        }
                    }

                    Ok(Regex::CharSet(RegexCharSet { negative, items: buf }))
                }
            },
        }
    }
}

/// Compiles a shorthand character class or Unicode category/script/block.
///
/// Refer to the [module-level documentation](self) for details about named
/// character classes.
fn named_class_to_regex(
    group: GroupName,
    negative: bool,
    flavor: RegexFlavor,
    span: Span,
) -> CompileResult<'static> {
    Ok(match group {
        GroupName::Word => {
            if flavor == RegexFlavor::JavaScript {
                Regex::CharSet(RegexCharSet {
                    negative,
                    items: vec![
                        RegexProperty::Other(OtherProperties::Alphabetic).negative_item(false),
                        RegexProperty::Category(Category::Mark).negative_item(false),
                        RegexProperty::Category(Category::Decimal_Number).negative_item(false),
                        RegexProperty::Category(Category::Connector_Punctuation)
                            .negative_item(false),
                    ],
                })
            } else {
                Regex::Shorthand(if negative {
                    RegexShorthand::NotWord
                } else {
                    RegexShorthand::Word
                })
            }
        }
        GroupName::Digit => {
            if flavor == RegexFlavor::JavaScript {
                RegexProperty::Category(Category::Decimal_Number).negative(negative)
            } else {
                Regex::Shorthand(if negative {
                    RegexShorthand::NotDigit
                } else {
                    RegexShorthand::Digit
                })
            }
        }
        GroupName::Space if negative => Regex::Shorthand(RegexShorthand::NotSpace),
        GroupName::Space => Regex::Shorthand(RegexShorthand::Space),

        GroupName::HorizSpace | GroupName::VertSpace
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) =>
        {
            let shorthand = if group == GroupName::HorizSpace {
                RegexShorthand::HorizSpace
            } else {
                RegexShorthand::VertSpace
            };

            if negative {
                Regex::CharSet(RegexCharSet {
                    negative: true,
                    items: vec![RegexCharSetItem::Shorthand(shorthand)],
                })
            } else {
                Regex::Shorthand(shorthand)
            }
        }
        GroupName::HorizSpace => Regex::CharSet(RegexCharSet {
            negative,
            items: vec![
                RegexCharSetItem::Char('\t'),
                RegexProperty::Category(Category::Space_Separator).negative_item(false),
            ],
        }),
        GroupName::VertSpace => Regex::CharSet(RegexCharSet {
            negative,
            items: vec![
                RegexCharSetItem::Range { first: '\x0A', last: '\x0D' },
                RegexCharSetItem::Char('\u{85}'),
                RegexCharSetItem::Char('\u{2028}'),
                RegexCharSetItem::Char('\u{2029}'),
            ],
        }),

        GroupName::Category(c) => RegexProperty::Category(c).negative(negative),
        GroupName::Script(s) => RegexProperty::Script(s).negative(negative),
        GroupName::CodeBlock(b) => match flavor {
            RegexFlavor::DotNet | RegexFlavor::Java | RegexFlavor::Ruby => {
                RegexProperty::Block(b).negative(negative)
            }
            _ => return Err(CompileErrorKind::Unsupported(Feature::UnicodeBlock, flavor).at(span)),
        },
        GroupName::OtherProperties(o) => {
            // TODO: Find out which regex engines (other than PCRE) don't support these
            if flavor == RegexFlavor::Pcre {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            }
            RegexProperty::Other(o).negative(negative)
        }
    })
}

fn named_class_to_regex_class_items(
    group: GroupName,
    negative: bool,
    flavor: RegexFlavor,
    span: Span,
    buf: &mut Vec<RegexCharSetItem>,
) -> Result<(), CompileError> {
    match group {
        GroupName::Word => {
            if let RegexFlavor::JavaScript = flavor {
                if negative {
                    return Err(
                        CompileErrorKind::Unsupported(Feature::NegativeShorthandW, flavor).at(span)
                    );
                }
                buf.push(RegexProperty::Other(OtherProperties::Alphabetic).negative_item(false));
                buf.push(RegexProperty::Category(Category::Mark).negative_item(false));
                buf.push(RegexProperty::Category(Category::Decimal_Number).negative_item(false));
                buf.push(
                    RegexProperty::Category(Category::Connector_Punctuation).negative_item(false),
                );
            } else {
                buf.push(RegexCharSetItem::Shorthand(if negative {
                    RegexShorthand::NotWord
                } else {
                    RegexShorthand::Word
                }));
            }
        }
        GroupName::Digit => {
            if flavor == RegexFlavor::JavaScript {
                buf.push(RegexProperty::Category(Category::Decimal_Number).negative_item(negative));
            } else if negative {
                buf.push(RegexCharSetItem::Shorthand(RegexShorthand::NotDigit));
            } else {
                buf.push(RegexCharSetItem::Shorthand(RegexShorthand::Digit));
            }
        }
        GroupName::Space => buf.push(RegexCharSetItem::Shorthand(if negative {
            RegexShorthand::NotSpace
        } else {
            RegexShorthand::Space
        })),

        GroupName::HorizSpace | GroupName::VertSpace if negative => {
            return Err(CompileErrorKind::Other(
                "horiz_space and vert_space can't be negated within a character class",
            )
            .at(span));
        }

        GroupName::HorizSpace | GroupName::VertSpace
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) =>
        {
            buf.push(RegexCharSetItem::Shorthand(if group == GroupName::HorizSpace {
                RegexShorthand::HorizSpace
            } else {
                RegexShorthand::VertSpace
            }));
        }
        GroupName::HorizSpace => {
            buf.push(RegexCharSetItem::Char('\t'));
            buf.push(RegexProperty::Category(Category::Space_Separator).negative_item(false));
        }
        GroupName::VertSpace => {
            buf.push(RegexCharSetItem::Range { first: '\x0A', last: '\x0D' });
            buf.push(RegexCharSetItem::Char('\u{85}'));
            buf.push(RegexCharSetItem::Char('\u{2028}'));
            buf.push(RegexCharSetItem::Char('\u{2029}'));
        }

        GroupName::Category(c) => buf.push(RegexProperty::Category(c).negative_item(negative)),
        GroupName::Script(s) => buf.push(RegexProperty::Script(s).negative_item(negative)),
        GroupName::CodeBlock(b) => match flavor {
            RegexFlavor::DotNet | RegexFlavor::Java | RegexFlavor::Ruby => {
                buf.push(RegexProperty::Block(b).negative_item(negative));
            }
            _ => return Err(CompileErrorKind::Unsupported(Feature::UnicodeBlock, flavor).at(span)),
        },
        GroupName::OtherProperties(o) => {
            // TODO: Find out which regex engines (other than PCRE) don't support these
            if flavor == RegexFlavor::Pcre {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            }
            buf.push(RegexProperty::Other(o).negative_item(negative));
        }
    }
    Ok(())
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexCharSet {
    negative: bool,
    items: Vec<RegexCharSetItem>,
}

impl RegexCharSet {
    pub(crate) fn new(items: Vec<RegexCharSetItem>) -> Self {
        Self { negative: false, items }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        if self.negative {
            buf.push_str("[^");
        } else {
            buf.push('[');
        }

        for item in &self.items {
            match *item {
                RegexCharSetItem::Char(c) => {
                    literal::compile_char_esc_in_class(c, buf, flavor);
                }
                RegexCharSetItem::Range { first, last } => {
                    literal::compile_char_esc_in_class(first, buf, flavor);
                    buf.push('-');
                    literal::compile_char_esc_in_class(last, buf, flavor);
                }
                RegexCharSetItem::Shorthand(s) => s.codegen(buf),
                RegexCharSetItem::Property { negative, value } => {
                    value.codegen(buf, negative, flavor);
                }
            }
        }

        buf.push(']');
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexCharSetItem {
    Char(char),
    Range { first: char, last: char },
    Shorthand(RegexShorthand),
    Property { negative: bool, value: RegexProperty },
}

impl RegexCharSetItem {
    pub(crate) fn range_unchecked(first: char, last: char) -> Self {
        Self::Range { first, last }
    }
}
