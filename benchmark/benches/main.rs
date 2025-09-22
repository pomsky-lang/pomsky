use pomsky::{Expr, diagnose::Diagnostic, options::CompileOptions};

const STRINGS: &str = include_str!("./files/strings.pomsky");
const PROPERTIES: &str = include_str!("./files/properties.pomsky");
const GROUPS: &str = include_str!("./files/groups.pomsky");
const CAPT_GROUPS: &str = include_str!("./files/capt_groups.pomsky");
const CLASSES: &str = include_str!("./files/classes.pomsky");
const REPETITIONS: &str = include_str!("./files/repetitions.pomsky");
const SPECIAL: &str = include_str!("./files/special.pomsky");
const MODES: &str = include_str!("./files/modes.pomsky");
const EMAIL: &str = include_str!("./files/email.pomsky");

const VERSION_POMSKY: &str = include_str!("./files/version.pomsky");
const VERSION_MELODY: &str = include_str!("./files/version.melody");

macro_rules! items {
    ($($name:ident: $item:ident),* $(,)?) => {
        $( group_item!($name, $item); )*
    };
}

#[divan::bench_group]
mod parse {
    use divan::{Bencher, counter::BytesCount};
    use pomsky::Expr;

    macro_rules! group_item {
        ($name:ident, $item:ident) => {
            #[divan::bench]
            fn $name(bencher: Bencher) {
                bencher
                    .with_inputs(|| super::$item)
                    .input_counter(|s| BytesCount::new(s.len()))
                    .bench_refs(|sample| {
                        let (expr, _warnings) = Expr::parse(sample);
                        expr.unwrap()
                    });
            }
        };
    }

    items!(
        strings: STRINGS,
        properties: PROPERTIES,
        groups: GROUPS,
        capturing_groups: CAPT_GROUPS,
        classes: CLASSES,
        repetitions: REPETITIONS,
        special: SPECIAL,
        modes: MODES,
        email: EMAIL,
    );
}

#[divan::bench_group]
mod compile {
    use divan::Bencher;
    use pomsky::{
        Expr,
        options::{CompileOptions, RegexFlavor},
    };

    fn ruby() -> CompileOptions {
        CompileOptions { flavor: RegexFlavor::Ruby, ..Default::default() }
    }

    macro_rules! group_item {
        ($name:ident, $item:ident) => {
            #[divan::bench]
            fn $name(bencher: Bencher) {
                bencher
                    .with_inputs(|| {
                        let sample = super::$item;
                        let (expr, _warnings) = Expr::parse(sample);
                        (expr.unwrap(), sample)
                    })
                    .bench_refs(|(expr, sample)| {
                        let compiled = expr.compile(sample, ruby());
                        super::unwrap_compiled(compiled)
                    });
            }
        };
    }

    items!(
        strings: STRINGS,
        properties: PROPERTIES,
        groups: GROUPS,
        capturing_groups: CAPT_GROUPS,
        classes: CLASSES,
        repetitions: REPETITIONS,
        special: SPECIAL,
        modes: MODES,
        email: EMAIL,
    );
}

#[divan::bench(args = 1..=13)]
pub fn range(bencher: divan::Bencher, n: usize) {
    let max = "3458709621".repeat(n.div_ceil(10));
    let max = &max[..n];
    let input = format!("range '0'-'{max}'");

    bencher
        .with_inputs(|| {
            let (expr, _warnings) = Expr::parse(&input);
            expr.unwrap()
        })
        .bench_refs(|expr| {
            let options = CompileOptions { max_range_size: 100, ..Default::default() };
            let compiled = expr.compile(&input, options);
            unwrap_compiled(compiled)
        })
}

fn unwrap_compiled(compiled: (Option<String>, Vec<Diagnostic>)) -> String {
    match compiled {
        (Some(s), _) => s,
        (None, _) => panic!("compilation failed"),
    }
}

#[divan::bench_group]
mod competition {
    use divan::counter::BytesCount;
    use pomsky::Expr;

    #[divan::bench(name = "pomsky (version number)")]
    pub fn pomsky(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| super::VERSION_POMSKY)
            .input_counter(|s| BytesCount::new(s.len()))
            .bench_refs(|sample| {
                let (expr, _, _) = Expr::parse_and_compile(sample, Default::default());
                expr.unwrap()
            });
    }

    #[divan::bench(name = "melody (version number)")]
    pub fn melody(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| super::VERSION_MELODY)
            .input_counter(|s| BytesCount::new(s.len()))
            .bench_refs(|sample| melody_compiler::compiler(sample).unwrap());
    }
}

fn main() {
    divan::main();
}
