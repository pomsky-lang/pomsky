use std::borrow::Borrow;

use pomsky_syntax::{
    exprs::{
        BoundaryKind, Category, CodeBlock, LookaroundKind, OtherProperties, RepetitionKind, Script,
    },
    Span,
};

use crate::{
    compile::CompileResult,
    diagnose::{CompileErrorKind, Feature, IllegalNegationKind},
    exprs::{
        alternation::RegexAlternation,
        boundary::boundary_kind_codegen,
        char_class::{RegexCharSet, RegexCharSetItem},
        group::{RegexGroup, RegexGroupKind},
        literal,
        lookaround::RegexLookaround,
        recursion,
        reference::RegexReference,
        repetition::RegexRepetition,
    },
    options::RegexFlavor,
};

mod optimize;

pub(super) use optimize::Count;

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum Regex {
    /// A literal string
    Literal(String),
    /// A regex string that is inserted verbatim into the output
    Unescaped(String),
    /// A character class, delimited with square brackets
    CharSet(RegexCharSet),
    /// A Unicode grapheme
    Grapheme,
    /// The dot, matching anything except `\n`
    Dot,
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(RegexGroup),
    /// An alternation, i.e. a list of alternatives; at least one of them has to
    /// match.
    Alternation(RegexAlternation),
    /// A repetition, i.e. a expression that must be repeated. The number of
    /// required repetitions is constrained by a lower and possibly an upper
    /// bound.
    Repetition(Box<RegexRepetition>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(BoundaryKind),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<RegexLookaround>),
    /// A backreference or forward reference.
    Reference(RegexReference),
    /// Recursively matches the entire regex.
    Recursion,
}

impl Regex {
    pub(super) fn validate_in_lookbehind_py(&self) -> Result<Option<u32>, CompileErrorKind> {
        match self {
            Regex::Literal(str) => Ok(Some(str.chars().count() as u32)),
            Regex::Unescaped(_) => Ok(None),
            Regex::CharSet(_) => Ok(Some(1)),
            Regex::Grapheme => Err(CompileErrorKind::UnsupportedInLookbehind {
                flavor: RegexFlavor::Python,
                feature: Feature::Grapheme,
            }),
            Regex::Dot => Ok(Some(1)),
            Regex::Group(g) => g.parts.iter().try_fold(Some(0), |acc, part| {
                Ok(match (acc, part.validate_in_lookbehind_py()?) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                })
            }),
            Regex::Alternation(alt) => {
                let mut count = None;
                for part in &alt.parts {
                    let c = part.validate_in_lookbehind_py()?;
                    count = match (count, c) {
                        (Some(a), Some(b)) if a == b => Some(a),
                        (Some(_), Some(_)) => {
                            return Err(CompileErrorKind::LookbehindNotConstantLength {
                                flavor: RegexFlavor::Python,
                            })
                        }
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        _ => None,
                    };
                }
                Ok(count)
            }
            Regex::Repetition(r) => {
                if let RepetitionKind { lower_bound, upper_bound: Some(upper) } = r.kind {
                    if lower_bound == upper {
                        return Ok(Some(upper));
                    }
                }
                Err(CompileErrorKind::LookbehindNotConstantLength { flavor: RegexFlavor::Python })
            }
            Regex::Boundary(_) => Ok(Some(0)),
            Regex::Lookaround(_) => Ok(Some(0)),
            Regex::Reference(_) => Ok(None), // TODO: somehow get the length of the referenced group
            Regex::Recursion => unreachable!("not supported in python"),
        }
    }

    pub(super) fn validate_in_lookbehind_pcre(&self) -> Result<(), CompileErrorKind> {
        match self {
            Regex::Literal(_) => Ok(()),
            Regex::Unescaped(_) => Ok(()),
            Regex::CharSet(_) => Ok(()),
            Regex::Grapheme => Err(CompileErrorKind::UnsupportedInLookbehind {
                flavor: RegexFlavor::Pcre,
                feature: Feature::Grapheme,
            }),
            Regex::Dot => Ok(()),
            Regex::Group(g) => {
                for part in &g.parts {
                    part.validate_in_lookbehind_pcre()?;
                }
                Ok(())
            }
            Regex::Alternation(alt) => {
                for part in &alt.parts {
                    part.validate_in_lookbehind_pcre()?;
                }
                Ok(())
            }
            Regex::Repetition(r) => match r.kind.upper_bound {
                Some(_) => Ok(()),
                _ => {
                    Err(CompileErrorKind::LookbehindNotConstantLength { flavor: RegexFlavor::Pcre })
                }
            },
            Regex::Boundary(_) => Ok(()),
            Regex::Lookaround(_) => Ok(()),
            Regex::Reference(_) => Ok(()), // TODO: somehow check the referenced group
            Regex::Recursion => Err(CompileErrorKind::UnsupportedInLookbehind {
                flavor: RegexFlavor::Pcre,
                feature: Feature::Recursion,
            }),
        }
    }
}

impl Default for Regex {
    fn default() -> Self {
        Regex::Literal("".into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl RegexShorthand {
    pub(crate) fn negate(&self) -> Option<RegexShorthand> {
        Some(match self {
            RegexShorthand::Word => RegexShorthand::NotWord,
            RegexShorthand::Digit => RegexShorthand::NotDigit,
            RegexShorthand::Space => RegexShorthand::NotSpace,
            RegexShorthand::NotWord => RegexShorthand::Word,
            RegexShorthand::NotDigit => RegexShorthand::Digit,
            RegexShorthand::NotSpace => RegexShorthand::Space,
            RegexShorthand::VertSpace => return None,
            RegexShorthand::HorizSpace => return None,
        })
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            RegexShorthand::Word => "word",
            RegexShorthand::Digit => "digit",
            RegexShorthand::Space => "space",
            RegexShorthand::NotWord => "!word",
            RegexShorthand::NotDigit => "!digit",
            RegexShorthand::NotSpace => "!space",
            RegexShorthand::VertSpace => "vert_space",
            RegexShorthand::HorizSpace => "horiz_space",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexProperty {
    Category(Category),
    Script(Script),
    Block(CodeBlock),
    Other(OtherProperties),
}

impl RegexProperty {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegexProperty::Category(c) => c.as_str(),
            RegexProperty::Script(s) => s.as_str(),
            RegexProperty::Block(b) => b.as_str(),
            RegexProperty::Other(o) => o.as_str(),
        }
    }

    pub(crate) fn negative_item(self, negative: bool) -> RegexCharSetItem {
        RegexCharSetItem::Property { negative, value: self }
    }
}

impl Regex {
    pub(crate) fn negate(self, not_span: Span, flavor: RegexFlavor) -> CompileResult {
        match self {
            Regex::Literal(l) => {
                let mut iter = l.chars();
                let Some(c) = iter.next().and_then(|c| iter.next().is_none().then_some(c)) else {
                    return Err(CompileErrorKind::IllegalNegation {
                        kind: IllegalNegationKind::Literal(l.to_string()),
                    }
                    .at(not_span));
                };
                if flavor == RegexFlavor::DotNet && c.len_utf16() > 1 {
                    return Err(CompileErrorKind::IllegalNegation {
                        kind: IllegalNegationKind::DotNetChar(c),
                    }
                    .at(not_span));
                }
                Ok(Regex::CharSet(RegexCharSet::new(c.into()).negate()))
            }
            Regex::CharSet(s) => Ok(Regex::CharSet(s.negate())),
            Regex::Boundary(b) => match b {
                BoundaryKind::Word => Ok(Regex::Boundary(BoundaryKind::NotWord)),
                BoundaryKind::NotWord => Ok(Regex::Boundary(BoundaryKind::Word)),
                _ => Err(CompileErrorKind::IllegalNegation { kind: IllegalNegationKind::Boundary }
                    .at(not_span)),
            },
            Regex::Lookaround(mut l) => {
                l.kind = match l.kind {
                    LookaroundKind::Ahead => LookaroundKind::AheadNegative,
                    LookaroundKind::Behind => LookaroundKind::BehindNegative,
                    LookaroundKind::AheadNegative => LookaroundKind::Ahead,
                    LookaroundKind::BehindNegative => LookaroundKind::Behind,
                };
                Ok(Regex::Lookaround(l))
            }
            Regex::Group(mut g)
                if matches!(g.kind, RegexGroupKind::Normal) && g.parts.len() == 1 =>
            {
                g.parts.pop().unwrap().negate(not_span, flavor)
            }

            Regex::Unescaped(_)
            | Regex::Grapheme
            | Regex::Dot
            | Regex::Group(_)
            | Regex::Alternation(_)
            | Regex::Repetition(_)
            | Regex::Reference(_)
            | Regex::Recursion => Err(CompileErrorKind::IllegalNegation {
                kind: match self {
                    Regex::Unescaped(_) => IllegalNegationKind::Unescaped,
                    Regex::Grapheme => IllegalNegationKind::Grapheme,
                    Regex::Dot => IllegalNegationKind::Dot,
                    Regex::Group(_) => IllegalNegationKind::Group,
                    Regex::Alternation(_) => IllegalNegationKind::Alternation,
                    Regex::Repetition(_) => IllegalNegationKind::Repetition,
                    Regex::Reference(_) => IllegalNegationKind::Reference,
                    Regex::Recursion => IllegalNegationKind::Recursion,
                    _ => unreachable!(),
                },
            }
            .at(not_span)),
        }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match self {
            Regex::Literal(l) => {
                // normalize line breaks: within string literals, \r, \n and \r\n should be
                // converted to \n
                let mut chars = l.chars();
                while let Some(c) = chars.next() {
                    if c == '\r' {
                        literal::codegen_char_esc('\n', buf, flavor);
                        match chars.next() {
                            Some('\n') | None => {}
                            Some(c) => literal::codegen_char_esc(c, buf, flavor),
                        }
                    } else {
                        literal::codegen_char_esc(c, buf, flavor);
                    }
                }
            }
            Regex::Unescaped(u) => {
                buf.push_str(u);
            }
            Regex::CharSet(c) => c.codegen(buf, flavor),
            Regex::Grapheme => buf.push_str("\\X"),
            Regex::Dot => buf.push('.'),
            Regex::Group(g) => g.codegen(buf, flavor),
            Regex::Alternation(a) => a.codegen(buf, flavor),
            Regex::Repetition(r) => r.codegen(buf, flavor),
            Regex::Boundary(b) => boundary_kind_codegen(*b, buf, flavor),
            Regex::Lookaround(l) => l.codegen(buf, flavor),
            Regex::Reference(r) => r.codegen(buf),
            Regex::Recursion => recursion::codegen(buf, flavor),
        }
    }

    pub(crate) fn needs_parens_in_sequence(&self) -> bool {
        match self {
            Regex::Alternation(_) => true,
            Regex::Literal(_)
            | Regex::Unescaped(_)
            | Regex::Group(_)
            | Regex::CharSet(_)
            | Regex::Grapheme
            | Regex::Repetition(_)
            | Regex::Boundary(_)
            | Regex::Lookaround(_)
            | Regex::Reference(_)
            | Regex::Dot
            | Regex::Recursion => false,
        }
    }

    pub(crate) fn needs_parens_before_repetition(&self, flavor: RegexFlavor) -> bool {
        match self {
            Regex::Literal(l) => literal::needs_parens_before_repetition(l.borrow()),
            Regex::Group(g) => g.needs_parens_before_repetition(flavor),
            Regex::Repetition(_)
            | Regex::Alternation(_)
            | Regex::Boundary(_)
            | Regex::Unescaped(_) => true,
            Regex::Lookaround(_) => matches!(flavor, RegexFlavor::JavaScript),
            Regex::CharSet(_)
            | Regex::Grapheme
            | Regex::Reference(_)
            | Regex::Dot
            | Regex::Recursion => false,
        }
    }

    pub(crate) fn result_is_empty(&self) -> bool {
        match self {
            Regex::Literal(l) => l.is_empty(),
            Regex::Group(g) => g.parts.iter().all(Regex::result_is_empty),
            Regex::Unescaped(r) => r.is_empty(),
            Regex::Repetition(r) => r.content.result_is_empty(),
            _ => false,
        }
    }

    pub(crate) fn is_assertion(&self) -> bool {
        match self {
            Regex::Lookaround(_) | Regex::Boundary(_) => true,
            Regex::Group(g) if matches!(g.kind, RegexGroupKind::Normal) => {
                let mut iter = g.parts.iter().filter(|part| !part.result_is_empty());
                iter.next().is_some_and(Regex::is_assertion) && iter.next().is_none()
            }
            Regex::Alternation(g) => g.parts.iter().any(Regex::is_assertion),
            _ => false,
        }
    }
}

impl RegexShorthand {
    pub(crate) fn codegen(self, buf: &mut String) {
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
    pub(crate) fn codegen(self, buf: &mut String, negative: bool, flavor: RegexFlavor) {
        let is_single = matches!(
            (self, flavor),
            (
                RegexProperty::Category(
                    Category::Letter
                        | Category::Mark
                        | Category::Number
                        | Category::Punctuation
                        | Category::Symbol
                        | Category::Separator
                        | Category::Other
                ),
                RegexFlavor::Java | RegexFlavor::Pcre | RegexFlavor::Rust | RegexFlavor::Ruby,
            )
        );
        if negative {
            buf.push_str("\\P");
        } else {
            buf.push_str("\\p");
        }
        if !is_single {
            buf.push('{');
        }

        match self {
            RegexProperty::Category(c) => {
                if let (RegexFlavor::Rust, Category::Cased_Letter | Category::Currency_Symbol) =
                    (flavor, c)
                {
                    buf.push_str("gc=");
                }
                buf.push_str(c.as_str());
            }
            RegexProperty::Script(s) => {
                if let RegexFlavor::JavaScript | RegexFlavor::Java = flavor {
                    buf.push_str("sc=");
                }
                buf.push_str(s.as_str());
            }
            RegexProperty::Block(b) => match flavor {
                RegexFlavor::DotNet => {
                    buf.push_str("Is");
                    buf.push_str(&b.as_str().replace("_And_", "_and_").replace('_', ""));
                }
                RegexFlavor::Java => {
                    buf.push_str("In");
                    buf.push_str(&b.as_str().replace('-', "_"));
                }
                _ => {
                    buf.push_str("In");
                    buf.push_str(b.as_str());
                }
            },
            RegexProperty::Other(o) => {
                if flavor == RegexFlavor::Java {
                    // Currently disabled since only some boolean properties are supported in Java
                    buf.push_str("Is");
                }
                buf.push_str(o.as_str());
            }
        }

        if !is_single {
            buf.push('}');
        }
    }
}
