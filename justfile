set positional-arguments

# run pomsky (debug mode)
run S *args:
    cargo run -- "$@"

# install pomsky
install:
    cargo install --path=pomsky-bin

# benchmark pomsky
bench *flags:
    cargo bench -p benchmark -- {{flags}}

# benchmark pomsky with the plotters backend
bench-plotters *flags:
    cargo bench -p benchmark -- --plotting-backend plotters {{flags}}

# test pomsky
test:
    cargo test

# test pomsky, include ignored tests
test-ignored:
    cargo test -- --ignored

# fuzz pomsky ranges
fuzz-ranges *flags:
    cargo test --test it -- --fuzz-ranges {{flags}}
