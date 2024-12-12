use std::ops::Index;

use pcre2::bytes::Regex as PcreRegex;
use pomsky::{
    diagnose::{Diagnostic, DiagnosticCode, Severity},
    features::PomskyFeatures,
    options::CompileOptions,
    test::{CaptureIdent, TestCase, TestCaseMatch, TestCaseMatchAll, TestCaseReject},
    Expr,
};
use regex::Regex as RustRegex;

pub(crate) fn run_tests(
    parsed: &Expr,
    input: &str,
    options: CompileOptions,
    errors: &mut Vec<Diagnostic>,
) {
    let (Some(pattern), _) = parsed
        .compile(input, CompileOptions { allowed_features: PomskyFeatures::default(), ..options })
    else {
        let msg = "Failed to compile the expression in the PCRE flavor for running tests".into();
        errors.push(Diagnostic::ad_hoc(Severity::Error, None, msg, None));
        return;
    };

    let regex = match options.flavor {
        pomsky::options::RegexFlavor::Pcre => {
            let regex = pcre2::bytes::RegexBuilder::new()
                .jit_if_available(true)
                .ucp(true)
                .utf(true)
                .build(&pattern);

            match regex {
                Ok(regex) => Regex::Pcre(regex),
                Err(e) => {
                    let help = Some(format!("The compiled regex is {pattern:?}"));
                    errors.push(Diagnostic::ad_hoc(Severity::Error, None, e.to_string(), help));
                    return;
                }
            }
        }
        pomsky::options::RegexFlavor::Rust => {
            let regex = RustRegex::new(&pattern);

            match regex {
                Ok(regex) => Regex::Rust(regex),
                Err(e) => {
                    let help = Some(format!("The compiled regex is {pattern:?}"));
                    errors.push(Diagnostic::ad_hoc(Severity::Error, None, e.to_string(), help));
                    return;
                }
            }
        }
        _ => panic!("Unsupported flavor"),
    };

    let tests = parsed.extract_tests_ref();
    for test in tests {
        for test_case in &test.cases {
            match test_case {
                TestCase::Match(m) => check_test_match(&regex, m, errors),
                TestCase::MatchAll(a) => check_all_test_matches(&regex, a, errors),
                TestCase::Reject(r) => check_test_reject(&regex, r, errors),
            }
        }
    }
}

fn check_test_match(regex: &Regex, test_case: &TestCaseMatch, errors: &mut Vec<Diagnostic>) {
    let result = regex.captures(&test_case.literal.content);
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
    test_case: &TestCaseMatchAll,
    errors: &mut Vec<Diagnostic>,
) {
    let captures_iter = regex
        .captures_iter(&test_case.literal.content)
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

fn check_test_reject(regex: &Regex, test_case: &TestCaseReject, errors: &mut Vec<Diagnostic>) {
    let result = regex.captures(&test_case.literal.content);
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

enum Regex {
    Pcre(PcreRegex),
    Rust(RustRegex),
}

impl Regex {
    fn as_str(&self) -> &str {
        match self {
            Regex::Pcre(regex) => regex.as_str(),
            Regex::Rust(regex) => regex.as_str(),
        }
    }

    fn captures<'a>(&'a self, subject: &'a str) -> Result<Option<Captures<'a>>, pcre2::Error> {
        match self {
            Regex::Pcre(regex) => regex.captures(subject.as_bytes()).map(|o| o.map(Captures::Pcre)),
            Regex::Rust(regex) => Ok(regex.captures(subject).map(Captures::Rust)),
        }
    }

    fn captures_iter<'r, 's>(&'r self, subject: &'s str) -> CaptureMatches<'r, 's> {
        match self {
            Regex::Pcre(regex) => CaptureMatches::Pcre(regex.captures_iter(subject.as_bytes())),
            Regex::Rust(regex) => CaptureMatches::Rust(regex.captures_iter(subject)),
        }
    }
}

enum Captures<'a> {
    Pcre(pcre2::bytes::Captures<'a>),
    Rust(regex::Captures<'a>),
}

impl Captures<'_> {
    fn name(&self, name: &str) -> Option<Match<'_>> {
        match self {
            Captures::Pcre(captures) => captures.name(name).map(Match::Pcre),
            Captures::Rust(captures) => captures.name(name).map(Match::Rust),
        }
    }

    fn get(&self, index: usize) -> Option<Match<'_>> {
        match self {
            Captures::Pcre(captures) => captures.get(index).map(Match::Pcre),
            Captures::Rust(captures) => captures.get(index).map(Match::Rust),
        }
    }
}

impl Index<usize> for Captures<'_> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Captures::Pcre(captures) => &captures[index],
            Captures::Rust(captures) => captures[index].as_bytes(),
        }
    }
}

enum Match<'s> {
    Pcre(pcre2::bytes::Match<'s>),
    Rust(regex::Match<'s>),
}

impl Match<'_> {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Match::Pcre(mat) => mat.as_bytes(),
            Match::Rust(mat) => mat.as_str().as_bytes(),
        }
    }
}

enum CaptureMatches<'r, 's> {
    Pcre(pcre2::bytes::CaptureMatches<'r, 's>),
    Rust(regex::CaptureMatches<'r, 's>),
}

impl<'s> Iterator for CaptureMatches<'_, 's> {
    type Item = Result<Captures<'s>, pcre2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CaptureMatches::Pcre(capture_matches) => {
                capture_matches.next().map(|r| r.map(Captures::Pcre))
            }
            CaptureMatches::Rust(capture_matches) => {
                capture_matches.next().map(|c| Ok(Captures::Rust(c)))
            }
        }
    }
}
