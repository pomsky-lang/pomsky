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
                ParseErrorMsg::Dollar => Some("Use `End` to match the end of the string".into()),
                ParseErrorMsg::SpecialGroup => get_special_group_help(slice),
                ParseErrorMsg::Backslash => get_backslash_help(slice),
                ParseErrorMsg::BackslashU4 => get_backslash_help_u4(slice),
                ParseErrorMsg::BackslashX2 => get_backslash_help_x2(slice),
                ParseErrorMsg::BackslashUnicode => get_backslash_help_unicode(slice),
                ParseErrorMsg::BackslashK => get_backslash_help_k(slice),
                ParseErrorMsg::UnclosedString => None,
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
                    https://rulex-rs.github.io/docs/language-tour/ranges/"
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
                https://rulex-rs.github.io/docs/language-tour/variables/"
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
            msg: warning.to_string(),
            source_code: source_code.into(),
            help: None,
            span,
        }
    }
}

fn get_special_group_help(str: &str) -> Option<String> {
    assert!(str.starts_with("(?"));
    let str = &str[2..];
    let mut iter = str.chars();

    Some(match (iter.next(), iter.next()) {
        (Some(':'), _) => "Non-capturing groups are just parentheses: `(...)`. \
            Capturing groups use the `:(...)` syntax."
            .into(),
        (Some('P'), Some('<')) => {
            let str = &str[2..];
            let rest = str.trim_start_matches(char::is_alphanumeric);
            let name = &str[..str.len() - rest.len()];
            format!(
                "Named capturing groups use the `:name(...)` syntax. Try `:{name}(...)` instead"
            )
        }
        (Some('>'), _) => "Atomic capturing groups are not supported".into(),
        (Some('|'), _) => "Branch reset groups are not supported".into(),
        (Some('('), _) => "Conditionals are not supported".into(),
        (Some('='), _) => "Lookahead uses the `>>` syntax. \
            For example, `>> 'bob'` matches if the position is followed by bob."
            .into(),
        (Some('!'), _) => "Negative lookahead uses the `!>>` syntax. \
            For example, `!>> 'bob'` matches if the position is not followed by bob."
            .into(),
        (Some('<'), Some('=')) => "Lookbehind uses the `<<` syntax. \
            For example, `<< 'bob'` matches if the position is preceded with bob."
            .into(),
        (Some('<'), Some('!')) => "Negative lookbehind uses the `!<<` syntax. \
            For example, `!<< 'bob'` matches if the position is not preceded with bob."
            .into(),
        (Some('<'), _) => {
            let str = &str[1..];
            let rest = str.trim_start_matches(char::is_alphanumeric);
            let name = &str[..str.len() - rest.len()];
            format!(
                "Named capturing groups use the `:name(...)` syntax. Try `:{name}(...)` instead"
            )
        }
        _ => return None,
    })
}

fn get_backslash_help(str: &str) -> Option<String> {
    assert!(str.starts_with('\\'));
    let str = &str[1..];
    let mut iter = str.chars();

    Some(match iter.next() {
        Some('b') => "Replace `\\b` with `%` to match a word boundary".into(),
        Some('B') => "Replace `\\B` with `!%` to match a place without a word boundary".into(),
        Some('A') => "Replace `\\A` with `<%` to match the start of the string".into(),
        Some('z') => "Replace `\\z` with `%>` to match the end of the string".into(),
        Some('Z') => "\\Z is not supported. Use `%>` to match the end of the string. \
            Note, however, that `%>` doesn't match the position before the final newline."
            .into(),
        Some('N') => "Replace `\\N` with `[.]`".into(),
        Some('X') => "Replace `\\X` with `Grapheme`".into(),
        Some('R') => "Replace `\\R` with `(r n | v)`".into(),
        Some('D') => "Replace `\\D` with `[!d]`".into(),
        Some('W') => "Replace `\\W` with `[!w]`".into(),
        Some('S') => "Replace `\\S` with `[!s]`".into(),
        Some(c @ ('a' | 'e' | 'f' | 'n' | 'r' | 't' | 'h' | 'v' | 'd' | 'w' | 's')) => {
            format!("Replace `\\{c}` with `[{c}]`")
        }
        Some(c @ '0'..='9') => format!("Replace `\\{c}` with `::{c}`"),
        _ => return None,
    })
}

fn get_backslash_help_u4(str: &str) -> Option<String> {
    assert!(str.starts_with("\\u"));
    let hex = &str[2..6];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_x2(str: &str) -> Option<String> {
    assert!(str.starts_with("\\x"));
    let hex = &str[2..4];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_unicode(str: &str) -> Option<String> {
    let hex_len = str[3..].chars().take_while(|c| c.is_ascii_hexdigit()).count();
    let hex = &str[3..3 + hex_len];
    Some(format!("Try `U+{hex}` instead"))
}

fn get_backslash_help_k(str: &str) -> Option<String> {
    assert!(str.starts_with("\\k<"));
    let name_len = str[3..].chars().take_while(|&c| c != '>').count();
    let name = &str[3..3 + name_len];
    Some(format!("Replace `\\k<{name}>` with `::{name}`"))
}
