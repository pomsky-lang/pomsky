use pomsky::{diagnose::Diagnostic, options::CompileOptions, Expr};

const STRINGS: &str = include_str!("./files/strings.pom");
const PROPERTIES: &str = include_str!("./files/properties.pom");
const GROUPS: &str = include_str!("./files/groups.pom");
const CAPT_GROUPS: &str = include_str!("./files/capt_groups.pom");
const CLASSES: &str = include_str!("./files/classes.pom");
const REPETITIONS: &str = include_str!("./files/repetitions.pom");
const SPECIAL: &str = include_str!("./files/special.pom");
const MODES: &str = include_str!("./files/modes.pom");

const VERSION_POMSKY: &str = include_str!("./files/version.pom");
const VERSION_MELODY: &str = include_str!("./files/version.melody");

macro_rules! items {
    ($($name:ident: $item:ident),* $(,)?) => {
        $( group_item!($name, $item); )*
    };
}

#[divan::bench_group]
mod parse {
    use divan::{black_box, counter::BytesCount, Bencher};
    use pomsky::Expr;

    macro_rules! group_item {
        ($name:ident, $item:ident) => {
            #[divan::bench]
            fn $name(bencher: Bencher) {
                bencher
                    .with_inputs(|| super::$item)
                    .input_counter(|s| BytesCount::new(s.len()))
                    .bench_refs(|sample| {
                        let (expr, _warnings) = Expr::parse(black_box(sample));
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
    );
}

#[divan::bench_group]
mod compile {
    use divan::{black_box, Bencher};
    use pomsky::{
        options::{CompileOptions, RegexFlavor},
        Expr,
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
                        let (expr, _warnings) = Expr::parse(black_box(sample));
                        (expr.unwrap(), sample)
                    })
                    .bench_refs(|(expr, sample)| {
                        let compiled = black_box(&expr).compile(black_box(sample), ruby());
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
    );
}

#[divan::bench(consts = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13])]
pub fn range<const N: usize>(bencher: divan::Bencher) {
    let max = "3458709621".repeat((N + 9) / 10);
    let max = &max[..N];
    let input = format!("range '0'-'{max}'");

    bencher
        .with_inputs(|| {
            let (expr, _warnings) = Expr::parse(divan::black_box(&input));
            expr.unwrap()
        })
        .bench_refs(|expr| {
            let options = CompileOptions { max_range_size: 100, ..Default::default() };
            let compiled = divan::black_box(&expr).compile(&input, options);
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
    use divan::{black_box, counter::BytesCount};
    use pomsky::Expr;

    #[divan::bench(name = "pomsky (version number)")]
    pub fn pomsky(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| super::VERSION_POMSKY)
            .input_counter(|s| BytesCount::new(s.len()))
            .bench_refs(|sample| {
                let (expr, _, _) = Expr::parse_and_compile(black_box(sample), Default::default());
                expr.unwrap()
            });
    }

    #[divan::bench(name = "melody (version number)")]
    pub fn melody(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| super::VERSION_MELODY)
            .input_counter(|s| BytesCount::new(s.len()))
            .bench_refs(|sample| melody_compiler::compiler(black_box(sample)).unwrap());
    }
}

fn main() {
    divan::main();
}
