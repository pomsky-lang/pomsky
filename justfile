set positional-arguments

coverage_flags := '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'

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

coverage:
    cargo clean
    RUSTFLAGS="{{ coverage_flags }}" RUSTDOCFLAGS="{{ coverage_flags }}" CARGO_INCREMENTAL=0 cargo +nightly test
    zip -0 cov.zip $(find . -name "pomsky*.gc*" -print)
    grcov cov.zip -s . -t lcov --llvm --ignore-not-existing --ignore "/*" -o lcov.info

# test pomsky
test *args:
    cargo test "$@"

test-it *args:
    cargo test --test it --all-features -- "$@"

# fuzz pomsky ranges
fuzz-ranges *flags:
    cargo test --test it -- --fuzz-ranges {{flags}}
