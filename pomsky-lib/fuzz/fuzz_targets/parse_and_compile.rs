#![no_main]
use libfuzzer_sys::fuzz_target;

use pomsky::{
    options::{CompileOptions, RegexFlavor},
    Expr,
};

fuzz_target!(|data: (&str, CompileOptions)| {
    let (input, compile_options) = data;
    let result = Expr::parse_and_compile(input, compile_options);

    if let (Some(regex), _warnings) = result {
        let features = compile_options.allowed_features;

        // the first check is just to make it less likely to run into regex's nesting
        // limit; the second check is to ignore compile errors produced by raw
        // regexes, which pomsky doesn't validate
        if regex.len() < 2048 && features == { features }.regexes(false) {
            match compile_options.flavor {
                RegexFlavor::Rust => {
                    regex::Regex::new(&regex).unwrap();
                }
                // Pomsky currently doesn't check if loobehind has repetitions for PCRE
                RegexFlavor::Pcre if features == { features }.lookbehind(false) => {
                    pcre2::bytes::RegexBuilder::new().utf(true).build(&regex).unwrap();
                }
                _ => {}
            }
        }
    }
});

#[cfg(any())]
fn failing_to_compile_in_pcre() {
    // this runs into PCRE2's limitation that lookbehinds can't contain repetitions
    // the same limitation exists in R
    "<<$*"
    // compiled:
    // (?<=(?:$)*)
}
