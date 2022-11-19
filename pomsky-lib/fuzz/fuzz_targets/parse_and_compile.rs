#![no_main]
use libfuzzer_sys::fuzz_target;

use pomsky::{
    options::{CompileOptions, RegexFlavor},
    Expr,
};

fuzz_target!(|data: (&str, CompileOptions)| {
    let (input, compile_options) = data;
    let result = Expr::parse_and_compile(input, compile_options);

    let features = compile_options.allowed_features;
    if compile_options.flavor == RegexFlavor::Rust && features == { features }.regexes(false) {
        if let Ok((regex, _warnings)) = result {
            regex::Regex::new(&regex).unwrap();
        }
    }
});
