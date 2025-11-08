use std::ops::Range;

use js_sys::{Array, Object, Reflect};
use pomsky::{
    diagnose::Diagnostic,
    options::{CompileOptions, RegexFlavor},
    test::{Test, TestCapture, TestCase, TestCaseMatch, TestCaseMatchAll, TestCaseReject},
    Expr, Span,
};
use wasm_bindgen::prelude::*;

mod utils;

#[wasm_bindgen(module = "/js/mod.js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "PomskyDiagnostic")]
    pub type PomskyDiagnostic;

    #[wasm_bindgen(constructor)]
    fn new(
        severity: &str,
        kind: &str,
        code: Option<String>,
        message: String,
        help: Option<String>,
        range: &[usize],
    ) -> PomskyDiagnostic;

    #[wasm_bindgen(typescript_type = "PomskyError")]
    pub type PomskyError;

    #[wasm_bindgen(constructor)]
    fn new(message: String) -> PomskyError;

    #[wasm_bindgen(typescript_type = "PomskyResult")]
    pub type PomskyResult;

    #[wasm_bindgen(constructor)]
    fn new(
        output: Option<String>,
        warnings: Vec<PomskyDiagnostic>,
        tests: Option<Array>,
    ) -> PomskyResult;
}

#[wasm_bindgen(typescript_custom_section)]
const ITEXT_STYLE: &str = r#"
interface PomskyDiagnostic {
    severity: "error" | "warning";
    kind: string;
    code: string;
    message: string;
    help: string | null;
    range: [number, number];
}

interface PomskyError extends Error {}

interface PomskyResult {
    output: string | null;
    diagnostics: PomskyDiagnostic[];
    tests: PomskyTest[] | null;
}

type PomskyTest =
    | { match: PomskyTestMatch }
    | { matchAll: PomskyTestMatchAll }
    | { reject: PomskyTestReject };

interface PomskyTestMatch {
    literal: string;
    range: [number, number];
    captures: PomskyTestCapture[];
}

interface PomskyTestMatchAll {
    literal: string;
    range: [number, number];
    matches: PomskyTestMatch[];
}

interface PomskyTestReject {
    literal: string;
    range: [number, number];
    asSubstring: boolean;
}

interface PomskyTestCapture {
    ident: string | number;
    identRange: [number, number];
    literal: string;
    range: [number, number];
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
        .ok_or_else(|| PomskyError::new(format!("Unknown regex flavor `{flavor}`")))?;

    let (result, diagnostics, tests) = Expr::parse_and_compile(
        input,
        CompileOptions { flavor, max_range_size: 12, ..Default::default() },
    );

    Ok(PomskyResult::new(
        result,
        diagnostics.into_iter().map(|d| convert_diagnostic(input, d)).collect(),
        tests_to_js(tests),
    ))
}

fn tests_to_js(tests: Vec<Test>) -> Option<Array> {
    fn range(span: Span) -> Array {
        let range = span.range().unwrap_or(0..0);
        Array::from_iter([
            JsValue::from_f64(range.start as f64),
            JsValue::from_f64(range.end as f64),
        ])
    }

    fn match_to_js(match_: TestCaseMatch) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &"literal".into(), &match_.literal.content.into()).unwrap();
        Reflect::set(&obj, &"range".into(), &range(match_.literal.span)).unwrap();

        let captures = match_.captures.into_iter().map(capture_to_js);
        Reflect::set(&obj, &"captures".into(), &Array::from_iter(captures)).unwrap();
        obj
    }

    fn match_all_to_js(match_all: TestCaseMatchAll) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &"literal".into(), &match_all.literal.content.into()).unwrap();
        Reflect::set(&obj, &"range".into(), &range(match_all.literal.span)).unwrap();

        let matches = match_all.matches.into_iter().map(match_to_js);
        Reflect::set(&obj, &"matches".into(), &Array::from_iter(matches)).unwrap();
        obj
    }

    fn reject_to_js(reject: TestCaseReject) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &"literal".into(), &reject.literal.content.into()).unwrap();
        Reflect::set(&obj, &"range".into(), &range(reject.literal.span)).unwrap();
        Reflect::set(&obj, &"asSubstring".into(), &reject.as_substring.into()).unwrap();
        obj
    }

    fn capture_to_js(capture: TestCapture) -> Object {
        let obj = Object::new();

        let ident = match capture.ident {
            pomsky::test::CaptureIdent::Name(name) => name.into(),
            pomsky::test::CaptureIdent::Index(idx) => idx.into(),
        };
        Reflect::set(&obj, &"ident".into(), &ident).unwrap();
        Reflect::set(&obj, &"identRange".into(), &range(capture.ident_span)).unwrap();

        Reflect::set(&obj, &"literal".into(), &capture.literal.content.into()).unwrap();
        Reflect::set(&obj, &"range".into(), &range(capture.literal.span)).unwrap();
        obj
    }

    if tests.is_empty() {
        return None;
    }

    let tests_obj = Array::new();
    for test in tests {
        for case in test.cases {
            let case_obj = Object::new();
            match case {
                TestCase::Match(m) => {
                    Reflect::set(&case_obj, &"match".into(), &match_to_js(m)).unwrap();
                }
                TestCase::MatchAll(m) => {
                    Reflect::set(&case_obj, &"matchAll".into(), &match_all_to_js(m)).unwrap();
                }
                TestCase::Reject(r) => {
                    Reflect::set(&case_obj, &"reject".into(), &reject_to_js(r)).unwrap();
                }
            }
            tests_obj.push(&case_obj);
        }
    }
    Some(tests_obj)
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

    PomskyDiagnostic::new(
        d.severity.into(),
        d.kind.into(),
        d.code.map(|c| c.to_string()),
        d.msg,
        d.help,
        &[start16, end16],
    )
}

fn split_in_three(input: &str, cut1: usize, cut2: usize) -> (&str, &str, &str) {
    let (rest, suffix) = input.split_at(cut2);
    let (prefix, middle) = rest.split_at(cut1);
    (prefix, middle, suffix)
}
