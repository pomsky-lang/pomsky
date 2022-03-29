# Installation instructions

## CLI

This CLI allows you to compile rulex expressions to regexes in the command line.

### Use pre-built binaries

Binaries are available for Windows, Linux and macOS. Download them from the
[releases page](https://github.com/Aloso/rulex/releases).

### Install from source

This requires that a recent Rust toolchain is installed. Instructions for how to install Rust can be
found [here](https://www.rust-lang.org/tools/install).

Install the CLI with

```sh
cargo install rulex-bin
```

## Rust macro

If you want to write a rulex directly in your Rust source code, the
[rulex-macro](https://crates.io/crates/rulex-macro) got you covered. Add this to your `Cargo.toml`:

```toml
rulex-macro = "0.3"
```

Then you can import and use it with

```rs
use rulex_macro::rulex;

const MY_REGEX: &str = rulex!(["great!"] | "great!");
```

Documentation can be [found here](https://docs.rs/rulex-macro/latest/rulex_macro/).
