# Pomsky CLI

This CLI allows you to compile [pomsky expressions](https://pomsky-lang.org/) to regexes in the
command line.

## Installation

You can install Pomsky by...

- downloading a pre-built binary
- using the shell or PowerShell installer
- using the Windows msi installer
- installing the `@pomsky-lang/cli` NPM module globally
- installing the `pomsky-bin` AUR package

See the [releases page](https://github.com/pomsky-lang/pomsky/releases) for instructions.

## Build from source

Ensure you have a recent Rust toolchain installed. Instructions for how to install Rust can be
found [here](https://www.rust-lang.org/tools/install). Then run

```sh
cargo install pomsky-bin
```

## Usage

Then you can compile pomsky expressions to a regex flavor of your choice; the default is PCRE.

Run `pomsky --help` for more information.

Pomsky provides nice error messages:

```sh
$ pomsky "'Hello world'* \X+"
Error:
  × Backslash escapes are not supported
   ╭────
 1 │ 'Hello world'* \X+
   ·                ─┬
   ·                 ╰── error occurred here
   ╰────
  help: Replace `\X` with `Grapheme`
```

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
