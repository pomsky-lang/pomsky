use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write as _,
    path::Path,
    sync::OnceLock,
};

use arbitrary::{Arbitrary, Unstructured};
use pomsky::{options::RegexFlavor, Expr};
use regex::RegexSet;
use regex_test::{Outcome, RegexTest};

fn get_test() -> &'static RegexTest {
    static TEST: OnceLock<RegexTest> = OnceLock::new();
    TEST.get_or_init(RegexTest::new)
}

macro_rules! debug {
    ($file:expr $(, $s:expr)* $(,)?) => {
        if let Some(f) = $file {
            write!(f $(, $s)*).unwrap();
        }
    };
}

fn main() {
    let mut f = matches!(std::env::var("FUZZ_LOG").as_deref(), Ok("1"))
        .then(|| OpenOptions::new().create(true).append(true).open("./log.txt").unwrap());
    let mut ef = Some(OpenOptions::new().create(true).append(true).open("./errors.txt").unwrap());
    let f = &mut f;
    let ef = &mut ef;

    let ignored_errors = parse_ignored_errors();

    afl::fuzz(true, |data| {
        let mut u = Unstructured::new(data);
        let Ok((compile_options, input)) = Arbitrary::arbitrary(&mut u) else { return };

        debug!(f, "\n\n{:#?}\n{:?} -- {:?}\n", input, input, compile_options);

        let result = Expr::compile(&input, "", compile_options);

        if let (Some(regex), _warnings) = result {
            debug!(f, " compiled;");

            let features = compile_options.allowed_features;

            // - make it less likely to run into regex's nesting limit
            // - regex-test doesn't work with empty inputs
            // - don't validate raw regexes, which may be invalid
            if regex.len() < 2048 && !regex.is_empty() && features == { features }.regexes(false) {
                debug!(f, " check");
                check(input, &regex, &ignored_errors, compile_options.flavor, f, ef);
            } else {
                debug!(f, " SKIPPED (too long or `regex` feature enabled)");
            }
        } else {
            debug!(f, " returned error");
        }
    });
}

fn check(
    expr: Expr,
    regex: &str,
    ignored_errors: &HashMap<RegexFlavor, RegexSet>,
    flavor: RegexFlavor,
    f: &mut Option<File>,
    ef: &mut Option<File>,
) {
    let test = get_test();
    let outcome = match flavor {
        RegexFlavor::Java => test.test_java(regex),
        RegexFlavor::JavaScript => test.test_js(regex),
        RegexFlavor::Ruby => test.test_ruby(regex),
        RegexFlavor::Rust => test.test_rust(regex),
        RegexFlavor::Python => test.test_python(regex),
        RegexFlavor::Pcre => test.test_pcre(regex),
        RegexFlavor::DotNet => test.test_dotnet(regex),
        _ => Outcome::Success,
    };
    if let Outcome::Error(e) = outcome {
        let e = e.trim();
        if let Some(ignored_errors) = ignored_errors.get(&flavor) {
            if ignored_errors.is_match(e) {
                debug!(f, " {regex:?} ({flavor:?}) ERROR IGNORED: {e}");
                return;
            }
        }

        debug!(ef, "\n{expr:?}\n{flavor:?}|{regex:?}|{e}\n");
        debug!(f, " {regex:?} ({flavor:?}) failed:\n{e}");
        panic!("Regex {regex:?} is invalid in the {flavor:?} flavor:\n{e}");
    }
}

fn parse_ignored_errors() -> HashMap<RegexFlavor, RegexSet> {
    let ignored_err_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ignored_errors.txt");
    let ignored_errors = std::fs::read_to_string(ignored_err_path).unwrap();
    let ignored_errors = ignored_errors
        .lines()
        .filter_map(|line| {
            if line.starts_with('#') || line.is_empty() {
                return None;
            }
            Some(match line.split_once('|') {
                Some(("JS" | "JavaScript", err)) => (RegexFlavor::JavaScript, err),
                Some(("Java", err)) => (RegexFlavor::Java, err),
                Some(("Py" | "Python", err)) => (RegexFlavor::Python, err),
                Some(("PCRE", err)) => (RegexFlavor::Pcre, err),
                Some((".NET" | "DotNet", err)) => (RegexFlavor::DotNet, err),
                Some(("Ruby", err)) => (RegexFlavor::Ruby, err),
                Some(("Rust", err)) => (RegexFlavor::Rust, err),
                Some((invalid, _)) => panic!("Flavor {invalid} is invalid"),
                None => return None,
            })
        })
        .fold(HashMap::new(), |mut acc, (flavor, err)| {
            let v: &mut Vec<&str> = acc.entry(flavor).or_default();
            v.push(err);
            acc
        });

    ignored_errors.into_iter().map(|(k, v)| (k, RegexSet::new(v).unwrap())).collect()
}
