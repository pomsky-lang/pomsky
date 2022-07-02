use std::time::Duration;

use criterion::{black_box, AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};
use pomsky::{
    features::PomskyFeatures,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    Expr,
};

const STRINGS: &str = r#"'hello' "world" 'this is great!' "I absolutely love it!" '"'"#;

const PROPERTIES: &str = "
[Adlam] [Adlm] [Alphabetic] ![InBasic_Latin]
[Tibetan] ![Tibt] [!Uppercase_Letter] [Z] ![!Zl] ![cntrl] [ascii_digit] [.] [w]
";

const GROUPS: &str = r#"
( ( ( ( ( ( ( () ) ) ( ( ( ( ( ( ('hello') ) ) ) ) ) ) ) ) ) ( ( ( ( ( () ) ) 'world') ) ) ) )
"#;

const CAPT_GROUPS: &str = r#"
:test(:foo(:this_is_a_really_long_capturing_group_name() :bar() :() :baz('test' :(:(:()):())):()))
"#;

const CLASSES: &str = r#"
['a' '0'-'9' 'a'-'z' Greek !InBasic_Latin U+09-U+0D U+10FFFF '"' "'"]
"#;

const REPETITIONS: &str = r#"
[w]{4}{3}{7,}{1,2}* [w]+ greedy [w]* lazy{500}
"#;

const SPECIAL: &str = r#"
% !% <% %> Grapheme !>> 'this is' | 'a test'
"#;

const MODES: &str = r#"
enable lazy; disable lazy; enable lazy; disable lazy; enable lazy;
(disable lazy; enable lazy; 'w'+)
"#;

static SAMPLES: &[(&str, &str)] = &[
    ("strings", STRINGS),
    ("properties", PROPERTIES),
    ("groups", GROUPS),
    ("capturing groups", CAPT_GROUPS),
    ("classes", CLASSES),
    ("repetitions", REPETITIONS),
    ("special", SPECIAL),
    ("modes", MODES),
];

pub fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| {
            b.iter(|| Expr::parse(black_box(sample), Default::default()).unwrap())
        });
    }
}

pub fn compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| {
            let (expr, _warnings) = Expr::parse(black_box(sample), Default::default()).unwrap();
            b.iter(|| black_box(&expr).compile(ruby()).unwrap())
        });
    }
}

pub fn range(c: &mut Criterion) {
    let mut group = c.benchmark_group("range");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for size in 1..=15 {
        group.throughput(Throughput::Elements(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let max = "3458709621".repeat(((size + 9) / 10) as usize);
            let max = &max[..size as usize];
            let input = format!("range '0'-'{max}'");
            let (expr, _warnings) = Expr::parse(
                black_box(&input),
                ParseOptions { max_range_size: 100, allowed_features: PomskyFeatures::default() },
            )
            .unwrap();

            b.iter(|| black_box(&expr).compile(Default::default()).unwrap())
        });
    }
}

fn ruby() -> CompileOptions {
    CompileOptions { flavor: RegexFlavor::Ruby }
}

pub fn benches(c: &mut Criterion) {
    parse(c);
    compile(c);
    range(c);
}

fn main() {
    let mut c = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .warm_up_time(Duration::from_secs(1))
        .configure_from_args();

    benches(&mut c);
    c.final_summary();
}
