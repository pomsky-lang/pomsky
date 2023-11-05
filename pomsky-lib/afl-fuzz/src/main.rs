use std::sync::OnceLock;

use arbitrary::{Arbitrary, Unstructured};
use pomsky::{features::PomskyFeatures, options::RegexFlavor, Expr};
use regex_test::{Outcome, RegexTest};

fn get_test() -> &'static RegexTest {
    static TEST: OnceLock<RegexTest> = OnceLock::new();
    TEST.get_or_init(RegexTest::new)
}

#[allow(unused)]
macro_rules! debug {
    (type) => {
        ()
    };
    (init: $input:expr, $options:expr) => {
        ()
    };
    ($file:expr $(, $s:expr)* $(,)?) => {};
}

#[cfg(FALSE)] // comment this attribute to enable debugging while using `cargo afl tmin`
macro_rules! debug {
    (type) => { std::fs::File };
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

type DebugFile = debug!(type);

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, compile_options)) = Arbitrary::arbitrary(&mut u) {
            #[allow(clippy::let_unit_value)]
            let mut _f = debug!(init: input, compile_options);

            let result = Expr::parse_and_compile(input, compile_options);

            if let (Some(regex), _warnings, _tests) = result {
                debug!(_f, " compiled;");

                let features = compile_options.allowed_features;

                // the first check is just to make it less likely to run into regex's nesting
                // limit; the second check is because regex-test doesn't work with empty inputs;
                // the third check is to ignore compile errors produced by raw regexes, which
                // pomsky doesn't validate
                if regex.len() < 2048
                    && !regex.is_empty()
                    && features == { features }.regexes(false)
                {
                    debug!(_f, " check");
                    check(&regex, features, compile_options.flavor, _f);
                } else {
                    debug!(_f, " skipped (too long or `regex` feature enabled)\n");
                }
            } else {
                debug!(_f, " returned error\n");
            }
        }
    });
}

fn check(regex: &str, features: PomskyFeatures, flavor: RegexFlavor, mut _f: DebugFile) {
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
        RegexFlavor::Pcre if features == { features }.lookbehind(false) => test.test_pcre(regex),
        RegexFlavor::DotNet => test.test_dotnet(regex),
        _ => Outcome::Success,
    };
    if let Outcome::Error(e) = outcome {
        if flavor == RegexFlavor::Rust
            && e.trim().ends_with("error: empty character classes are not allowed")
        {
            // This is on my radar, but more difficult to fix!
            return;
        }
        debug!(_f, " {regex:?} ({flavor:?}) failed:\n{e}");
        panic!("Regex {regex:?} is invalid in the {flavor:?} flavor:\n{e}");
    }
}
