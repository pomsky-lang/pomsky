extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use rulex::{
    error::Diagnostic,
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

#[proc_macro]
pub fn rulex(items: TokenStream) -> TokenStream {
    let group = Group::new(Delimiter::None, items);
    let global_span = group.span();

    match rulex_impl(group.stream().into_iter()) {
        Ok(lit) => TokenTree::Literal(lit).into(),
        Err(Error { msg, span }) => {
            let span = span.unwrap_or(global_span);
            let msg = format!("error: {msg}");
            error(&msg, span, span)
        }
    }
}

struct Error {
    msg: String,
    span: Option<Span>,
}

impl Error {
    fn new(msg: String, span: Span) -> Self {
        Error {
            msg,
            span: Some(span),
        }
    }

    fn from_msg(msg: String) -> Self {
        Error { msg, span: None }
    }
}

macro_rules! bail {
    ($l:literal) => {
        return Err(Error::from_msg(format!($l)))
    };
    ($l:literal, $e:expr) => {
        return Err(Error::new(format!($l), $e))
    };
    ($e1:expr, $e2:expr) => {
        return Err(Error::new($e1, $e2))
    };
}

fn rulex_impl(mut items: impl Iterator<Item = TokenTree>) -> Result<Literal, Error> {
    let lit = items
        .next()
        .ok_or_else(|| Error::from_msg("Expected string literal".into()))?;
    let span = lit.span();

    match lit {
        TokenTree::Literal(lit) => {
            let s = lit.to_string();
            if !s.starts_with('r') {
                bail!(
                    r##"Expected raw string literal: `r"..."` or `r#"..."#`"##,
                    span
                );
            }
            let s = s[1..].trim_matches('#');
            let input = s
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or_else(|| Error::new("Expected string literal".into(), span))?;

            let flavor = get_flavor(items)?;

            match Rulex::parse(input, Default::default()) {
                Ok(parsed) => {
                    let options = CompileOptions {
                        flavor,
                        ..Default::default()
                    };
                    match parsed.compile(options) {
                        Ok(compiled) => Ok(Literal::string(&compiled)),
                        Err(e) => bail!(fmt(Diagnostic::from_compile_error(e, input)), span),
                    }
                }
                Err(e) => bail!(fmt(Diagnostic::from_parse_error(e, input)), span),
            }
        }
        TokenTree::Group(x) => bail!("Expected string literal, got group", x.span()),
        TokenTree::Ident(x) => bail!("Expected string literal, got identifier", x.span()),
        TokenTree::Punct(x) => bail!("Expected string literal, got punctuation", x.span()),
    }
}

fn get_flavor(mut items: impl Iterator<Item = TokenTree>) -> Result<RegexFlavor, Error> {
    match items.next() {
        None => return Ok(RegexFlavor::Rust),
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
        Some(tt) => bail!("Unexpected token `{tt}`", tt.span()),
    }

    match items.next() {
        None => return Ok(RegexFlavor::Rust),
        Some(TokenTree::Ident(id)) if &id.to_string() == "flavor" => {}
        Some(tt) => bail!("Expected `flavor =`, got `{tt}`", tt.span()),
    }

    match items.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
        Some(tt) => bail!("Unexpected token `{tt}`", tt.span()),
        None => bail!("Expected `=`"),
    }

    Ok(match items.next() {
        Some(TokenTree::Ident(id)) => match id.to_string().as_str() {
            "DotNet" => RegexFlavor::DotNet,
            "Java" => RegexFlavor::Java,
            "JavaScript" => RegexFlavor::JavaScript,
            "Pcre" => RegexFlavor::Pcre,
            "Python" => RegexFlavor::Python,
            "Ruby" => RegexFlavor::Ruby,
            "Rust" => RegexFlavor::Rust,
            s => bail!(
                "Expected one of: DotNet, Java, JavaScript, Pcre, Python, Ruby, Rust\nGot: {s}",
                id.span()
            ),
        },
        Some(tt) => bail!("Unexpected token `{tt}`", tt.span()),
        None => bail!("Expected identifier"),
    })
}

fn fmt(diagnostic: Diagnostic) -> String {
    let mut buf = String::new();
    buf.push_str("error: ");
    buf.push_str(&diagnostic.msg);
    buf.push('\n');

    let range = diagnostic
        .span
        .range()
        .unwrap_or(0..diagnostic.source_code.len());
    let slice = &diagnostic.source_code[range.clone()];
    let span = rulex::span::Span::from(range);

    let before = diagnostic.source_code[..span.start]
        .lines()
        .next_back()
        .unwrap_or_default();
    let after = diagnostic.source_code[span.end..]
        .lines()
        .next()
        .unwrap_or_default();

    let line_number = diagnostic.source_code[..span.start].lines().count().max(1);
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

fn error(s: &str, start: Span, end: Span) -> TokenStream {
    let group = vec![respan(Literal::string(s), Span::call_site())]
        .into_iter()
        .collect();

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