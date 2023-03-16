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
//! - A __code point__, e.g. `['a']` or `[U_107]`
//!
//!   - This includes [non-printable characters](https://www.regular-expressions.info/nonprint.html).\
//!     Supported are `[n]`, `[r]`, `[t]`, `[a]`, `[e]` and `[f]`.
//!
//! - A __range of code points__. For example, `[U_10 - U_200]` matches any code
//!   point P where `U_10 ≤ P ≤ U_200`
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
//! ## Compilation
//!
//! When a character class contains only a single item (e.g. `[w]`), the
//! character class is "flattened":
//!
//! - `['a']` = `a`
//! - `[w]` = `\w`
//! - `[Letter]` = `\p{Letter}`
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
//! - `w`, `s`, `d` and Unicode categories/scripts/blocks can be negated
//!   individually _within a character class_, e.g. `[s !s]` = `[\s\S]`,
//!   `![!Latin 'a']` = `[^\P{Latin}a]`.
//!
//!   When a negated character class only contains 1 item, which is also
//! negated, the class is   removed and the negations cancel each other out:
//! `![!w]` = `\w`, `![!L]` = `\p{L}`.

use std::collections::HashSet;

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileError, CompileErrorKind, Feature},
    exprs::literal,
    options::{CompileOptions, RegexFlavor},
    regex::{Regex, RegexProperty, RegexShorthand},
};

use pomsky_syntax::{
    exprs::{Category, CharClass, CodeBlock, GroupItem, GroupName, OtherProperties, Script},
    Span,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for CharClass {
    fn compile(
        &self,
        options: CompileOptions,
        state: &mut CompileState<'_, 'i>,
    ) -> CompileResult<'i> {
        if self.inner.is_empty() {
            // TODO: Check if this is still needed since an empty class is a syntactic error
            return Err(CompileErrorKind::EmptyClass.at(self.span));
        }

        let mut prev_group_items: Vec<GroupItem> = vec![];
        let mut prev_items: HashSet<GroupItem> = HashSet::new();

        let mut negative = self.negative;
        let is_single = self.inner.len() == 1;
        let mut buf = Vec::new();
        for item in &self.inner {
            if prev_items.contains(item) {
                continue;
            }
            prev_items.insert(*item);

            match *item {
                GroupItem::Char(c) => {
                    validate_char_in_class(c, options.flavor, self.span)?;
                    buf.push(RegexCharSetItem::Char(c));
                }
                GroupItem::Range { first, last } => {
                    validate_char_in_class(first, options.flavor, self.span)?;
                    validate_char_in_class(last, options.flavor, self.span)?;
                    buf.push(RegexCharSetItem::Range { first, last });
                }
                GroupItem::Named { name, negative: item_negative } => {
                    if self.negative {
                        check_char_class_empty(*item, &prev_group_items)
                            .map_err(|kind| kind.at(self.span))?;

                        prev_group_items.push(*item);
                    }
                    if state.ascii_only {
                        named_class_to_regex_ascii(
                            name,
                            item_negative,
                            options.flavor,
                            self.span,
                            &mut buf,
                        )?;
                    } else {
                        named_class_to_regex_unicode(
                            name,
                            item_negative,
                            &mut negative,
                            is_single,
                            options.flavor,
                            self.span,
                            &mut buf,
                        )?;
                    }
                }
            }
        }

        Ok(Regex::CharSet(RegexCharSet { negative, items: buf }))
    }
}

fn validate_char_in_class(char: char, flavor: RegexFlavor, span: Span) -> Result<(), CompileError> {
    if flavor == RegexFlavor::DotNet && char > '\u{FFFF}' {
        Err(CompileErrorKind::Unsupported(Feature::LargeCodePointInCharClass(char), flavor)
            .at(span))
    } else {
        Ok(())
    }
}

/// Check if the character set is empty, i.e. matches nothing, e.g. `![w !w]`
fn check_char_class_empty(
    item: GroupItem,
    prev_group_items: &[GroupItem],
) -> Result<(), CompileErrorKind> {
    if let GroupItem::Named { mut name, negative } = item {
        if name == GroupName::Category(Category::Separator) {
            name = GroupName::Space;
        }

        // if the class is negative, it can't contain both `w` and
        // `!w`, where `w` is any group name that can be negated
        let negated = GroupItem::Named { name, negative: !negative };
        if prev_group_items.contains(&negated) {
            return Err(CompileErrorKind::EmptyClassNegated { group1: negated, group2: item });
        }
    }

    Ok(())
}

fn named_class_to_regex_ascii(
    group: GroupName,
    negative: bool,
    flavor: RegexFlavor,
    span: Span,
    buf: &mut Vec<RegexCharSetItem>,
) -> Result<(), CompileError> {
    if negative
        // In JS, \W and \D can be used for negation because they're ascii-only
        && (flavor != RegexFlavor::JavaScript
            || (group != GroupName::Digit && group != GroupName::Word))
    {
        return Err(CompileErrorKind::NegativeShorthandInAsciiMode.at(span));
    }

    match group {
        GroupName::Word => {
            if flavor == RegexFlavor::JavaScript {
                let s = if negative { RegexShorthand::NotWord } else { RegexShorthand::Word };
                buf.push(RegexCharSetItem::Shorthand(s));
            } else {
                // we already checked above if negative
                buf.extend([
                    RegexCharSetItem::Range { first: 'a', last: 'z' },
                    RegexCharSetItem::Range { first: 'A', last: 'Z' },
                    RegexCharSetItem::Range { first: '0', last: '9' },
                    RegexCharSetItem::Char('_'),
                ]);
            }
        }
        GroupName::Digit => {
            if flavor == RegexFlavor::JavaScript {
                let s = if negative { RegexShorthand::NotDigit } else { RegexShorthand::Digit };
                buf.push(RegexCharSetItem::Shorthand(s));
            } else {
                // we already checked above if negative
                buf.push(RegexCharSetItem::Range { first: '0', last: '9' });
            }
        }
        GroupName::Space => buf.extend([
            RegexCharSetItem::Char(' '),
            RegexCharSetItem::Range { first: '\x09', last: '\x0D' }, // \t\n\v\f\r
        ]),
        GroupName::HorizSpace => buf.push(RegexCharSetItem::Char('\t')),
        GroupName::VertSpace => buf.push(RegexCharSetItem::Range { first: '\x0A', last: '\x0D' }),
        _ => return Err(CompileErrorKind::UnicodeInAsciiMode.at(span)),
    }
    Ok(())
}

fn named_class_to_regex_unicode(
    group: GroupName,
    negative: bool,
    group_negative: &mut bool,
    is_single: bool,
    flavor: RegexFlavor,
    span: Span,
    buf: &mut Vec<RegexCharSetItem>,
) -> Result<(), CompileError> {
    match group {
        GroupName::Word => {
            if flavor == RegexFlavor::JavaScript {
                if negative {
                    if is_single {
                        *group_negative ^= true;
                    } else {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::NegativeShorthandW,
                            flavor,
                        )
                        .at(span));
                    }
                }
                buf.extend([
                    RegexProperty::Other(OtherProperties::Alphabetic).negative_item(false),
                    RegexProperty::Category(Category::Mark).negative_item(false),
                    RegexProperty::Category(Category::Decimal_Number).negative_item(false),
                    RegexProperty::Category(Category::Connector_Punctuation).negative_item(false),
                ]);
            } else {
                let s = if negative { RegexShorthand::NotWord } else { RegexShorthand::Word };
                buf.push(RegexCharSetItem::Shorthand(s));
            }
        }
        GroupName::Digit => {
            if flavor == RegexFlavor::JavaScript {
                buf.push(RegexProperty::Category(Category::Decimal_Number).negative_item(negative));
            } else {
                let s = if negative { RegexShorthand::NotDigit } else { RegexShorthand::Digit };
                buf.push(RegexCharSetItem::Shorthand(s));
            }
        }

        GroupName::Space => buf.push(RegexCharSetItem::Shorthand(if negative {
            RegexShorthand::NotSpace
        } else {
            RegexShorthand::Space
        })),

        GroupName::HorizSpace | GroupName::VertSpace if negative => {
            return Err(CompileErrorKind::NegatedHorizVertSpace.at(span));
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
            if flavor == RegexFlavor::Python {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            } else {
                buf.push(RegexProperty::Category(Category::Space_Separator).negative_item(false));
            }
        }
        GroupName::VertSpace => buf.extend([
            RegexCharSetItem::Range { first: '\x0A', last: '\x0D' },
            RegexCharSetItem::Char('\u{85}'),
            RegexCharSetItem::Char('\u{2028}'),
            RegexCharSetItem::Char('\u{2029}'),
        ]),

        _ if flavor == RegexFlavor::Python => {
            return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
        }
        GroupName::Category(c) => {
            if let (RegexFlavor::Rust, Category::Surrogate)
            | (RegexFlavor::DotNet, Category::Cased_Letter) = (flavor, c)
            {
                return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
            }
            buf.push(RegexProperty::Category(c).negative_item(negative));
        }
        GroupName::Script(s) => {
            if flavor == RegexFlavor::DotNet {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeScript, flavor).at(span));
            }
            if let (
                RegexFlavor::Pcre | RegexFlavor::Ruby | RegexFlavor::Java,
                Script::Kawi | Script::Nag_Mundari,
            )
            | (RegexFlavor::Rust, Script::Unknown) = (flavor, s)
            {
                return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
            }
            buf.push(RegexProperty::Script(s).negative_item(negative));
        }
        GroupName::CodeBlock(b) => match flavor {
            RegexFlavor::DotNet | RegexFlavor::Java | RegexFlavor::Ruby => {
                match (flavor, b) {
                    (
                        RegexFlavor::Java,
                        CodeBlock::Arabic_Extended_C
                        | CodeBlock::CJK_Unified_Ideographs_Extension_H
                        | CodeBlock::Combining_Diacritical_Marks_For_Symbols
                        | CodeBlock::Cyrillic_Extended_D
                        | CodeBlock::Cyrillic_Supplement
                        | CodeBlock::Devanagari_Extended_A
                        | CodeBlock::Greek_And_Coptic
                        | CodeBlock::Kaktovik_Numerals
                        | CodeBlock::No_Block,
                    )
                    | (
                        RegexFlavor::Ruby,
                        CodeBlock::Arabic_Extended_C
                        | CodeBlock::CJK_Unified_Ideographs_Extension_H
                        | CodeBlock::Cyrillic_Extended_D
                        | CodeBlock::Devanagari_Extended_A
                        | CodeBlock::Kaktovik_Numerals,
                    ) => {
                        return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
                    }
                    (RegexFlavor::DotNet, _) => {
                        let dotnet_name = b.as_str().replace("_And_", "_and_").replace('_', "");
                        if pomsky_syntax::blocks_supported_in_dotnet()
                            .binary_search(&dotnet_name.as_str())
                            .is_err()
                        {
                            return Err(
                                CompileErrorKind::unsupported_specific_prop_in(flavor).at(span)
                            );
                        }
                    }
                    _ => {}
                }

                buf.push(RegexProperty::Block(b).negative_item(negative));
            }
            _ => return Err(CompileErrorKind::Unsupported(Feature::UnicodeBlock, flavor).at(span)),
        },
        GroupName::OtherProperties(o) => {
            use OtherProperties as OP;
            use RegexFlavor as RF;

            if let RF::JavaScript | RF::Rust | RF::Pcre | RF::Ruby = flavor {
                match (flavor, o) {
                    (RF::JavaScript, _) => {}
                    (_, OP::Changes_When_NFKC_Casefolded)
                    | (RF::Pcre, OP::Assigned)
                    | (RF::Ruby, OP::Bidi_Mirrored) => {
                        return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
                    }
                    _ => {}
                }
                buf.push(RegexProperty::Other(o).negative_item(negative));
            } else {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            }
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
        if self.items.len() == 1 {
            match self.items.first().unwrap() {
                RegexCharSetItem::Shorthand(s) => {
                    let shorthand = if self.negative { s.negate() } else { Some(*s) };
                    if let Some(shorthand) = shorthand {
                        return shorthand.codegen(buf);
                    }
                }
                RegexCharSetItem::Property { negative, value } => {
                    return value.codegen(buf, negative ^ self.negative, flavor);
                }
                RegexCharSetItem::Char(c) if !self.negative => {
                    return literal::codegen_char_esc(*c, buf, flavor);
                }
                _ => {}
            }
        }

        if self.negative {
            buf.push_str("[^");
        } else {
            buf.push('[');
        }

        let mut is_first = true;
        for item in &self.items {
            match *item {
                RegexCharSetItem::Char(c) => {
                    literal::compile_char_esc_in_class(c, buf, is_first, flavor);
                }
                RegexCharSetItem::Range { first, last } => {
                    literal::compile_char_esc_in_class(first, buf, is_first, flavor);
                    buf.push('-');
                    literal::compile_char_esc_in_class(last, buf, false, flavor);
                }
                RegexCharSetItem::Shorthand(s) => s.codegen(buf),
                RegexCharSetItem::Property { negative, value } => {
                    value.codegen(buf, negative, flavor);
                }
            }
            is_first = false;
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
