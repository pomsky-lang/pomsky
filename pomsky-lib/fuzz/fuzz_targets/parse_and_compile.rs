#![no_main]
use libfuzzer_sys::fuzz_target;

use pomsky::{options::CompileOptions, Expr};

fuzz_target!(|data: (&str, CompileOptions)| {
    let (input, compile_options) = data;
    let _ = Expr::parse_and_compile(input, compile_options);
});
