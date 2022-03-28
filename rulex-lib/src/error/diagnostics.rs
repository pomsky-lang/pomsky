use crate::{parse::ParseErrorMsg, repetition::RepetitionError, span::Span};

use super::{compile_error::CompileErrorKind, CompileError, ParseError, ParseErrorKind};

#[cfg_attr(feature = "miette", derive(Debug, thiserror::Error))]
#[cfg_attr(feature = "miette", error("{}", .msg))]
#[non_exhaustive]
pub struct Diagnostic {
    pub msg: String,
    pub code: Option<String>,
    pub source_code: String,
    pub help: Option<String>,
    pub span: Span,
}

#[cfg(feature = "miette")]
impl From<Span> for miette::SourceSpan {
    fn from(s: Span) -> Self {
        let std::ops::Range { start, end } = s.range();
        miette::SourceSpan::new(start.into(), (end - start).into())
    }
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
        let std::ops::Range { start, end } = self.span.range();
        Some(Box::new(
            [miette::LabeledSpan::new(Some("error occurred here".into()), start, end - start)]
                .into_iter(),
        ))
    }
}

impl Diagnostic {
    pub fn from_parse_error(error: ParseError, source_code: &str) -> Self {
        let range = error.span.map(Span::range).unwrap_or(0..source_code.len());
        let slice = &source_code[range.clone()];
        let mut span = Span::from(range);

        let help = match error.kind {
            ParseErrorKind::LexErrorWithMessage(msg) => match msg {
                ParseErrorMsg::SpecialGroup => get_special_group_help(slice),
                ParseErrorMsg::Backslash => get_backslash_help(slice),
                ParseErrorMsg::BackslashU4 => get_backslash_help_u4(slice),
                ParseErrorMsg::BackslashX2 => get_backslash_help_x2(slice),
                ParseErrorMsg::BackslashUnicode => get_backslash_help_unicode(slice),
                ParseErrorMsg::BackslashK => get_backslash_help_k(slice),
                ParseErrorMsg::Caret => None,
                ParseErrorMsg::Dollar => None,
                ParseErrorMsg::UnclosedString => None,
            },
            ParseErrorKind::Repetition(RepetitionError::QuestionMarkAfterRepetition) => Some(
                "If you meant to make the repetition lazy, append the `lazy` keyword instead.\n\
                If this is intentional, consider adding parentheses around the inner repetition."
                    .into(),
            ),
            ParseErrorKind::InvalidEscapeInStringAt(offset) => {
                let span_start = span.range().start;
                span = Span::new(span_start + offset - 1, span_start + offset + 1);
                None
            }
            _ => None,
        };

        Diagnostic {
            code: None,
            msg: error.kind.to_string(),
            source_code: source_code.into(),
            help,
            span,
        }
    }

    pub fn from_compile_error(
        CompileError { kind, span }: CompileError,
        source_code: &str,
    ) -> Self {
        match kind {
            CompileErrorKind::ParseError(kind) => {
                Diagnostic::from_parse_error(ParseError { kind, span }, source_code)
            }
            _ => {
                let range = span.map(Span::range).unwrap_or(0..source_code.len());
                let span = Span::from(range);

                Diagnostic {
                    code: None,
                    msg: kind.to_string(),
                    source_code: source_code.into(),
                    help: None,
                    span,
                }
            }
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
        (Some('('), _) => "Branch reset groups are not supported".into(),
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
