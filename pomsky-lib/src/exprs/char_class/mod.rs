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
//!   negated, the class is   removed and the negations cancel each other out:
//!   `![!w]` = `\w`, `![!L]` = `\p{L}`.

use pomsky_syntax::exprs::{
    Category, CharClass, CodeBlock, GroupItem, GroupName, OtherProperties, Script, ScriptExtension,
};
use pomsky_syntax::Span;

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileError, CompileErrorKind, Feature},
    exprs::literal,
    options::{CompileOptions, RegexFlavor},
    regex::{Regex, RegexProperty, RegexShorthand},
    unicode_set::UnicodeSet,
};
pub(crate) use char_set_item::{RegexCharSet, RegexCharSetItem, RegexCompoundCharSet};

use super::Compile;

mod char_set_item;

impl Compile for CharClass {
    fn compile(&self, options: CompileOptions, _state: &mut CompileState<'_>) -> CompileResult {
        // when single, a `[!w]` can be turned into `![w]`
        let is_single = self.inner.len() == 1;
        let mut group_negative = false;

        let mut set = UnicodeSet::new();
        for item in &self.inner {
            match *item {
                GroupItem::Char(c) => {
                    if !is_single {
                        validate_char_in_class(c, options.flavor, self.span)?;
                    }
                    set.add_char(c)
                }
                GroupItem::Range { first, last } => {
                    validate_char_in_class(first, options.flavor, self.span)?;
                    validate_char_in_class(last, options.flavor, self.span)?;
                    set.add_range(first..=last);
                }
                GroupItem::Named { name, negative, span } => {
                    if self.unicode_aware {
                        named_class_to_regex_unicode(
                            name,
                            negative,
                            &mut group_negative,
                            is_single,
                            options.flavor,
                            span,
                            &mut set,
                        )?;
                    } else {
                        named_class_to_regex_ascii(name, negative, options.flavor, span, &mut set)?;
                    }
                }
            }
        }

        // this makes it possible to use code points outside the BMP in .NET,
        // as long as there is only one in the character set
        if let Some(only_char) = set.try_into_char() {
            return Ok(Regex::Literal(only_char.to_string()));
        }

        Ok(Regex::CharSet(RegexCharSet { negative: group_negative, set }))
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

pub(crate) fn check_char_class_empty(
    char_set: &RegexCharSet,
    span: Span,
) -> Result<(), CompileError> {
    if char_set.negative {
        if let Some((group1, group2)) = char_set.set.full_props() {
            return Err(CompileErrorKind::EmptyClassNegated { group1, group2 }.at(span));
        }
    }
    Ok(())
}

pub fn is_ascii_only_in_flavor(group: GroupName, flavor: RegexFlavor) -> bool {
    match flavor {
        RegexFlavor::JavaScript => matches!(group, GroupName::Word | GroupName::Digit),
        RegexFlavor::RE2 => matches!(group, GroupName::Word | GroupName::Digit | GroupName::Space),
        _ => false,
    }
}

fn named_class_to_regex_ascii(
    group: GroupName,
    negative: bool,
    flavor: RegexFlavor,
    span: Span,
    set: &mut UnicodeSet,
) -> Result<(), CompileError> {
    // In JS, \W and \D can be used for negation because they're ascii-only
    // Same goes for \W, \D and \S in RE2
    if negative && !is_ascii_only_in_flavor(group, flavor) {
        return Err(CompileErrorKind::NegativeShorthandInAsciiMode.at(span));
    }

    match group {
        GroupName::Word => {
            if let RegexFlavor::JavaScript | RegexFlavor::RE2 = flavor {
                let s = if negative { RegexShorthand::NotWord } else { RegexShorthand::Word };
                set.add_prop(RegexCharSetItem::Shorthand(s));
            } else {
                // we already checked above if negative
                set.add_range('a'..='z');
                set.add_range('A'..='Z');
                set.add_range('0'..='9');
                set.add_char('_');
            }
        }
        GroupName::Digit => {
            if let RegexFlavor::JavaScript | RegexFlavor::RE2 = flavor {
                let s = if negative { RegexShorthand::NotDigit } else { RegexShorthand::Digit };
                set.add_prop(RegexCharSetItem::Shorthand(s));
            } else {
                // we already checked above if negative
                set.add_range('0'..='9');
            }
        }
        GroupName::Space => {
            if let RegexFlavor::RE2 = flavor {
                let s = if negative { RegexShorthand::NotSpace } else { RegexShorthand::Space };
                set.add_prop(RegexCharSetItem::Shorthand(s));
            } else {
                set.add_char(' ');
                set.add_range('\x09'..='\x0D'); // \t\n\v\f\r
            }
        }
        GroupName::HorizSpace => set.add_char('\t'),
        GroupName::VertSpace => set.add_range('\x0A'..='\x0D'),
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
    set: &mut UnicodeSet,
) -> Result<(), CompileError> {
    match group {
        GroupName::Word => {
            if flavor == RegexFlavor::RE2 {
                return Err(CompileErrorKind::Unsupported(Feature::ShorthandW, flavor).at(span));
            } else if flavor == RegexFlavor::JavaScript {
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
                set.add_prop(
                    RegexProperty::Other(OtherProperties::Alphabetic).negative_item(false),
                );
                set.add_prop(RegexProperty::Category(Category::Mark).negative_item(false));
                set.add_prop(
                    RegexProperty::Category(Category::Decimal_Number).negative_item(false),
                );
                set.add_prop(
                    RegexProperty::Category(Category::Connector_Punctuation).negative_item(false),
                );
            } else {
                let s = if negative { RegexShorthand::NotWord } else { RegexShorthand::Word };
                set.add_prop(RegexCharSetItem::Shorthand(s));
            }
        }
        GroupName::Digit => {
            if matches!(flavor, RegexFlavor::JavaScript | RegexFlavor::RE2) {
                set.add_prop(
                    RegexProperty::Category(Category::Decimal_Number).negative_item(negative),
                );
            } else {
                let s = if negative { RegexShorthand::NotDigit } else { RegexShorthand::Digit };
                set.add_prop(RegexCharSetItem::Shorthand(s));
            }
        }

        GroupName::Space => {
            if flavor == RegexFlavor::RE2 {
                if negative {
                    if is_single {
                        *group_negative ^= true;
                    } else {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::NegativeShorthandS,
                            flavor,
                        )
                        .at(span));
                    }
                }

                // [ \f\n\r\t\u000b\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000\ufeff]
                set.add_prop(RegexCharSetItem::Shorthand(RegexShorthand::Space));
                set.add_char('\x0b');
                set.add_char('\u{a0}');
                set.add_char('\u{1680}');
                set.add_range('\u{2000}'..='\u{200a}');
                set.add_range('\u{2028}'..='\u{2029}');
                set.add_char('\u{202f}');
                set.add_char('\u{205f}');
                set.add_char('\u{3000}');
                set.add_char('\u{feff}');
            } else {
                set.add_prop(RegexCharSetItem::Shorthand(if negative {
                    RegexShorthand::NotSpace
                } else {
                    RegexShorthand::Space
                }))
            }
        }

        GroupName::HorizSpace | GroupName::VertSpace if negative => {
            return Err(CompileErrorKind::NegatedHorizVertSpace.at(span));
        }

        GroupName::HorizSpace | GroupName::VertSpace
            if matches!(flavor, RegexFlavor::Pcre | RegexFlavor::Java) =>
        {
            set.add_prop(RegexCharSetItem::Shorthand(if group == GroupName::HorizSpace {
                RegexShorthand::HorizSpace
            } else {
                RegexShorthand::VertSpace
            }));
        }
        GroupName::HorizSpace => {
            set.add_char('\t');
            if flavor == RegexFlavor::Python {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            } else {
                set.add_prop(
                    RegexProperty::Category(Category::Space_Separator).negative_item(false),
                );
            }
        }
        GroupName::VertSpace => {
            set.add_range('\x0A'..='\x0D');
            set.add_char('\u{85}');
            set.add_char('\u{2028}');
            set.add_char('\u{2029}');
        }

        _ if flavor == RegexFlavor::Python => {
            return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
        }
        GroupName::Category(c) => {
            if let (RegexFlavor::Rust, Category::Surrogate)
            | (RegexFlavor::DotNet | RegexFlavor::RE2, Category::Cased_Letter) = (flavor, c)
            {
                return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
            }
            set.add_prop(RegexProperty::Category(c).negative_item(negative));
        }
        GroupName::Script(s, e) => {
            if flavor == RegexFlavor::DotNet {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeScript, flavor).at(span));
            }
            if let (RegexFlavor::Ruby, Script::Kawi | Script::Nag_Mundari)
            | (RegexFlavor::Rust, Script::Unknown) = (flavor, s)
            {
                return Err(CompileErrorKind::unsupported_specific_prop_in(flavor).at(span));
            }

            let set_extensions = match e {
                ScriptExtension::Yes => match flavor {
                    RegexFlavor::Rust | RegexFlavor::Pcre | RegexFlavor::JavaScript => {
                        ScriptExtension::Yes
                    }
                    RegexFlavor::Java
                    | RegexFlavor::DotNet
                    | RegexFlavor::Ruby
                    | RegexFlavor::Python
                    | RegexFlavor::RE2 => {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::ScriptExtensions,
                            flavor,
                        )
                        .at(span))
                    }
                },
                ScriptExtension::No => match flavor {
                    // PCRE is currently the only flavor when `\p{Greek}` is the same as `\p{scx=Greek}`
                    RegexFlavor::Pcre => ScriptExtension::No,
                    _ => ScriptExtension::Unspecified,
                },
                _ => ScriptExtension::Unspecified,
            };

            set.add_prop(RegexProperty::Script(s, set_extensions).negative_item(negative));
        }
        GroupName::CodeBlock(b) => match flavor {
            RegexFlavor::DotNet | RegexFlavor::Java | RegexFlavor::Ruby => {
                match (flavor, b) {
                    (RegexFlavor::Java, CodeBlock::No_Block)
                    | (
                        // These should work since Oniguruma updated to Unicode 15.1
                        // ... but our C bindings for Oniguruma are unmaintained!
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

                set.add_prop(RegexProperty::Block(b).negative_item(negative));
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
                set.add_prop(RegexProperty::Other(o).negative_item(negative));
            } else if flavor == RF::Java {
                if pomsky_syntax::props_supported_in_java().binary_search(&o.as_str()).is_ok() {
                    set.add_prop(RegexProperty::Other(o).negative_item(negative));
                } else {
                    return Err(CompileErrorKind::Unsupported(
                        Feature::SpecificUnicodeProp,
                        flavor,
                    )
                    .at(span));
                }
            } else {
                return Err(CompileErrorKind::Unsupported(Feature::UnicodeProp, flavor).at(span));
            }
        }
    }
    Ok(())
}
