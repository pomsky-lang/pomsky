# Contributing to Pomsky

There are multiple ways to contribute to Pomsky. The easiest way is to use Pomsky and report any bugs or issues you find in the [issue tracker](https://github.com/pomsky-lang/pomsky/issues) or propose new features. If you'd like to contribute code, please read on. You can also [sponsor me](https://github.com/sponsors/Aloso) to support Pomsky's development.

## Conduct

When participating in discussions, please follow the [code of conduct](./CODE_OF_CONDUCT.md). To make sure this remains a friendly and safe space for everyone, all official communication venues are moderated. This currently includes

- [Issue tracker](https://github.com/pomsky-lang/pomsky/issues)
- [GitHub discussions](https://github.com/pomsky-lang/pomsky/discussions)

If you notice a comment that violates the Code of Conduct, is offending, or violates any law, please [reach out](mailto:ludwig.stecher@gmx.de). If you have questions or are unsure if a comment is allowed, feel free to reach out as well.

## Security

If you have a concern that may warrant a security advisory, please [contact me directly](mailto:ludwig.stecher@gmx.de). Don't file an issue: Most of the time, security concerns should only be made public once a fix is available.

## Contributing Code

Pomsky is written in Rust. To contribute code, you should be comfortable writing Rust. If you have Rust-related questions, I can recommend the [Rust discord](https://discord.gg/rust-lang) to quickly get help.

### Finding something to contribute

Issues that might be suitable to start contributing are labelled [good first issue](https://github.com/pomsky-lang/pomsky/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22). Once you found an issue you want to fix, please add a comment to tell us that you're working on it. If you need help, I can give you mentoring instructions, don't hesitate to ask.

If what you want to implement doesn't have an issue yet, please open an issue first. This doesn't apply to code quality and documentation improvements.

### Prerequisites

To develop Pomsky, you need the latest Rust compiler and cargo; I strongly recommend using [rustup](https://rustup.rs/) to manage versions and components. I also recommend using [rust-analyzer](https://rust-analyzer.github.io/) for IDE functionality ([IntelliJ-Rust](https://intellij-rust.github.io/) if you prefer JetBrains IDEs).

Furthermore, you need the following components:

- [rustfmt](https://github.com/rust-lang/rustfmt) for formatting your code
- [clippy](https://doc.rust-lang.org/clippy/) for better lints

Configure your IDE to run rustfmt on save and display clippy diagnostics.

### Other tools

To be able to run tests, you need the following tools installed:

- `python` (Python 3)
- `java` (Java 8+)
- `node` (Node.js)

If you want to run the fuzzers, you also need `cargo-afl` and `cargo-fuzz`.

### Crate structure

Pomsky is divided into multiple components (crates):

- `pomsky-syntax` contains the parser. It has no dependencies and is not configurable; a Pomsky file is parsed the same way regardless of the regex flavor or enabled features.
- `pomsky-lib` is the compiler and depends on `pomsky-syntax`. This crate does most of the heavy lifting.
- `pomsky-bin` is the command-line tool. It parses CLI arguments, calls `pomsky-lib` appropriately, and displays the result.
- `pomsky-wasm` is a Node.js module that embeds `pomsky-lib` as a WASM module and provides a JavaScript API.
- `helptext` is a utility crate for the CLI to generate help messages.
- `benchmark` contains code to monitor Pomsky's performance and compare it with Melody.
- `pomsky-fuzz` (nested in the `pomsky-lib` folder) is a fuzzer using libfuzz.
- `afl-fuzz` (nested in the `pomsky-lib` folder) is a fuzzer using AFL.

### Testing

Pomsky uses integration tests for most things. These live in the `pomsky-lib/tests/testcases` folder and are run with a custom test harness in the `pomsky-lib/tests/it` folder. Running these tests requires Python, Java and Node.js to be installed on your system.

You can run all tests with `cargo test`, or only integration tests with `cargo test --test it --all-features`. If you use [just](https://github.com/casey/just), you can instead run `just test-it`.

Make sure to add tests for code you contribute. Test cases look like this:

```
POMSKY EXPRESSION
-----
COMPILED REGEX
```

You can optionally set the flavor with `#! flavor=FLAVOR`, for example:

```
#! flavor=java
'foo' 'bar'
-----
foobar
```

You don't have to type all of this yourself, only the Pomsky expression. Then run the integration test with the `--bless` flag to generate the rest: `just test-it --bless`, or `cargo test --test it --all-features -- --bless`.

Test cases can also check if an input produces a certain error, using `#! expect=error`. Error test cases can also be generated with `--bless`.

## License

Pomsky is dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the [Apache 2.0 license](https://opensource.org/licenses/Apache-2.0). Any code you contribute will be licensed in the same way. If we ever decide to change the licenses, all contributors will be asked for their permission.
