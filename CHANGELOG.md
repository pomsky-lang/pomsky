# Changelog

All notable changes to the _rulex regular expression language_ will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2022-03-29

### Added

- A [**book**](https://aloso.github.io/rulex/), with instructions, a language tour and a formal
  grammar!

- **Variables**! For example, `let x = 'test';` declares a variable `x` that can be used below. Read
  [this chapter](https://aloso.github.io/rulex/language-tour/variables.html) from the book to find
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

  - In addition to Unicode general categories and scripts, rulex now supports blocks and other
    boolean properties
  - Rulex now validates properties and tells you when a property isn't supported by the target
    regex flavor
  - Shorthands (`[h]` and `[v]`) are substituted with character classes when required to support
    Unicode everywhere

- Named references compile to numeric references (like relative references), which are better
  supported

- A `?` after a repetition is now forbidden, because it easy confuse to with a lazy quantifier.

### Removed

- `R` was removed, because it didn't work properly, and I'm still unsure about the best syntax
  and behavior. The error can be silenced by wrapping the inner expression in parentheses, e.g.
  `([w]{3})?`.

### Fixed

- A `?` following a repetition no longer miscompiles: `([w]{3})?` now correctly emits `(?:\w{3})?`
  instead of `\w{3}?`.
- A `{0,42}` repetition no longer miscompiles (it previously emitted `{,42}`).

## [0.2.0] - 2022-03-12

### Changed

- Improved the Rust macro; rulex expressions are written directly in the Rust source code, not in a
  string literal:
  ```rs
  let regex: &str = rulex!("hello" | "world" '!'+);
  ```
- There are a few limitations in the Rust macro due to the way Rust tokenizes code:
  - Strings with more than 1 code point must be enclosed in double quotes, single quotes don't work
  - Strings can't contain backslashes; this will be fixed in a future release
  - Code points must be written without the `+`, e.g. `U10FFFF` instead of `U+10FFFF`
  - Rulexes can contain Rust comments; they can't contain comments starting with `#`

## [0.1.0] - 2022-03-11

Initial release

[unreleased]: https://github.com/Aloso/rulex/compare/v0.3...HEAD
[0.3.0]: https://github.com/Aloso/rulex/compare/v0.2...v0.3
[0.2.0]: https://github.com/Aloso/rulex/compare/v0.1...v0.2
[0.1.0]: https://github.com/Aloso/rulex/releases/tag/v0.1
