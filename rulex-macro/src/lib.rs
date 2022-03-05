extern crate proc_macro;
use proc_macro::{Literal, TokenStream, TokenTree};
use rulex::{
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

#[proc_macro]
pub fn rulex(items: TokenStream) -> TokenStream {
    match rulex_impl(items.into_iter()) {
        Ok(lit) => TokenTree::Literal(lit).into(),
        Err(msg) => {
            let msg = format!("error: {msg}");
            format!("compile_error!({msg:?})").parse().unwrap()
        }
    }
}

fn rulex_impl(mut items: impl Iterator<Item = TokenTree>) -> Result<Literal, String> {
    let lit = items
        .next()
        .ok_or_else(|| "Expected string literal".to_string())?;

    match lit {
        TokenTree::Literal(lit) => {
            let s = lit.to_string();
            if !s.starts_with('r') {
                return Err(r##"Expected a raw string literal: `r"..."` or `r#"..."#`"##.into());
            }
            let s = s[1..].trim_matches('#');
            let input = s
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or_else(|| "Expected string literal".to_string())?;

            let flavor = get_flavor(items)?;

            match Rulex::parse(input, Default::default()) {
                Ok(parsed) => {
                    let options = CompileOptions {
                        flavor,
                        ..Default::default()
                    };
                    match parsed.compile(options) {
                        Ok(compiled) => Ok(Literal::string(&compiled)),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => {
                    let e = e.with_context(input);
                    Err(e.to_string())
                }
            }
        }
        TokenTree::Group(_) => Err("Expected string literal, got group".into()),
        TokenTree::Ident(_) => Err("Expected string literal, got identifier".into()),
        TokenTree::Punct(_) => Err("Expected string literal, got punctuation".into()),
    }
}

fn get_flavor(mut items: impl Iterator<Item = TokenTree>) -> Result<RegexFlavor, String> {
    match items.next() {
        None => return Ok(RegexFlavor::Rust),
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
        Some(tt) => return Err(format!("Unexpected token `{tt}`")),
    }

    match items.next() {
        None => return Ok(RegexFlavor::Rust),
        Some(TokenTree::Ident(id)) if &id.to_string() == "flavor" => {}
        Some(tt) => return Err(format!("Expected `flavor =`, got `{tt}`")),
    }

    match items.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
        Some(tt) => return Err(format!("Unexpected token `{tt}`")),
        None => return Err("Expected `=`".to_string()),
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
            s => {
                return Err(format!(
                    "Expected one of: DotNet, Java, JavaScript, Pcre, Python, Ruby, Rust\nGot: {s}"
                ))
            }
        },
        Some(tt) => return Err(format!("Unexpected token `{tt}`")),
        None => return Err("Expected identifier".to_string()),
    })
}
