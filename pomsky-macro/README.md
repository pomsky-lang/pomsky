# pomsky-macro

This Rust procedural macro allows converting a [pomsky expression](../README.md) to a regex
string literal at compile time:

```rust
use pomsky_macro::pomsky;

const REGEX: &str = pomsky!("foo" | "bar"+ greedy);
```

This string can then used with the `regex` crate:

```rust
let my_regex = regex::Regex::new(REGEX).unwrap();
```

## Diagnostics

Errors from pomsky are shown at compile time and are highlighted in your IDE. You can improve the
diagnostics by enabling the `diagnostics` feature, which requires Rust Nightly.

## Regex flavor

If you want to use a regex flavor _other than Rust_, you can specify it after a hashtag:

```rust
const REGEX: &str = pomsky!(
    #flavor = Pcre
    >> "test" %
);
```

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
