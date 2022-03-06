# rulex-macro

This Rust procedural macro allows converting a [rulex expression](../README.md) to a regex
string literal at compile time:

```rust
use rulex_macro::rulex;

const REGEX: &str = rulex!(r#" 'foo' | 'bar'+ greedy "#);
```

Errors from rulex are shown at compile time, too, and are highlighted in your IDE.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
