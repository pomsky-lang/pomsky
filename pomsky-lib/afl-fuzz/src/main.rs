use arbitrary::{Arbitrary, Unstructured};
use pomsky::{options::RegexFlavor, Expr};

#[allow(unused)]
macro_rules! debug {
    (init: $input:expr, $options:expr) => {
        ()
    };
    ($file:expr $(, $s:expr)* $(,)?) => {};
}

#[cfg(FALSE)] // uncomment to enable debugging while using `cargo afl tmin`
macro_rules! debug {
    (init: $input:expr, $options:expr) => {{
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open("./log.txt").unwrap();
        use std::io::Write as _;
        write!(file, "\n{:?} -- {:?}\n", $input, $options).unwrap();
        file
    }};
    ($file:expr $(, $s:expr)* $(,)?) => {{
        use std::io::Write as _;
        write!($file $(, $s)*).unwrap();
    }};
}

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, compile_options)) = Arbitrary::arbitrary(&mut u) {
            let mut _f = debug!(init: input, compile_options);

            let result = Expr::parse_and_compile(input, compile_options);

            if let (Some(regex), _warnings) = result {
                debug!(_f, " compiled;");

                let features = compile_options.allowed_features;

                // the first check is just to make it less likely to run into regex's nesting
                // limit; the second check is to ignore compile errors produced by raw
                // regexes, which pomsky doesn't validate
                if regex.len() < 2048 && features == { features }.regexes(false) {
                    debug!(_f, " check");
                    match compile_options.flavor {
                        RegexFlavor::Rust => {
                            debug!(_f, " rust...");
                            regex::Regex::new(&regex).unwrap();
                            debug!(_f, " done!\n");
                        }
                        // Pomsky currently doesn't check if loobehind has repetitions for PCRE
                        RegexFlavor::Pcre if features == { features }.lookbehind(false) => {
                            debug!(_f, " pcre...");
                            pcre2::bytes::RegexBuilder::new().utf(true).build(&regex).unwrap();
                            debug!(_f, " done!\n");
                        }
                        _ => {
                            debug!(_f, " skipped (other flavor)\n");
                        }
                    }
                }
            } else {
                debug!(_f, " returned error: {:?}\n", result.1);
            }
        }
    });
}
