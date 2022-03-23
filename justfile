# run rulex (debug mode)
run S *args:
    cargo run -- '{{S}}' {{args}}

# install rulex
install:
    cargo install --path=rulex-bin

# benchmark rulex
bench *flags:
    cargo bench -p benchmark -- {{flags}}

# benchmark rulex with the plotters backend
bench-plotters *flags:
    cargo bench -p benchmark -- --plotting-backend plotters {{flags}}

# test rulex
test:
    cargo test

# test rulex, include ignored tests
test-ignored:
    cargo test -- --ignored

# fuzz rulex ranges
fuzz-ranges *flags:
    cargo test --test it -- --fuzz-ranges {{flags}}
