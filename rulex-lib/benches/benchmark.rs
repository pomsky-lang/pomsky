use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rulex::Rulex;

const PARSE_INPUT: &str = r#"
<.> <w> <s> <cp> <alpha> <.> <w> <s> <cp> <alpha>
([ab] | [+-*/%] | '[' | ']' | '0'-'9' | 'f'-'o' | 'j'-'z')
(<.> | <w> | :() | % "tests" % | % "test" %)
((((((((((((((((((((((((('a')))))))))))))))))))))))))
:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:(:("test")))))))))))))))))))))))))
"hello"{2}{3}{4}{5}{6}?{1,4}{2,9} greedy {,3}* greedy
"#;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| Rulex::parse(black_box(PARSE_INPUT), Default::default()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
