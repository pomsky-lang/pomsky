# Pomsky WASM module for node

WASM module of [Pomsky](https://pomsky-lang.org).

## Usage

```js
import { compile } from '@pomsky-lang/compiler-node'

const { output, diagnostics } = compile(`^ C* '.' C* $`, 'js')
```

This _should_ just work in Node.js. To use Pomsky in the browser, use [unplugin](https://www.npmjs.com/package/@pomsky-lang/unplugin) if you're using a bundler, or [compiler-web](https://www.npmjs.com/package/@pomsky-lang/compiler-web) if you want to compile Pomsky expressions on the client.

Don't forget to check if `output === null`, which means that compilation failed, and you have to look at the diagnostics. Even when the expression compiled successfully, `diagnostics` may contain useful warnings.

## License

Dual-licensed under the [MIT license][mit-license] or the [Apache 2.0 license][apache-2-license].

[mit-license]: https://opensource.org/licenses/MIT
[apache-2-license]: https://opensource.org/licenses/Apache-2.0
