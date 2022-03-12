use std::ops::Range;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use rulex::error::Diagnostic;

pub(crate) fn fmt(diagnostic: Diagnostic, _: Group) -> String {
    let mut buf = String::new();
    buf.push_str("error: ");
    buf.push_str(&diagnostic.msg);
    buf.push('\n');

    let range = diagnostic.span.range();
    let slice = &diagnostic.source_code[range.clone()];
    let Range { start, end } = range;

    let before = diagnostic.source_code[..start].lines().next_back().unwrap_or_default();
    let after = diagnostic.source_code[end..].lines().next().unwrap_or_default();

    let line_number = diagnostic.source_code[..start].lines().count().max(1);
    let line_number_len = (line_number as f32).log10().floor() as usize + 1;
    let before_len = before.chars().count();
    let arrow_len = slice.chars().count().max(1);

    buf.push_str(&format!(
        "\
{space:line_number_len$} |
{line_number} | {before}{slice}{after}
{space:line_number_len$} | {space:before_len$}{space:^<arrow_len$}",
        space = ""
    ));
    buf.push('\n');

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
