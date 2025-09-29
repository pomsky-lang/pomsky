# Changelog

All notable changes to the _Pomsky regular expression language_ will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### New

- Support for the RE2 flavor

- Intersection of character sets. For example, `[Letter] & [Latin]` matches a letter that is also in the Latin script.

- Character class prefixes: `gc:` (general category), `sc:` (script), `scx:` (script extension), `blk:` (blocks).
  For example, `[scx:Syriac]` matches all characters with the Syriac script extension.

  - Adds support for script extensions (currently supported in PCRE, JavaScript, and Rust)
  - If the `blk:` prefix is used, `In` must be removed; e.g. `[InPrivate_Use]` becomes `[blk:Private_Use]`
  - Writing the prefix is optional, except for script extensions

- A `pomsky test` subcommand for running unit tests

  - Two supported regex engines for testing: `pcre2` and `rust`
  - The `--test` argument is now deprecated

- Many optimizations (see below)

### Changed

- Change hygiene of `lazy` and `unicode` mode to behave as one would expect.
  Going forward, modes depend on the scope where an expression is defined, not where it is used:

  ```pomsky
  let foo = 'foo'*;  # this repetition is not lazy
  (enable lazy; foo)
  ```

- Increase the maximum length of group names from 32 to 128 characters.
  Group names this long are supported in PCRE2 since version 10.44.

- Produce an error if the contents of a lookbehind assertion are not supported by the regex flavor (Java, Python, PCRE)

- Produce an error if infinite recursion is detected

- Remove the compatibility warning for lookbehind in JavaScript.
  Lookbehind is now widely supported in JavaScript engines.

- Allow all supported boolean Unicode properties in the Java flavor

- Deprecate the `--test` argument; use `pomsky test -p <PATH>` instead

### Optimizations

- De-duplicate and merge character ranges: `['b' 'a'-'f' 'c'-'m']` becomes `[a-m]`

  - Note that this doesn't work with Unicode classes, e.g. `Alphabetic`

- Merge common alternation prefixes: `'do' | 'double' | 'down'` becomes `do(?:uble|wn)??`

  - This only works with string literals and character sets, for now
  - Only adjacent alternatives can be merged to ensure that precedence isn't affected

- Combine single-character alternations into a set: `'a' | 'b' | 'c' | 'f'` becomes `[a-cf]`

- Merge constant nested repetitions: `('a'{3}){4}` becomes `a{12}`

### Bugfixes

- Do not miscompile `[r]`

## [0.11.0] - 2023-11-09

### New

- **Unit tests**: Run tests using PCRE2 during compilation with the `--test=pcre2` flag.

  ```pomsky
  test {
    match '13.17.4.5';
    match '13.17.4.5' as { 1:'13', 2:'17', 3:'4', 4:'5' };
    match '13.17.4.5', '17.3.4.5'
          in 'The IP addresses are 13.17.4.5 and 17.3.4.5.';
    reject '256.0.0.0';
    reject in 'This test contains no IP addresses';
  }

  let octet = range '0'-'255';
  octet ('.' octet){3}
  ```

- **Word start and end** boundaries: `<` (start) and `>` (end) as more specific alternatives to `%`

- **Recursion**: The `recursion` keyword recursively matches the entire regex where supported

- **Arbitrary negation**: Any expression can be negated on the syntax level; negation is applied
  after name resolution, so variables can be negated as well.

### Changed

- Require numeric ranges that allow leading zeroes to be fixed-length. E.g. `range '0024'-'1001'`
  is allowed, but `range '024'-'1001'` is not.

- Forbid single element character ranges like `['a'-'a']`. The upper bound must be strictly higher
  than the lower bound.

- Warn when non-printable character shorthands (e.g. `n` for new line) are used in a character
  range. That's because `[a-f]` is misleading and not very useful.

- Support atomic groups in Python.

- Reserve `call` keyword (not used for now).

### Bugfixes

- Fix incorrect order of capturing groups in .NET (#96)
- Wrap numbered references in a non-capturing group (#97)
- Don't allow numeric ranges with an empty string as lower bound (#98)
- Make sure negated code points above U+FFFF are forbidden in .NET
- Disallow lookahead within lookbehind in Ruby (#101)
- Disallow forward references in Ruby (#102)

## [0.10.0] - 2023-03-21

### New

- Brand new tooling arrived!

  - [VSCode extension](https://marketplace.visualstudio.com/items?itemName=pomsky-lang.pomsky-vscode)
  - [JavaScript plugin for Vite/Webpack/Rollup/ESBuild/ESM](https://github.com/pomsky-lang/unplugin-pomsky/)
  - [Shell completions](https://github.com/pomsky-lang/pomsky/tree/main/completions)

- You can now opt out of Unicode support with `disable unicode;` to produce smaller and more efficient regexes when you know that the input is ASCII. In addition, `%` in JavaScript now requires opting out of Unicode to reflect that word boundaries in JS aren't Unicode aware.

- The [set of Unicode properties](https://pomsky-lang.org/docs/reference/unicode-properties/) was extended to support all general categories, scripts, blocks and boolean properties as of Unicode 15.0. When a property isn't supported by the regex flavor, an error is reported.

- Added `--list shorthands` CLI flag to list all supported Unicode properties and shorthands

### Changed

- Line breaks within string literals are now normalized to `\n`. To match a Windows line break (`\r\n`), use `[r][n]`.

- Code points (e.g. `U+01FF`) can now contain white space before and after the `+`.

- Two keywords were reserved, `U` and `test`. Note that keywords are not available as variable names. `U` is used for code points, `test` is not yet in use.

- Warn when using lookbehind in JavaScript, since it isn't universally supported. You can silence this warning in the CLI with `-W compat=0`.

### Removed

- Leading zeroes in numbers are no longer allowed. Instead of `C{0123}` you now need to write `C{123}`. Some other languages treat numbers with a leading zero as octal, but Pomsky did not, which could cause confusion.

- `[.]` now produces a hard error rather than a warning. It was already deprecated in Pomsky 0.6. Use `.` without the brackets instead.

- Previously, the `+` in code points was optional, so `U+FF` could be written as `UFF`. This was not documented and was ambiguous with variable names, so it was removed.

### Bugfixes

- Fix a bug where parentheses were sometimes omitted in a lookaround assertion
- Fix lexing of numbers and code points containing non-digit/non-hexadecimal code points
- Fix codegen for Unicode scripts in Java
- Fix character classes with a single code point not getting escaped
- Polyfill code points above U+FFFF in .NET, show error when a code point above U+FFFF appears in a character class

## [0.9.0] - 2023-01-14

Join our [Discord](https://discord.gg/uwap2uxMFp) to get help or meet other users and contributors!

If you want to contribute, Pomsky now has a [contributor's guide](./CONTRIBUTING.md) and a [code of conduct](./CODE_OF_CONDUCT.md).

### CLI changes

- Added `--json` flag to work better with other tools and IDEs. Someone's already working on an
  IntelliJ plugin, which will be announced soon!
- Every error and warning now has a diagnostic code such as `P0116`

### Bugfixes

- Don't allow `::0`, which doesn't work
- In Python, don't allow Unicode properties (`\p{Prop}`), since Python doesn't support these
- In Python, don't allow forward references, since Python doesn't support these
- In Python, emit `\UHHHHHHHH` (where `H` is a hex digit) rather than `\u{...}` for large code
  points
- In Java, replace dashes (`-`) in Unicode properties with underscores rather than removing them
- In Java and Ruby, emit `\x{...}` rather than `\u{...}` for large code points
- In Ruby, enforce that expressions with named capturing groups can't contain references to
  unnamed groups
- In Ruby, don't emit `\xHH` (where `HH` are two hex digits) for non-ASCII code points, since Ruby
  treats them as bytes rather than code points, and bytes above `\x7F` may be invalid in UTF-8
- In Ruby, disallow repeated assertions (boundaries or lookarounds)
- In JS, wrap repeated assertions in an extra non-capturing group

### Other

- The test harness and fuzzer were improved to also compile the output of all Python, Java,
  JavaScript and Ruby test cases to detect syntax errors. All flavors except C# are tested in this
  way now, and the fuzzer already found a few bugs.

- Major refactor of error handling, so that reporting multiple errors at once becomes less awkward.
  This will enable us to report better diagnostics in the future.

- GitHub CI improvements

- Website improvements: Replaced PNG with SVG logo, new images and more spacing on the home page, redesigned “examples” section, new color palette, breadcrumbs on documentation pages, added Discord icon, bugfixes

## [0.8.0] - 2022-12-12

**Special announcement**: You can [sponsor me](https://github.com/sponsors/Aloso) now for my work on Pomsky. If you can spare a few dollars or convince your employer to donate, that would really help me to make maintaining Pomsky more sustainable. If I get enough donations, I can invest more time in the development of Pomsky, as there's still a lot of work to do!

Remember that you can also help out by filing issues or contributing 😉

### Language changes

- Added inline regex expressions: Include text that is not transformed or validated. For example:

  ```pomsky
  regex '[\w[^a-f]]'
  ```

  This allows using regex features not yet supported by Pomsky, like nested character classes. Note, however, that Pomsky does not validate inline regexes, so there's no guarantee that the output is correct.

- Added the dot (`.`). It matches anything except line breaks by default, or anything
  _including_ line breaks in multiline mode. [More information](https://www.regular-expressions.info/dot.html)

- Added an optimization pass, which removes redundant groups, simplifies repetitions and deduplicates the contents of character classes.

  Optimizations are useful when making heavy use of variables to write readable code and still get the most efficient output. More optimizations are planned, stay tuned!

- Group names now must be no longer than 32 characters. For example, `:this_is_a_very_very_very_long_name()` is no longer allowed. The reason is that group names this long are unsupported by PCRE, and we're enforcing the same limit everywhere to make Pomsky more consistent across regex flavors.

### CLI changes

- The CLI help interface was overhauled. It is now more informative and beautiful. To get help, type `pomsky -h` for short help, or `pomsky --help` for longer descriptions and additional details.

- It is now possible to specify allowed features in the CLI. This was previously only possible in the Rust library. Use `pomsky --help` for more information.

### Bugfixes

- Fix Unicode script codegen for JavaScript: Pomsky now emits the correct syntax for Unicode scripts in JS.
- Escape `[`, `&` and `|` within character classes. This is required in regex flavors that support nested character classes.
- Fix `\e` being emitted, even though it is not supported in the Rust flavor
- Fix broken feature gates: A few feature gates were defunct and have been fixed.
- Fix position of error report labels with Unicode chars: This was a long-standing bug in [miette] that was [fixed](https://github.com/zkat/miette/pull/202) recently.
- Don't silently ignore exclamation points at the end of a character class.
- Only allow Unicode properties such as `Lowercase` or `Emoji` in regex flavors that support them.

### Other

- Audit dependencies using `cargo-audit` in continuous integration. This means that we'll be made aware of any vulnerability in our dependencies reported to the [RustSec database](https://rustsec.org/).

- Make release binaries auditable: The binaries published on GitHub are now built with `cargo-auditable`. This means that `cargo audit bin /path/to/pomsky` can now scan all included dependencies.

- Remove thiserror dependency from the `pomsky` and `pomsky-syntax` crates, improving compile time.

- Testing improvements:

  - Compile all PCRE and Rust regular expressions produced by integration tests to make sure the output is well-formed. This caught some of the bugs mentioned above! We're currently looking into ways to do the same with the other flavors.

  - Measure test coverage in CI and publish it to coveralls.io. The results are [here](https://coveralls.io/github/pomsky-lang/pomsky?branch=main) (also accessible by clicking on the badge in the README). Note that the measurement is imperfect, so the results may not be accurate.

  - Add end-to-end tests for the CLI and improve test coverage

## [0.7.0] - 2022-09-10

### Added

- `atomic ()` groups, supported in all flavors except Python, Rust and JavaScript.
  Atomic groups discard backtracking information to optimize match performance
  ([more information](https://www.regular-expressions.info/atomic.html)).

- The pomsky library is now published as a WASM module to npm! You can install it with

  ```sh
  $ npm install pomsky-wasm  # yarn add pomsky-wasm
  ```

  How to use it is described [here](https://pomsky-lang.org/docs/get-started/quick-start/#node-module).

### Changed

- The parser was rewritten and is now much faster with fewer dependencies. In my benchmarks,
  it is 3 to 5 times faster than the previous parser.

- The parser was moved to the `pomsky-syntax` crate. You can now directly use it in Rust programs,
  without pulling in the whole compiler.

- The limit for the number of repetitions after an expression has been removed, although the
  limitation was almost impossible to run into in real code.

- Release binaries are now stripped by default, to reduce the binary size.

- The clap argument parser was replaced with the much smaller lexopt. This further reduces the
  binary size.

### Removed

- The `<%`, `%>`, `[cp]` and `[codepoint]` syntax has been removed. Previously it was deprecated
  and issued a warning.

### Fixed

- When compiling the library crate with `miette` support, the `fancy` feature is now enabled
  by default to fix a compilation error.

- A repeated boundary or anchor is now correctly wrapped in parentheses.

## [0.6.0] - 2022-08-03

### Added

- `^` and `$` as aliases for `Start` and `End`

- Leading pipes. This allows you to format expressions more beautifully:

  ```pomsky
  | 'Lorem'
  | :group(
      | 'ipsum'
      | 'dolor'
      | 'sit'
      | 'amet'
    )
  | 'consetetur'
  ```

- Improved diagnostics for typos. When you spell a variable, capturing group or character class
  wrong, Pomsky will suggest the correct spelling:

  ```
  $ pomsky '[Alpabetic]'
  error:
    × Unknown character class `Alpabetic`
    ╭────
  1 │ [Alpabetic]
    ·  ────┬────
    ·      ╰── error occurred here
    ╰────
    help: Perhaps you meant `Alphabetic`
  ```

- Many regex syntax diagnostics were added. Pomsky now recognizes most regex syntax and suggests
  the equivalent Pomsky syntax. For example:

  ```
  $ pomsky '(?<grp> "test")'
  error:
    × This syntax is not supported
    ╭────
  1 │ (?<grp> "test")
    · ───┬───
    ·    ╰── error occurred here
    ╰────
    help: Named capturing groups use the `:name(...)` syntax. Try `:grp(...)` instead
  ```

### Changed

- A plus directly after a repetition (e.g. `'a'{2}+`) is now **forbidden**. Fix it by adding
  parentheses: `('a'{2})+`

  The reason is that this syntax is used by regular expressions for possessive quantifiers.
  Forbidding this syntax in Pomsky allows for better diagnostics.

- Deprecated `[.]`, `[codepoint]` and `[cp]`. They should have been deprecated before, but the
  warnings were missed in the previous release.

- Pomsky now sometimes reports multiple errors at once. The number of errors is limited to 8 in the
  CLI.

## [0.5.0] - 2022-07-04

This is the first release since [Rulex was renamed to Pomsky](https://pomsky-lang.org/blog/renaming-rulex/).

If you are using the `rulex` crate, replace it with `pomsky`. The `rulex-macro` crate should be replaced with `pomsky-macro`. To install the new binary, [see instructions](https://github.com/pomsky-lang/pomsky/releases/tag/v0.5). If you installed rulex with cargo, you can remove it with

```sh
rm $(type -P rulex)
```

### Added

- Deprecation warnings for `<%` and `%>`. These were deprecated before, but Pomsky wasn't able
  to show warnings until now.

### Changed

- Improved codegen for Unicode chars between 128 and 255

- Some diagnostics involving built-in variables were improved

- The words `atomic`, `if`, `else` and `recursion` are now reserved

### Fixed

- `Grapheme` is now only allowed in the PCRE, Java and Ruby flavors. Previously, it was accepted by
  Pomsky for some flavors that don't support `\X`.
- Keywords and reserved words are no longer accepted as variable names

### Library changes

- The `Rulex` struct was renamed to `Expr`, and `RulexFeatures` was renamed to `PomskyFeatures`
- `Span::range()` now returns an `Option<Range<usize>>` instead of a `Range<usize>`
- `Expr::parse` and `Expr::parse_and_compile` now return a `(String, Vec<Warning>)` tuple

## [0.4.3] - 2022-06-19

### Added

- Add libFuzzer and AFL fuzzing boilerplate to find panics

- Add artificial recursion limit during parsing to prevent stack exhaustion.
  _This means that groups can be nested by at most 127 levels. I don't think you'll ever run into this limitation, but if you do, you can refactor your expression into variables._

### Fixed

- Fixed crash caused by slicing into a multi-byte UTF-8 code point after a backslash or in a string
- Fixed crash caused by stack exhaustion when parsing a very deeply nested expression

## [0.4.2] - 2022-06-16

### Added

- Built-in variables were added:

  - `Start` as an alias for `<%`, which matches the start of the string
  - `End` as an alias for `%>`, which matches the end of the string
  - `Codepoint` and `C` as aliases for `[codepoint]`, matching a single code point
  - `G` as an alias for `Grapheme`, matching an extended grapheme cluster

- `Grapheme` was turned from a keyword into a built-in variable.

- The repository now has issue templates and a pull request template.

### Important note

`<%`, `%>`, `[codepoint]`, `[cp]` and `[.]` will be deprecated in the future. It is recommended
to use `Start`, `End` and `Codepoint`/`C` instead.

There won't be a replacement for `[.]`, but you can use `![n]` to match any code point except
the ASCII line break.

### Fixed/improved

- [#29](https://github.com/pomsky-lang/pomsky/pull/29): Fix a miscompilation of a repeated empty group,
  e.g. `()?`. Thanks, [sebastiantoh](https://github.com/sebastiantoh)!

- Make the parser more permissive to parse arbitrary negated expressions. This results in better
  error messages.

- Add missing help messages to diagnostics and fix a few that were broken:

  - When parsing `^`: _Use `Start` to match the start of the string_
  - When parsing `$`: _Use `End` to match the end of the string_
  - When parsing e.g. `(?<grp>)`: _Named capturing groups use the `:name(...)` syntax. Try `:grp(...)` instead_
  - When parsing e.g. `\4`: _Replace `\\4` with `::4`_
  - When parsing e.g. `(?<=test)`: _Lookbehind uses the `<<` syntax. For example, `<< 'bob'` matches if the position is preceded with bob._
  - When parsing e.g. `(?<!test)`: _Negative lookbehind uses the `!<<` syntax. For example, `!<< 'bob'` matches if the position is not preceded with bob._

- Improve test suite: Help messages are now tested as well, and failing tests can be "blessed" when
  the output has changed. Test coverage was also improved.

- The entire public API is now documented.

## [0.4.1] - 2022-06-03

### Fixed

- Fixed a miscompilation in situations where a variable followed by a `?` expands to a repetition

## [0.4.0] - 2022-06-03

The repository was moved to its own organization! 🎉 It also has a new website with an
[online playground](https://playground.pomsky-lang.org/)!

### Added

- API to selectively disable some language features

- [Online playground](https://playground.pomsky-lang.org/) to try out Rulex. You can write
  Rulex expressions on the left and immediately see the output on the right.

### Changed

- Ranges now have a maximum number of digits. The default is 6, but can be configured.

  This prevents DoS attacks when compiling untrusted input, since compiling ranges has exponential
  runtime with regard to the number of digits.

### Library changes

- `ParseOptions` was moved out of `CompileOptions`. This means that the
  [`parse_and_compile`](https://docs.rs/rulex/0.4.0/rulex/struct.Rulex.html#method.parse_and_compile)
  method now expects three parameters instead of two.

## [0.3.0] - 2022-03-29

### Added

- A [**book**](https://pomsky-lang.org/docs/), with instructions, a language tour and a formal
  grammar!

- **Variables**! For example, `let x = 'test';` declares a variable `x` that can be used below. Read
  [this chapter](https://pomsky-lang.org/docs/language-tour/variables) from the book to find
  out more.

- **Number range expressions**! For example, `range '0'-'255'` generates this regex:

  ```regexp
  0|1[0-9]{0,2}|2(?:[0-4][0-9]?|5[0-5]?|[6-9])?|[3-9][0-9]?
  ```

- **Relative references**: `::-1` refers to the previous capturing group, `::+1` to the next one

- `w`, `d`, `s`, `h`, `v` and `X` now have aliases: `word`, `digit`, `space`, `horiz_space`,
  `vert_space` and `Grapheme`.

- `enable lazy;` and `disable lazy;` to enable or disable lazy matching by default at the global
  scope or in a group.

### Changed

- **Made `greedy` the default** for repetitions. You can opt into lazy matching with the `lazy`
  keyword or globally with `enable lazy;`.

- **POSIX classes (e.g. `alnum`) have been renamed** to start with `ascii_`, since they only support
  Basic Latin

- Double quoted strings can now contain escaped quotes, e.g. `"\"test\""`. Backslashes now must be
  escaped. Single quoted strings were not changed.

- Improved Unicode support

  - In addition to Unicode general categories and scripts, Rulex now supports blocks and other
    boolean properties
  - Rulex now validates properties and tells you when a property isn't supported by the target
    regex flavor
  - Shorthands (`[h]` and `[v]`) are substituted with character classes when required to support
    Unicode everywhere

- Named references compile to numeric references (like relative references), which are better
  supported

- A `?` after a repetition is now forbidden, because it easy confuse to with a lazy quantifier.
  The error can be silenced by wrapping the inner expression in parentheses, e.g. `([w]{3})?`.

### Removed

- `R` was removed, because it didn't work properly, and I'm still unsure about the best syntax
  and behavior.

### Fixed

- A `?` following a repetition no longer miscompiles: `([w]{3})?` now correctly emits `(?:\w{3})?`
  instead of `\w{3}?`.
- A `{0,42}` repetition no longer miscompiles (it previously emitted `{,42}`).

## [0.2.0] - 2022-03-12

### Changed

- Improved the Rust macro; Rulex expressions are written directly in the Rust source code, not in a
  string literal:
  ```rs
  let regex: &str = rulex!("hello" | "world" '!'+);
  ```
- There are a few limitations in the Rust macro due to the way Rust tokenizes code:
  - Strings with more than 1 code point must be enclosed in double quotes, single quotes don't work
  - Strings can't contain backslashes; this will be fixed in a future release
  - Code points must be written without the `+`, e.g. `U10FFFF` instead of `U+10FFFF`
  - Rulex expressions can contain Rust comments; they can't contain comments starting with `#`

## [0.1.0] - 2022-03-11

Initial release

[unreleased]: https://github.com/pomsky-lang/pomsky/compare/v0.11...HEAD
[0.11.0]: https://github.com/pomsky-lang/pomsky/compare/v0.10...v0.11
[0.10.0]: https://github.com/pomsky-lang/pomsky/compare/v0.9...v0.10
[0.9.0]: https://github.com/pomsky-lang/pomsky/compare/v0.8...v0.9
[0.8.0]: https://github.com/pomsky-lang/pomsky/compare/v0.7...v0.8
[0.7.0]: https://github.com/pomsky-lang/pomsky/compare/v0.6...v0.7
[0.6.0]: https://github.com/pomsky-lang/pomsky/compare/v0.5...v0.6
[0.5.0]: https://github.com/pomsky-lang/pomsky/compare/v0.4.3...v0.5
[0.4.3]: https://github.com/pomsky-lang/pomsky/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/pomsky-lang/pomsky/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/pomsky-lang/pomsky/compare/v0.4...v0.4.1
[0.4.0]: https://github.com/pomsky-lang/pomsky/compare/v0.3...v0.4
[0.3.0]: https://github.com/pomsky-lang/pomsky/compare/v0.2...v0.3
[0.2.0]: https://github.com/pomsky-lang/pomsky/compare/v0.1...v0.2
[0.1.0]: https://github.com/pomsky-lang/pomsky/releases/tag/v0.1
[miette]: https://crates.io/crates/miette
