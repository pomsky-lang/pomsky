<div align="center">

![Pomsky logo](https://raw.githubusercontent.com/pomsky-lang/pomsky/main/assets/logo.svg)

# Pomsky

A portable<sup><a href="#portability">1</a></sup>, modern regular expression language

[![Website][web-badge]][web-link] [![Docs][doc-badge]][doc-link] [![Playground][playground-badge]][playground-link] [![VS Code plugin][vscode-badge]][vscode-link] [![Discord][discord-badge]][discord-link] [![Crates.io][crates-badge]][crates-link]

</div>

[web-badge]: https://img.shields.io/badge/website-%23e70?style=for-the-badge&logo=esri
[web-link]: https://pomsky-lang.org
[doc-badge]: https://img.shields.io/badge/docs-%23b90?style=for-the-badge&logo=read.cv
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

## Compatibility and portability

Pomsky is currently compatible with PCRE, JavaScript, Java, .NET, Python, Ruby, Rust, and RE2. The regex
flavor must be specified during compilation, so Pomsky can ensure that the produced regex works as
desired on the targeted regex engine.

**Note**: You should enable Unicode support in your regex engine, if it isn't enabled by default.
This is [explained here][enable-unicode].

There are a few situations where Pomsky expressions are not portable, [explained here][portability].

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
[portability]: https://pomsky-lang.org/docs/appendix/portability/
[billion-lols]: https://en.wikipedia.org/wiki/Billion_laughs_attack
[security]: https://pomsky-lang.org/docs/reference/security/
[comparison]: https://pomsky-lang.org/docs/reference/comparison/
[mit-license]: https://opensource.org/licenses/MIT
[apache-2-license]: https://opensource.org/licenses/Apache-2.0
