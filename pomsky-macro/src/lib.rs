#![cfg_attr(feature = "diagnostics", feature(proc_macro_span))]

extern crate proc_macro;

use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Literal, Span, TokenStream, TokenTree};

use pomsky::{
    options::{CompileOptions, RegexFlavor},
    Expr,
};

mod diagnostic;

#[proc_macro]
pub fn pomsky(items: TokenStream) -> TokenStream {
    let group = Group::new(Delimiter::None, items);
    let global_span = group.span();

    match pomsky_impl(group.stream().into_iter()) {
        Ok(lit) => TokenTree::Literal(lit).into(),
        Err(Error { msg, span }) => {
            let span = span.unwrap_or(global_span);
            diagnostic::error(&msg, span, span)
        }
    }
}

struct Error {
    msg: String,
    span: Option<Span>,
}

impl Error {
    fn new(msg: String, span: Span) -> Self {
        Error { msg, span: Some(span) }
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
    ($e1:expr) => {
        return Err(Error::from_msg($e1))
    };
    ($e1:expr, $e2:expr) => {
        return Err(Error::new($e1, $e2))
    };
}

fn expect(
    iter: &mut Peekable<impl Iterator<Item = TokenTree>>,
    pred: fn(&TokenTree) -> bool,
    error_msg: impl Into<String>,
) -> Result<(), Error> {
    match iter.peek() {
        Some(tt) if pred(tt) => {
            iter.next();
            Ok(())
        }
        Some(tt) => bail!(error_msg.into(), tt.span()),
        None => bail!(error_msg.into()),
    }
}

fn pomsky_impl(items: impl Iterator<Item = TokenTree>) -> Result<Literal, Error> {
    let mut iter = items.peekable();

    let found_hashtag =
        expect(&mut iter, |t| matches!(t, TokenTree::Punct(p) if p.as_char() == '#'), "");

    let flavor = if found_hashtag.is_ok() {
        expect(
            &mut iter,
            |t| matches!(t, TokenTree::Ident(id) if &id.to_string() == "flavor"),
            "expected `flavor`",
        )?;
        expect(
            &mut iter,
            |t| matches!(t, TokenTree::Punct(p) if p.as_char() == '='),
            "expected `=`",
        )?;

        get_flavor(iter.next())?
    } else {
        RegexFlavor::Rust
    };

    let group = Group::new(Delimiter::None, iter.collect());

    #[cfg(not(feature = "diagnostics"))]
    let (span, input) = (group.span(), group.to_string());

    #[cfg(feature = "diagnostics")]
    let (span, input) = {
        if let (Some(first), Some(last)) =
            (group.stream().into_iter().next(), group.stream().into_iter().last())
        {
            let span = first.span().join(last.span()).unwrap();
            (span, span.source_text().unwrap())
        } else {
            (group.span_close(), String::new())
        }
    };

    let input = input.trim_start_matches("/*«*/").trim_end_matches("/*»*/");

    match Expr::parse_and_compile(input, CompileOptions { flavor, ..Default::default() }) {
        (Some(compiled), _warnings) => Ok(Literal::string(&compiled)),

        (None, errors) => {
            let errors = errors.into_iter().map(|d| diagnostic::fmt(d, &group)).collect::<Vec<_>>();
            bail!(errors.join("\n\n"), span)
        }
    }
}

fn get_flavor(item: Option<TokenTree>) -> Result<RegexFlavor, Error> {
    Ok(match item {
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
