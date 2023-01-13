#![no_main]
use libfuzzer_sys::fuzz_target;

use once_cell::sync::OnceCell;
use pomsky::{
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    Expr,
};
use regex_test::{sync::RegexTest, Outcome};

fn get_test() -> &'static RegexTest {
    static TEST: OnceCell<RegexTest> = OnceCell::new();
    TEST.get_or_init(RegexTest::new)
}

fuzz_target!(|data: (&str, CompileOptions)| {
    let (input, compile_options) = data;
    let result = Expr::parse_and_compile(input, compile_options);

    if let (Some(regex), _warnings) = result {
        let features = compile_options.allowed_features;

        // the first check is just to make it less likely to run into regex's nesting
        // limit; the second check is because regex-test doesn't work with empty inputs;
        // the third check is to ignore compile errors produced by raw regexes, which
        // pomsky doesn't validate
        if regex.len() < 2048 && !regex.is_empty() && features == { features }.regexes(false) {
            check(&regex, features, compile_options.flavor);
        }
    }
});

fn check(regex: &str, features: PomskyFeatures, flavor: RegexFlavor) {
    let test = get_test();
    let outcome = match flavor {
        // Pomsky currently doesn't check if loobehind has repetitions, so we don't check some
        // regexes
        RegexFlavor::Java if features == { features }.lookbehind(false) => test.test_java(regex),
        RegexFlavor::JavaScript => test.test_js(regex),
        RegexFlavor::Ruby => test.test_ruby(regex),
        RegexFlavor::Rust => test.test_rust(regex),
        RegexFlavor::Python if features == { features }.lookbehind(false) => {
            test.test_python(regex)
        }
        RegexFlavor::Pcre if features == { features }.lookbehind(false) => test.test_pcre(&regex),
        _ => Outcome::Success,
    };
    if let Outcome::Error(e) = outcome {
        if flavor == RegexFlavor::Rust
            && e.trim().ends_with("error: empty character classes are not allowed")
        {
            // This is on my radar, but more difficult to fix!
            return;
        }
        panic!("Regex {regex:?} is invalid in the {flavor:?} flavor:\n{e}");
    }
}

#[cfg(any())]
fn failing_to_compile_in_pcre() {
    // this runs into PCRE2's limitation that lookbehinds can't contain repetitions
    // the same limitation exists in R
    "<<$*"
    // compiled:
    // (?<=(?:$)*)
}
