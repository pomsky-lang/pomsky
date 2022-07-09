use crate::{parse::ParseErrorMsg, repetition::RepetitionError, span::Span, warning::Warning};

use super::{
    compile_error::CompileErrorKind, CharClassError, CharStringError, CompileError, ParseError,
    ParseErrorKind,
};

#[cfg_attr(feature = "miette", derive(Debug, thiserror::Error))]
#[cfg_attr(feature = "miette", error("{}", .msg))]
#[non_exhaustive]
/// A struct containing detailed information about an error, which can be
/// displayed beautifully with [miette](https://docs.rs/miette/latest/miette/).
pub struct Diagnostic {
    /// Whether this is an error, a warning or advice
    pub severity: Severity,
    /// The error message
    pub msg: String,
    /// The error code (optional, currently unused)
    pub code: Option<String>,
    /// The source code where the error occurred
    pub source_code: String,
    /// An (optional) help message explaining how the error could be fixed
    pub help: Option<String>,
    /// The start and end byte positions of the source code where the error
    /// occurred.
    pub span: Span,
}

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
}

#[cfg(feature = "miette")]
impl miette::Diagnostic for Diagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.code.as_deref().map(|c| Box::new(c) as Box<dyn std::fmt::Display + 'a>)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help.as_deref().map(|h| Box::new(h) as Box<dyn std::fmt::Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_code)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            Some(Box::new(
                [miette::LabeledSpan::new(
                    Some(
                        (match self.severity {
                            Severity::Error => "error occurred here",
                            Severity::Warning => "warning originated here",
                        })
                        .into(),
                    ),
                    start,
                    end - start,
                )]
                .into_iter(),
            ))
        } else {
            None
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
        })
    }
}

impl Diagnostic {
    /// Create a `Diagnostic` from a [`ParseError`]
    pub fn from_parse_error(error: ParseError, source_code: &str) -> Self {
        let range = error.span.range().unwrap_or(0..source_code.len());
        let slice = &source_code[range.clone()];
        let mut span = Span::from(range);

        let help = match error.kind {
            ParseErrorKind::LexErrorWithMessage(msg) => match msg {
                ParseErrorMsg::Caret => Some("Use `Start` to match the start of the string".into()),
                ParseErrorMsg::CaretInGroup => {
                    Some("Use `![...]` to negate a character class".into())
                }
                ParseErrorMsg::Dollar => Some("Use `End` to match the end of the string".into()),
                ParseErrorMsg::GroupNonCapturing => Some(
                    "Non-capturing groups are just parentheses: `(...)`. \
                    Capturing groups use the `:(...)` syntax."
                        .into(),
                ),
                ParseErrorMsg::GroupLookahead => Some(
                    "Lookahead uses the `>>` syntax. \
                    For example, `>> 'bob'` matches if the position is followed by bob."
                        .into(),
                ),
                ParseErrorMsg::GroupLookaheadNeg => Some(
                    "Negative lookahead uses the `!>>` syntax. \
                    For example, `!>> 'bob'` matches if the position is not followed by bob."
                        .into(),
                ),
                ParseErrorMsg::GroupLookbehind => Some(
                    "Lookbehind uses the `<<` syntax. \
                    For example, `<< 'bob'` matches if the position is preceded with bob."
                        .into(),
                ),
                ParseErrorMsg::GroupLookbehindNeg => Some(
                    "Negative lookbehind uses the `!<<` syntax. \
                    For example, `!<< 'bob'` matches if the position is not preceded with bob."
                        .into(),
                ),
                ParseErrorMsg::GroupComment => {
                    Some("Comments start with `#` and go until the end of the line.".into())
                }
                ParseErrorMsg::GroupNamedCapture => get_named_capture_help(slice),
                ParseErrorMsg::GroupPcreBackreference => get_pcre_backreference_help(slice),
                ParseErrorMsg::Backslash => get_backslash_help(slice),
                ParseErrorMsg::BackslashU4 => get_backslash_help_u4(slice),
                ParseErrorMsg::BackslashX2 => get_backslash_help_x2(slice),
                ParseErrorMsg::BackslashUnicode => get_backslash_help_unicode(slice),
                ParseErrorMsg::BackslashGK => get_backslash_gk_help(slice),
                ParseErrorMsg::BackslashProperty => get_backslash_property_help(slice),

                ParseErrorMsg::GroupAtomic
                | ParseErrorMsg::GroupConditional
                | ParseErrorMsg::GroupBranchReset
                | ParseErrorMsg::GroupSubroutineCall
                | ParseErrorMsg::GroupOther
                | ParseErrorMsg::UnclosedString => None,
            },
            ParseErrorKind::RangeIsNotIncreasing => {
                let dash_pos = slice.find('-').unwrap();
                let (part1, part2) = slice.split_at(dash_pos);
                let part2 = part2.trim_start_matches('-');
                Some(format!("Switch the numbers: {}-{}", part2.trim(), part1.trim()))
            }
            ParseErrorKind::Dot => Some(
                "The dot is deprecated. Use `Codepoint` to match any code point, \
                or `![n]` to exclude line breaks"
                    .into(),
            ),
            ParseErrorKind::CharClass(CharClassError::DescendingRange(..)) => {
                let dash_pos = slice.find('-').unwrap();
                let (part1, part2) = slice.split_at(dash_pos);
                let part2 = part2.trim_start_matches('-');
                Some(format!("Switch the characters: {}-{}", part2.trim(), part1.trim()))
            }
            ParseErrorKind::CharClass(CharClassError::Empty) => {
                Some("You can use `[s !s]` to match nothing".into())
            }
            ParseErrorKind::CharString(CharStringError::TooManyCodePoints)
                if slice.trim_matches(&['"', '\''][..]).chars().all(|c| c.is_ascii_digit()) =>
            {
                Some(
                    "Try a `range` expression instead:\n\
                    https://pomsky-lang.org/docs/language-tour/ranges/"
                        .into(),
                )
            }
            ParseErrorKind::KeywordAfterLet(_) => Some("Use a different variable name".into()),
            ParseErrorKind::UnallowedDoubleNot => Some("Remove 2 exclamation marks".into()),
            ParseErrorKind::LetBindingExists => Some("Use a different name".into()),
            ParseErrorKind::Repetition(RepetitionError::QuestionMarkAfterRepetition) => Some(
                "If you meant to make the repetition lazy, append the `lazy` keyword instead.\n\
                If this is intentional, consider adding parentheses around the inner repetition."
                    .into(),
            ),
            ParseErrorKind::InvalidEscapeInStringAt(offset) => {
                let span_start = span.range_unchecked().start;
                span = Span::new(span_start + offset - 1, span_start + offset + 1);
                None
            }
            ParseErrorKind::RecursionLimit => Some(
                "Try a less nested expression. It helps to refactor it using variables:\n\
                https://pomsky-lang.org/docs/language-tour/variables/"
                    .into(),
            ),
            _ => None,
        };

        Diagnostic {
            severity: Severity::Error,
            code: None,
            msg: error.kind.to_string(),
            source_code: source_code.into(),
            help,
            span,
        }
    }

    /// Create a `Diagnostic` from a [`CompileError`]
    pub fn from_compile_error(
        CompileError { kind, span }: CompileError,
        source_code: &str,
    ) -> Self {
        match kind {
            CompileErrorKind::ParseError(kind) => {
                Diagnostic::from_parse_error(ParseError { kind, span }, source_code)
            }
            _ => {
                let range = span.range().unwrap_or(0..source_code.len());
                let span = Span::from(range);

                Diagnostic {
                    severity: Severity::Error,
                    code: None,
                    msg: kind.to_string(),
                    source_code: source_code.into(),
                    help: None,
                    span,
                }
            }
        }
    }

    /// Create a `Diagnostic` from a [`CompileError`]
    pub fn from_warning(warning: Warning, source_code: &str) -> Self {
        let range = warning.span.range().unwrap_or(0..source_code.len());
        let span = Span::from(range);

        Diagnostic {
            severity: Severity::Warning,
            code: None,
            msg: warning.kind.to_string(),
            source_code: source_code.into(),
            help: None,
            span,
        }
    }
}

fn get_named_capture_help(str: &str) -> Option<String> {
    // (?<name>), (?P<name>)
    let name =
        str.trim_start_matches("(?").trim_start_matches('P').trim_matches(&['<', '>', '\''][..]);

    if name.contains('-') {
        Some("Balancing groups are not supported".into())
    } else {
        Some(format!(
            "Named capturing groups use the `:name(...)` syntax. Try `:{name}(...)` instead"
        ))
    }
}

fn get_pcre_backreference_help(str: &str) -> Option<String> {
    // (?P=name)
    let name = str.trim_start_matches("(?P=").trim_end_matches(')');
    Some(format!("Backreferences use the `::name` syntax. Try `::{name}` instead"))
}

fn get_backslash_help(str: &str) -> Option<String> {
    assert!(str.starts_with('\\'));
    let str = &str[1..];
    let mut iter = str.chars();

    Some(match iter.next() {
        Some('b') => "Replace `\\b` with `%` to match a word boundary".into(),
        Some('B') => "Replace `\\B` with `!%` to match a place without a word boundary".into(),
        Some('A') => "Replace `\\A` with `Start` to match the start of the string".into(),
        Some('z') => "Replace `\\z` with `End` to match the end of the string".into(),
        Some('Z') => "\\Z is not supported. Use `End` to match the end of the string.\n\
            Note, however, that `End` doesn't match the position before the final newline."
            .into(),
        Some('N') => "Replace `\\N` with `![n]`".into(),
        Some('X') => "Replace `\\X` with `Grapheme`".into(),
        Some('R') => "Replace `\\R` with `([r] [n] | [v])`".into(),
        Some('D') => "Replace `\\D` with `[!d]`".into(),
        Some('W') => "Replace `\\W` with `[!w]`".into(),
        Some('S') => "Replace `\\S` with `[!s]`".into(),
        Some('V') => "Replace `\\V` with `![v]`".into(),
        Some('H') => "Replace `\\H` with `![h]`".into(),
        Some('G') => "Match attempt anchors are not supported".into(),
        Some(c @ ('a' | 'e' | 'f' | 'n' | 'r' | 't' | 'h' | 'v' | 'd' | 'w' | 's')) => {
            format!("Replace `\\{c}` with `[{c}]`")
        }
        Some('0') => "Replace `\\0` with `U+00`".into(),
        Some(c @ '1'..='7') => format!(
            "If this is a backreference, replace it with `::{c}`.\n\
            If this is an octal escape, replace it with `U+0{c}`."
        ),
        Some(c @ '1'..='9') => format!("Replace `\\{c}` with `::{c}`"),
        _ => return None,
    })
}

fn get_backslash_help_u4(str: &str) -> Option<String> {
    // \uFFFF
    let hex = &str[2..];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_x2(str: &str) -> Option<String> {
    // \xFF
    let hex = &str[2..];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_unicode(str: &str) -> Option<String> {
    // \u{...}, \x{...}
    let hex = str[2..].trim_matches(&['{', '}'][..]);
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_gk_help(str: &str) -> Option<String> {
    // \k<name>, \k'name', \k{name}, \k0, \k-1, \k+1,
    // \g<name>, \g'name', \g{name}, \g0, \g-1, \g+1
    let name = str[2..].trim_matches(&['{', '}', '<', '>', '\''][..]);

    if name == "0" {
        Some("Recursion is currently not supported".to_string())
    } else {
        Some(format!("Replace `{str}` with `::{name}`"))
    }
}

fn get_backslash_property_help(str: &str) -> Option<String> {
    // \pL, \PL, \p{Letter}, \P{Letter}, \p{^Letter}, \P{^Letter}
    let is_negative =
        (str.starts_with("\\P") && !str.starts_with("\\P{^")) || str.starts_with("\\p{^");
    let name = str[2..].trim_matches(&['{', '}', '^'][..]).replace(&['+', '-'][..], "_");

    if is_negative {
        Some(format!("Replace `{str}` with `[!{name}]`"))
    } else {
        Some(format!("Replace `{str}` with `[{name}]`"))
    }
}
