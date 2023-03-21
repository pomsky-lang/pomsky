# Pomsky WASM module for web

WASM module of [Pomsky](https://pomsky-lang.org).

## Usage

```js
import { compile } from '@pomsky-lang/compiler-web'

const { output, diagnostics } = compile(`^ C* '.' C* $`, 'js')
```

If this doesn't work with your bundler, try initializing the module explicitly:

```js
import init, { compile } from 'pomsky-wasm'

await init()
const { output, diagnostics } = compile(`^ C* '.' C* $`, 'js')
```

Don't forget to check if `output === null`, which means that compilation failed, and you have to look at the diagnostics. Even when the expression compiled successfully, `diagnostics` may contain useful warnings.

### With vite

If you're using vite, you also need to update your vite config like this:

```diff
  import { defineConfig } from 'vite'

  export default defineConfig(({ mode }) => ({
+   optimizeDeps: {
+     exclude: mode === 'production' ? [] : ['pomsky-wasm'],
+   },
  }))
```

## License

Dual-licensed under the [MIT license][mit-license] or the [Apache 2.0 license][apache-2-license].

[mit-license]: https://opensource.org/licenses/MIT
[apache-2-license]: https://opensource.org/licenses/Apache-2.0
