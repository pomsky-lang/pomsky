use std::time::Duration;

use criterion::{black_box, AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};
use pomsky::{
    diagnose::Diagnostic,
    options::{CompileOptions, RegexFlavor},
    Expr,
};

const STRINGS: &str = include_str!("./files/strings.pom");
const PROPERTIES: &str = include_str!("./files/properties.pom");
const GROUPS: &str = include_str!("./files/groups.pom");
const CAPT_GROUPS: &str = include_str!("./files/capt_groups.pom");
const CLASSES: &str = include_str!("./files/classes.pom");
const REPETITIONS: &str = include_str!("./files/repetitions.pom");
const SPECIAL: &str = include_str!("./files/special.pom");
const MODES: &str = include_str!("./files/modes.pom");

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

const VERSION_POMSKY: &str = include_str!("./files/version.pom");
const VERSION_MELODY: &str = include_str!("./files/version.melody");

pub fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| {
            b.iter(|| {
                let (expr, _warnings) = Expr::parse(black_box(sample));
                expr.unwrap()
            })
        });
    }
}

pub fn compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| {
            let (expr, _warnings) = Expr::parse(black_box(sample));
            let expr = expr.unwrap();
            b.iter(|| unwrap_compiled(black_box(&expr).compile(black_box(sample), ruby())))
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
            let (expr, _warnings) = Expr::parse(black_box(&input));
            let expr = expr.unwrap();

            b.iter(|| {
                let options = CompileOptions { max_range_size: 100, ..Default::default() };
                unwrap_compiled(black_box(&expr).compile(&input, options))
            })
        });
    }
}

fn unwrap_compiled(compiled: (Option<String>, Vec<Diagnostic>)) -> String {
    match compiled {
        (Some(s), _) => s,
        (None, _) => panic!("compilation failed"),
    }
}

pub fn competition(c: &mut Criterion) {
    let mut group = c.benchmark_group("version number");

    group.throughput(Throughput::Bytes(VERSION_POMSKY.len() as u64));
    group.bench_function("rulex", |b| {
        b.iter(|| {
            let (expr, _warnings) =
                Expr::parse_and_compile(black_box(VERSION_POMSKY), Default::default());
            expr.unwrap()
        })
    });

    group.throughput(Throughput::Bytes(VERSION_MELODY.len() as u64));
    group.bench_function("melody", |b| {
        b.iter(|| melody_compiler::compiler(VERSION_MELODY).unwrap())
    });
}

fn ruby() -> CompileOptions {
    CompileOptions { flavor: RegexFlavor::Ruby, ..Default::default() }
}

pub fn benches(c: &mut Criterion) {
    parse(c);
    compile(c);
    range(c);
    competition(c);
}

fn main() {
    let mut c = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .warm_up_time(Duration::from_secs(1))
        .configure_from_args();

    benches(&mut c);
    c.final_summary();
}
