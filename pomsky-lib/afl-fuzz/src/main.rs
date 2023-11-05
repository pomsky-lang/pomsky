use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::Path;
use std::{env, sync::OnceLock};

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
    let mut f = if let Ok("1") = env::var("FUZZ_LOG").as_deref() {
        let file = OpenOptions::new().create(true).append(true).open("./log.txt").unwrap();
        Some(file)
    } else {
        None
    };
    let mut ef = Some(OpenOptions::new().create(true).append(true).open("./errors.txt").unwrap());
    let f = &mut f;
    let ef = &mut ef;

    let ignored_errors = parse_ignored_errors();

    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, compile_options)) = Arbitrary::arbitrary(&mut u) {
            let _: &str = input;
            let input: String = input.chars().fold(String::new(), |mut acc, c| match c {
                // increase likelihood of generating these key words and important sequences by chance
                'à' => acc + " Codepoint ",
                'á' => acc + " Grapheme ",
                'â' => acc + " Start ",
                'ã' => acc + " End ",
                'ä' => acc + " lazy ",
                'å' => acc + " greedy ",
                'æ' => acc + " enable ",
                'ç' => acc + " disable ",
                'è' => acc + " unicode ",
                'é' => acc + " test {",
                'ê' => acc + " match ",
                'ë' => acc + " reject ",
                'ì' => acc + " in ",
                'í' => acc + " as ",
                'î' => acc + " if ",
                'ï' => acc + " else ",
                'ð' => acc + " regex ",
                'ñ' => acc + " recursion ",
                'ò' => acc + " range ",
                'ó' => acc + " base ",
                'ô' => acc + " let ",
                'õ' => acc + " U+1FEFF ",
                'ö' => acc + ":bla(",
                'ø' => acc + "::bla ",
                'ù' => acc + "<< ",
                'ú' => acc + ">> ",
                'û' => acc + "'test'",
                'ü' => acc + "atomic",
                'ý' => acc + " U+FEFF ",
                // 'þ' => acc + "",
                // 'ÿ' => acc + "",
                _ => {
                    acc.push(c);
                    acc
                }
            });

            debug!(f, "\n{:?} -- {:?}\n", input, compile_options);

            let result = Expr::parse_and_compile(&input, compile_options);

            if let (Some(regex), _warnings, _tests) = result {
                debug!(f, " compiled;");

                let features = compile_options.allowed_features;

                // the first check is just to make it less likely to run into regex's nesting
                // limit; the second check is because regex-test doesn't work with empty inputs;
                // the third check is to ignore compile errors produced by raw regexes, which
                // pomsky doesn't validate
                if regex.len() < 2048
                    && !regex.is_empty()
                    && features == { features }.regexes(false)
                {
                    debug!(f, " check");
                    check(&regex, &ignored_errors, compile_options.flavor, f, ef);
                } else {
                    debug!(f, " SKIPPED (too long or `regex` feature enabled)");
                }
            } else {
                debug!(f, " returned error");
            }
        }
    });
}

fn check(
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

        debug!(ef, "{flavor:?}|{regex:?}|{e}\n");
        debug!(f, " {regex:?} ({flavor:?}) failed:\n{e}");
        panic!("Regex {regex:?} is invalid in the {flavor:?} flavor:\n{e}");
    }
}

fn parse_ignored_errors() -> HashMap<RegexFlavor, RegexSet> {
    let ignored_err_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ignored_errors.txt");
    let ignored_errors = fs::read_to_string(ignored_err_path).unwrap();
    let ignored_errors = ignored_errors
        .lines()
        .filter_map(|line| {
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
