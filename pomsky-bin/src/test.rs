use pcre2::bytes::Regex;
use pomsky::{
    diagnose::{Diagnostic, DiagnosticCode, Severity},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    test::{CaptureIdent, TestCase, TestCaseMatch, TestCaseMatchAll, TestCaseReject},
    Expr,
};

pub(crate) fn run_tests(
    parsed: Expr,
    input: &str,
    options: CompileOptions,
    errors: &mut Vec<Diagnostic>,
) {
    let (Some(pcre_pattern), _) = parsed.compile(
        input,
        CompileOptions {
            flavor: RegexFlavor::Pcre,
            allowed_features: PomskyFeatures::default(),
            ..options
        },
    ) else {
        let msg = "Failed to compile the expression in the PCRE flavor for running tests".into();
        errors.push(Diagnostic::ad_hoc(Severity::Error, None, msg, None));
        return;
    };

    let regex = pcre2::bytes::RegexBuilder::new()
        .jit_if_available(true)
        .ucp(true)
        .utf(true)
        .build(&pcre_pattern);

    let regex = match regex {
        Ok(regex) => regex,
        Err(e) => {
            let help = Some(format!("The compiled regex is {pcre_pattern:?}"));
            errors.push(Diagnostic::ad_hoc(Severity::Error, None, e.to_string(), help));
            return;
        }
    };

    let tests = parsed.extract_tests();
    for test in tests {
        for test_case in test.cases {
            match test_case {
                TestCase::Match(m) => check_test_match(&regex, m, errors),
                TestCase::MatchAll(a) => check_all_test_matches(&regex, a, errors),
                TestCase::Reject(r) => check_test_reject(&regex, r, errors),
            }
        }
    }
}

fn check_test_match(regex: &Regex, test_case: TestCaseMatch, errors: &mut Vec<Diagnostic>) {
    let result = regex.captures(test_case.literal.content.as_bytes());
    match result {
        Ok(Some(captures)) => {
            if captures[0].len() != test_case.literal.content.len() {
                errors.push(Diagnostic::test_failure(
                    test_case.literal.span,
                    DiagnosticCode::TestNoExactMatch,
                    None,
                ));
                return;
            }

            for capture in &test_case.captures {
                let Some(got_capture) = (match &capture.ident {
                    CaptureIdent::Name(name) => captures.name(name),
                    &CaptureIdent::Index(idx) => captures.get(idx as usize),
                }) else {
                    errors.push(Diagnostic::test_failure(
                        capture.ident_span,
                        DiagnosticCode::TestMissingCaptureGroup,
                        None,
                    ));
                    continue;
                };
                if got_capture.as_bytes() != capture.literal.content.as_bytes() {
                    errors.push(Diagnostic::test_failure(
                        capture.literal.span,
                        DiagnosticCode::TestWrongCaptureGroup,
                        Some(&String::from_utf8_lossy(got_capture.as_bytes())),
                    ));
                }
            }
        }
        Ok(None) => {
            errors.push(Diagnostic::test_failure(
                test_case.literal.span,
                DiagnosticCode::TestNoExactMatch,
                None,
            ));
        }
        Err(e) => {
            let help = Some(format!("The compiled regex is {:?}", regex.as_str()));
            errors.push(Diagnostic::ad_hoc(Severity::Error, None, e.to_string(), help));
        }
    }
}

fn check_all_test_matches(
    regex: &Regex,
    test_case: TestCaseMatchAll,
    errors: &mut Vec<Diagnostic>,
) {
    let captures_iter = regex
        .captures_iter(test_case.literal.content.as_bytes())
        .map(Some)
        .chain(std::iter::repeat_with(|| None));
    let expected_iter = test_case.matches.iter().map(Some).chain(std::iter::repeat(None));

    for (i, (captures, expected)) in std::iter::zip(captures_iter, expected_iter).enumerate() {
        match (captures, expected) {
            (None, None) => break,

            (None, Some(expected)) => {
                errors.push(Diagnostic::test_failure(
                    expected.literal.span,
                    DiagnosticCode::TestMissingSubstringMatch,
                    None,
                ));
            }
            (Some(Ok(captures)), None) => {
                if i == 0 {
                    break;
                }
                errors.push(Diagnostic::test_failure(
                    test_case.literal.span,
                    DiagnosticCode::TestUnexpectedSubstringMatch,
                    Some(&String::from_utf8_lossy(&captures[0])),
                ));
            }
            (Some(Ok(captures)), Some(test_case)) => {
                if &captures[0] != test_case.literal.content.as_bytes() {
                    errors.push(Diagnostic::test_failure(
                        test_case.literal.span,
                        DiagnosticCode::TestWrongSubstringMatch,
                        Some(&String::from_utf8_lossy(&captures[0])),
                    ));
                    continue;
                }

                for capture in &test_case.captures {
                    let Some(got_capture) = (match &capture.ident {
                        CaptureIdent::Name(name) => captures.name(name),
                        &CaptureIdent::Index(idx) => captures.get(idx as usize),
                    }) else {
                        errors.push(Diagnostic::test_failure(
                            capture.ident_span,
                            DiagnosticCode::TestMissingCaptureGroup,
                            None,
                        ));
                        continue;
                    };
                    if got_capture.as_bytes() != capture.literal.content.as_bytes() {
                        errors.push(Diagnostic::test_failure(
                            capture.literal.span,
                            DiagnosticCode::TestWrongCaptureGroup,
                            Some(&String::from_utf8_lossy(got_capture.as_bytes())),
                        ));
                    }
                }
            }

            (Some(Err(e)), _) => {
                let help = Some(format!("The compiled regex is {:?}", regex.as_str()));
                errors.push(Diagnostic::ad_hoc(Severity::Error, None, e.to_string(), help));
            }
        }
    }
}

fn check_test_reject(regex: &Regex, test_case: TestCaseReject, errors: &mut Vec<Diagnostic>) {
    let result = regex.captures(test_case.literal.content.as_bytes());
    match result {
        Ok(Some(captures)) => {
            let is_exact = captures[0].len() == test_case.literal.content.len();
            if test_case.as_substring || is_exact {
                let actual_value;
                let (code, actual_value) = if is_exact || !test_case.as_substring {
                    (DiagnosticCode::TestUnexpectedExactMatch, None)
                } else {
                    actual_value = String::from_utf8_lossy(&captures[0]);
                    (DiagnosticCode::TestUnexpectedSubstringMatch, Some(&*actual_value))
                };
                errors.push(Diagnostic::test_failure(test_case.literal.span, code, actual_value));
            }
        }
        Ok(None) => {
            // success
        }
        Err(e) => {
            errors.push(Diagnostic::ad_hoc(
                Severity::Error,
                None,
                e.to_string(),
                Some(format!("The compiled regex is {:?}", regex.as_str())),
            ));
        }
    }
}
