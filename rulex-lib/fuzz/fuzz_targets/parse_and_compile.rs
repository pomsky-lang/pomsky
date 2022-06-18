#![no_main]
use libfuzzer_sys::fuzz_target;

use rulex::{
    options::{CompileOptions, ParseOptions},
    Rulex,
};

fuzz_target!(|data: (&str, ParseOptions, CompileOptions)| {
    let (input, parse_options, compile_options) = data;
    let _ = Rulex::parse_and_compile(input, parse_options, compile_options);
});
