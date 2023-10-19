<div align="center">

![Pomsky logo](https://raw.githubusercontent.com/pomsky-lang/pomsky/main/assets/logo.svg)

# Pomsky

A portable<sup><a href="#portability">1</a></sup>, modern regular expression language

[![Website][web-badge]][web-link] [![Documentation][doc-badge]][doc-link] [![Playground][playground-badge]][playground-link] [![VS Code plugin][vscode-badge]][vscode-link] [![Discord][discord-badge]][discord-link] [![Crates.io][crates-badge]][crates-link]

</div>

[web-badge]: https://img.shields.io/badge/website-%23e70?style=for-the-badge&logo=esri
[web-link]: https://pomsky-lang.org
[doc-badge]: https://img.shields.io/badge/documentation-%23b90?style=for-the-badge&logo=read.cv
[doc-link]: https://pomsky-lang.org/docs/get-started/introduction/
[playground-badge]: https://img.shields.io/badge/Playground-%232a2?style=for-the-badge&logo=asciinema
[playground-link]: https://playground.pomsky-lang.org
[vscode-badge]: https://img.shields.io/badge/VS%20Code%20plugin-blue?style=for-the-badge&logo=visualstudiocode
[vscode-link]: https://marketplace.visualstudio.com/items?itemName=pomsky-lang.pomsky-vscode
[discord-badge]: https://img.shields.io/badge/discord-%2355d?style=for-the-badge&logo=discord&logoColor=%23fff
[discord-link]: https://discord.gg/uwap2uxMFp
[crates-badge]: https://img.shields.io/crates/v/pomsky-bin?style=for-the-badge&color=red
[crates-link]: https://crates.io/crates/pomsky-bin
[test-badge]: https://github.com/pomsky-lang/pomsky/actions/workflows/test.yml/badge.svg
[test-link]: https://github.com/pomsky-lang/pomsky/actions/workflows/test.yml
[coverage-badge]: https://coveralls.io/repos/github/pomsky-lang/pomsky/badge.svg?branch=main
[coverage-link]: https://coveralls.io/github/pomsky-lang/pomsky?branch=main

## Get Started

To begin, check out [the website][web-link].

## What's New

Read the [blog](https://pomsky-lang.org/blog/) or the [changelog](./CHANGELOG.md) to learn about new features.

## Installation

Pre-built binaries can be found on the [releases page](https://github.com/pomsky-lang/pomsky/releases).

Pomsky can also be installed from source via [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) with `cargo install pomsky-bin`.

## Compatibility

Pomsky is currently compatible with PCRE, JavaScript, Java, .NET, Python, Ruby and Rust. The regex
flavor must be specified during compilation, so Pomsky can ensure that the produced regex works as
desired on the targeted regex engine.

**Note**: You should enable Unicode support in your regex engine, if it isn't enabled by default.
This is [explained here][enable-unicode].

## Portability

Pomsky aims to be as portable as possible, polyfilling Unicode and unsupported features where feasible. That said, there are some cases where portability is not possible:

- Some features (e.g. lookaround, backreferences, Unicode properties) aren't supported in every flavor. Pomsky fails to compile when you're using an unsupported feature.

- `\b` (word boundaries) are not Unicode aware in JavaScript. Pomsky therefore only allows word boundaries when Unicode is disabled.

- `\w` in .NET handles Unicode incorrectly, with no way to polyfill it properly. This means that in .NET, `[word]` only matches the `L`, `Mn`, `Nd`, and `Pc` general categories, instead of `Alphabetic`, `M`, `Nd`, `Pc` and `Join_Control`.

- In .NET, `.`, `Codepoint` and character classes (e.g. `[Latin]`) only match a single UTF-16 code unit rather than a codepoint.

- `[space]` matches slightly different code points in JavaScript than in Java. This will be fixed.

- Backreferences behave differently in JavaScript and Python when the referenced group has no captured text. There is nothing we can do about it, but we could add a warning for this in the future.

## Security

**Never compile or execute an untrusted Pomsky expression on your critical infrastructure**.
This may make you vulnerable for denial of service attacks, like the
[Billion Laughs attack][billion-lols].

[Read more][security]

## Diagnostics

Pomsky looks for mistakes and displays helpful diagnostics:

- It shows an error if you use a feature not supported by the targeted regex flavor
- It detects syntax errors and shows suggestions on how to resolve them
- It parses backslash escapes (which are not allowed in a Pomsky expression) and explains what to
  write instead
- It looks for likely mistakes and displays warnings
- It looks for patterns that can be very slow for certain inputs and are susceptible to
  Denial-of-Service attacks _(coming soon)_

## Comparison with other projects

I wrote an in-depth comparison with similar projects, which [you can find here][comparison].

## Code of Conduct

The Code of Conduct [can be found here](./CODE_OF_CONDUCT.md).

## Contributing

You can contribute by using Pomsky and providing feedback. If you find a bug or have a question,
please create an issue.

I also gladly accept code contributions. [More information](./CONTRIBUTING.md)

## Sponsor this project

[Go to my sponsors page](https://github.com/sponsors/Aloso/)

## License

Dual-licensed under the [MIT license][mit-license] or the [Apache 2.0 license][apache-2-license].

[book]: https://pomsky-lang.org/docs/get-started/introduction/
[enable-unicode]: https://pomsky-lang.org/docs/get-started/enable-unicode/
[billion-lols]: https://en.wikipedia.org/wiki/Billion_laughs_attack
[security]: https://pomsky-lang.org/docs/reference/security/
[comparison]: https://pomsky-lang.org/docs/reference/comparison/
[mit-license]: https://opensource.org/licenses/MIT
[apache-2-license]: https://opensource.org/licenses/Apache-2.0
