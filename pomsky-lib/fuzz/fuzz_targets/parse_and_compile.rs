#![no_main]
use libfuzzer_sys::fuzz_target;

use pomsky::{
    options::{CompileOptions, ParseOptions},
    Expr,
};

fuzz_target!(|data: (&str, ParseOptions, CompileOptions)| {
    let (input, parse_options, compile_options) = data;
    let _ = Expr::parse_and_compile(input, parse_options, compile_options);
});
