# rulex-macro

⚠️ **DEPRECATED** ⚠️ Use the `pomsky-macro` crate instead. Rulex was
[renamed to pomsky](https://pomsky-lang.org/blog/renaming-rulex/).

This Rust procedural macro allows converting a [rulex expression](../README.md) to a regex
string literal at compile time:

```rust
use rulex_macro::rulex;

const REGEX: &str = rulex!("foo" | "bar"+ greedy);
```

This string can then used with the `regex` crate:

```rust
let my_regex = regex::Regex::new(REGEX).unwrap();
```

## Diagnostics

Errors from rulex are shown at compile time and are highlighted in your IDE. You can improve the
diagnostics by enabling the `diagnostics` feature, which requires Rust Nightly.

## Regex flavor

If you want to use a regex flavor _other than Rust_, you can specify it after a hashtag:

```rust
const REGEX: &str = rulex!(
    #flavor = Pcre
    >> "test" %
);
```

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
