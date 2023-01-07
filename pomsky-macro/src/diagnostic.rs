use std::{fmt::Write, ops::Range};

use pomsky::diagnose::Diagnostic;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub(crate) fn fmt(diagnostic: Diagnostic, _: &Group) -> String {
    let mut buf = String::new();
    buf.push_str("error: ");
    buf.push_str(&diagnostic.msg);
    buf.push('\n');

    if let Some(range) = diagnostic.span.range() {
        let source_code = diagnostic.source_code.as_deref().unwrap_or_default();
        let slice = &source_code[range.clone()];
        let Range { start, end } = range;

        let before = source_code[..start].lines().next_back().unwrap_or_default();
        let after = source_code[end..].lines().next().unwrap_or_default();

        let line_number = source_code[..start].lines().count().max(1);
        let line_number_len = (line_number as f32).log10().floor() as usize + 1;
        let before_len = before.chars().count();
        let arrow_len = slice.chars().count().max(1);

        write!(
            &mut buf,
            "\
{space:line_number_len$} |
{line_number} | {before}{slice}{after}
{space:line_number_len$} | {space:before_len$}{space:^<arrow_len$}",
            space = ""
        )
        .unwrap();
        buf.push('\n');
    }

    if let Some(help) = diagnostic.help {
        buf.push_str("help: ");
        buf.push_str(&help);
        buf.push('\n');
    }

    buf
}

pub(crate) fn error(s: &str, start: Span, end: Span) -> TokenStream {
    let group = vec![respan(Literal::string(s), Span::call_site())].into_iter().collect();

    vec![
        respan(Ident::new("compile_error", start), start),
        respan(Punct::new('!', Spacing::Alone), Span::call_site()),
        respan(Group::new(Delimiter::Brace, group), end),
    ]
    .into_iter()
    .collect()
}

fn respan<T: Into<TokenTree>>(t: T, span: Span) -> TokenTree {
    let mut t = t.into();
    t.set_span(span);
    t
}
