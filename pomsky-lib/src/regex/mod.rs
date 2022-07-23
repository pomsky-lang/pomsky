use std::borrow::{Borrow, Cow};

use crate::{
    exprs::{
        alternation::RegexAlternation,
        boundary::BoundaryKind,
        char_class::{
            unicode::{Category, CodeBlock, OtherProperties, Script},
            RegexCharClass, RegexClassItem,
        },
        group::RegexGroup,
        literal,
        lookaround::RegexLookaround,
        reference::RegexReference,
        repetition::RegexRepetition,
    },
    options::RegexFlavor,
};

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum Regex<'i> {
    /// A literal string
    Literal(Cow<'i, str>),
    /// A literal char
    Char(char),
    /// A character class, delimited with square brackets
    CharClass(RegexCharClass),
    /// A shorthand such as `\w`
    Shorthand(RegexShorthand),
    /// A (Unicode) property such as Letter, Greek or Alphabetic
    Property { value: RegexProperty, negative: bool },
    /// A Unicode grapheme
    Grapheme,
    /// The dot, matching anything except `\n`
    Dot,
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(RegexGroup<'i>),
    /// An alternation, i.e. a list of alternatives; at least one of them has to
    /// match.
    Alternation(RegexAlternation<'i>),
    /// A repetition, i.e. a expression that must be repeated. The number of
    /// required repetitions is constrained by a lower and possibly an upper
    /// bound.
    Repetition(Box<RegexRepetition<'i>>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(BoundaryKind),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<RegexLookaround<'i>>),
    /// A backreference or forward reference.
    Reference(RegexReference),
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexShorthand {
    Word,
    Digit,
    Space,
    NotWord,
    NotDigit,
    NotSpace,
    VertSpace,
    HorizSpace,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexProperty {
    Category(Category),
    Script(Script),
    Block(CodeBlock),
    Other(OtherProperties),
}

impl RegexProperty {
    pub(crate) fn negative_item(self, negative: bool) -> RegexClassItem {
        RegexClassItem::Property { negative, value: self }
    }

    pub(crate) fn negative(self, negative: bool) -> Regex<'static> {
        Regex::Property { negative, value: self }
    }
}

impl<'i> Regex<'i> {
    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match self {
            Regex::Literal(l) => {
                for c in l.chars() {
                    literal::codegen_char_esc(c, buf, flavor);
                }
            }
            &Regex::Char(c) => {
                literal::codegen_char_esc(c, buf, flavor);
            }
            Regex::CharClass(c) => c.codegen(buf, flavor),
            Regex::Shorthand(s) => s.codegen(buf),
            Regex::Property { value, negative } => value.codegen(buf, *negative, flavor),
            Regex::Grapheme => buf.push_str("\\X"),
            Regex::Dot => buf.push('.'),
            Regex::Group(g) => g.codegen(buf, flavor),
            Regex::Alternation(a) => a.codegen(buf, flavor),
            Regex::Repetition(r) => r.codegen(buf, flavor),
            Regex::Boundary(b) => b.codegen(buf),
            Regex::Lookaround(l) => l.codegen(buf, flavor),
            Regex::Reference(r) => r.codegen(buf, flavor),
        }
    }

    pub(crate) fn needs_parens_in_group(&self) -> bool {
        match self {
            Regex::Alternation(_) => true,
            Regex::Literal(_)
            | Regex::Char(_)
            | Regex::Group(_)
            | Regex::CharClass(_)
            | Regex::Grapheme
            | Regex::Repetition(_)
            | Regex::Boundary(_)
            | Regex::Lookaround(_)
            | Regex::Reference(_)
            | Regex::Shorthand(_)
            | Regex::Property { .. }
            | Regex::Dot => false,
        }
    }

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        match self {
            Regex::Literal(l) => literal::needs_parens_before_repetition(l.borrow()),
            Regex::Group(g) => g.needs_parens_before_repetition(),
            Regex::Repetition(_) | Regex::Alternation(_) => true,
            Regex::CharClass(_)
            | Regex::Char(_)
            | Regex::Grapheme
            | Regex::Boundary(_)
            | Regex::Lookaround(_)
            | Regex::Reference(_)
            | Regex::Shorthand(_)
            | Regex::Property { .. }
            | Regex::Dot => false,
        }
    }
}

impl RegexShorthand {
    pub(crate) fn codegen(&self, buf: &mut String) {
        match self {
            RegexShorthand::Word => buf.push_str("\\w"),
            RegexShorthand::Digit => buf.push_str("\\d"),
            RegexShorthand::Space => buf.push_str("\\s"),
            RegexShorthand::NotWord => buf.push_str("\\W"),
            RegexShorthand::NotDigit => buf.push_str("\\D"),
            RegexShorthand::NotSpace => buf.push_str("\\S"),
            RegexShorthand::VertSpace => buf.push_str("\\v"),
            RegexShorthand::HorizSpace => buf.push_str("\\h"),
        }
    }
}

impl RegexProperty {
    pub(crate) fn codegen(&self, buf: &mut String, negative: bool, flavor: RegexFlavor) {
        if negative {
            buf.push_str("\\P{");
        } else {
            buf.push_str("\\p{");
        }
        match self {
            RegexProperty::Category(c) => buf.push_str(c.as_str()),
            RegexProperty::Script(s) => buf.push_str(s.as_str()),
            RegexProperty::Block(b) => match flavor {
                RegexFlavor::DotNet => {
                    buf.push_str("Is");
                    buf.push_str(&b.as_str().replace('_', ""));
                }
                RegexFlavor::Java => {
                    buf.push_str("In");
                    buf.push_str(&b.as_str().replace('-', ""));
                }
                _ => {
                    buf.push_str("In");
                    buf.push_str(b.as_str());
                }
            },
            RegexProperty::Other(o) => buf.push_str(o.as_str()),
        }
        buf.push('}');
    }
}
