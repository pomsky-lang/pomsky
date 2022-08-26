use std::time::Duration;

use criterion::{black_box, AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};
use pomsky::{
    options::{CompileOptions, RegexFlavor},
    Expr,
};

const STRINGS: &str = include_str!("./files/strings.rulex");
const PROPERTIES: &str = include_str!("./files/properties.rulex");
const GROUPS: &str = include_str!("./files/groups.rulex");
const CAPT_GROUPS: &str = include_str!("./files/capt_groups.rulex");
const CLASSES: &str = include_str!("./files/classes.rulex");
const REPETITIONS: &str = include_str!("./files/repetitions.rulex");
const SPECIAL: &str = include_str!("./files/special.rulex");
const MODES: &str = include_str!("./files/modes.rulex");

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

const VERSION_RULEX: &str = include_str!("./files/version.rulex");
const VERSION_MELODY: &str = include_str!("./files/version.melody");

pub fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| b.iter(|| Expr::parse(black_box(sample)).unwrap()));
    }
}

pub fn compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    for &(sample_name, sample) in SAMPLES {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_function(sample_name, |b| {
            let (expr, _warnings) = Expr::parse(black_box(sample)).unwrap();
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
            let (expr, _warnings) = Expr::parse(black_box(&input)).unwrap();

            b.iter(|| {
                black_box(&expr)
                    .compile(CompileOptions { max_range_size: 100, ..Default::default() })
                    .unwrap()
            })
        });
    }
}

pub fn competition(c: &mut Criterion) {
    let mut group = c.benchmark_group("version number");

    group.throughput(Throughput::Bytes(VERSION_RULEX.len() as u64));
    group.bench_function("rulex", |b| {
        b.iter(|| Expr::parse_and_compile(black_box(VERSION_RULEX), Default::default()).unwrap())
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
