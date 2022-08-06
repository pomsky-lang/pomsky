# pomsky-wasm

Node.js/WASM module of [pomsky](..).

## Usage

```js
import { compile } from "pomsky-wasm";

const { output } = compile(`^ C* '.' C* $`, "js");
```

### With vite

If you're using vite, you also need to install `vite-plugin-wasm` and update your vite config like this:

```diff
  import { defineConfig } from 'vite'
+ import wasm from 'vite-plugin-wasm'

  export default defineConfig({
    plugins: [
+     wasm()
    ],
  })
```

## License

Dual-licensed under the [MIT license][mit-license] or the [Apache 2.0 license][apache-2-license].

[mit-license]: https://opensource.org/licenses/MIT
[apache-2-license]: https://opensource.org/licenses/Apache-2.0
