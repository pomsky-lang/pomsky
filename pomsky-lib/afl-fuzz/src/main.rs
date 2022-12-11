use arbitrary::{Arbitrary, Unstructured};
use pomsky::{options::RegexFlavor, Expr};

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, compile_options)) = Arbitrary::arbitrary(&mut u) {
            let result = Expr::parse_and_compile(input, compile_options);

            if let Ok((regex, _warnings)) = result {
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
        }
    });
}
