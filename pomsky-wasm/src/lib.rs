use std::ops::Range;

use pomsky::{
    error::Diagnostic,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    Expr,
};
use wasm_bindgen::prelude::*;

mod utils;

#[wasm_bindgen(module = "/js/mod.js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "PomskyDiagnostic")]
    pub type PomskyDiagnostic;

    #[wasm_bindgen(constructor)]
    fn new(message: String, help: Option<String>, range: &[usize]) -> PomskyDiagnostic;

    #[wasm_bindgen(typescript_type = "PomskyError")]
    pub type PomskyError;

    #[wasm_bindgen(constructor)]
    fn new(message: String, diagnostics: Option<Vec<PomskyDiagnostic>>) -> PomskyError;

    #[wasm_bindgen(typescript_type = "PomskyResult")]
    pub type PomskyResult;

    #[wasm_bindgen(constructor)]
    fn new(output: String, warnings: Vec<PomskyDiagnostic>) -> PomskyResult;
}

#[wasm_bindgen(typescript_custom_section)]
const ITEXT_STYLE: &str = r#"
interface PomskyDiagnostic {
    message: string;
    help: string | null;
    range: [number, number];
}

interface PomskyError extends Error {
    diagnostics: PomskyDiagnostic[];
}

interface PomskyResult {
    output: string;
    warnings: PomskyDiagnostic[];
}
"#;

#[wasm_bindgen]
/// Compile a pomsky expression. It returns a `PomskyResult`, or throws
/// a `PomskyError` if the input is invalid.
///
/// `flavor` must be one of the following values:
///  - "javascript" or "js"
///  - "java"
///  - "dotnet" or ".net"
///  - "pcre"
///  - "python"
///  - "ruby"
///  - "rust"
pub fn compile(input: &str, flavor: &str) -> Result<PomskyResult, PomskyError> {
    utils::set_panic_hook();

    let flavor = parse_flavor(flavor)
        .ok_or_else(|| PomskyError::new(format!("Unknown regex flavor `{flavor}`"), None))?;

    let result = Expr::parse_and_compile(
        input,
        ParseOptions { max_range_size: 12, ..Default::default() },
        CompileOptions { flavor },
    );

    match result {
        Ok((output, warnings)) => Ok(PomskyResult::new(
            output,
            warnings
                .into_iter()
                .map(|warning| convert_diagnostic(input, Diagnostic::from_warning(warning, input)))
                .collect(),
        )),
        Err(err) => {
            let diagnostics = err
                .diagnostics(input)
                .into_iter()
                .map(|d| convert_diagnostic(input, d))
                .collect::<Vec<_>>();

            Err(PomskyError::new("Failed to compile pomsky expression".into(), Some(diagnostics)))
        }
    }
}

fn parse_flavor(flavor: &str) -> Option<RegexFlavor> {
    Some(match flavor {
        "javascript" | "js" => RegexFlavor::JavaScript,
        "java" => RegexFlavor::Java,
        "dotnet" | ".net" => RegexFlavor::DotNet,
        "pcre" => RegexFlavor::Pcre,
        "python" => RegexFlavor::Python,
        "ruby" => RegexFlavor::Ruby,
        "rust" => RegexFlavor::Rust,
        _ => return None,
    })
}

fn convert_diagnostic(input: &str, d: Diagnostic) -> PomskyDiagnostic {
    let Range { start, end } = d.span.range().unwrap_or_default();
    let (prefix, content, _) = split_in_three(input, start, end);
    let start16 = prefix.encode_utf16().count();
    let end16 = start16 + content.encode_utf16().count();

    PomskyDiagnostic::new(d.msg, d.help, &[start16, end16])
}

fn split_in_three(input: &str, cut1: usize, cut2: usize) -> (&str, &str, &str) {
    let (rest, suffix) = input.split_at(cut2);
    let (prefix, middle) = rest.split_at(cut1);
    (prefix, middle, suffix)
}
