<div align="center">

![Crown in double quotes logo](./assets/logo.svg)

# rulex

A new, portable, regular expression language

</div>

## Examples

On the left are rulex expressions (_rulexes_ for short), on the right is the compiled regex:

```py
# String
'hello world'                 # hello world

# Lazy repetition
'hello'{1,5}                  # (?:hello){1,5}?
'hello'*                      # (?:hello)*?
'hello'+                      # (?:hello)+?

# Greedy repetition
'hello'{1,5} greedy           # (?:hello){1,5}
'hello'* greedy               # (?:hello)*
'hello'+ greedy               # (?:hello)+

# Alternation
'hello' | 'world'             # hello|world

# Character classes
['aeiou']                     # [aeiou]
['p'-'s']                     # [p-s]

# Named character classes
[.] [w] [s] [n]               # .\w\s\n

# Combined
[w 'a' 't'-'z' U+15]          # [\wat-z\x15]

# Negated character classes
!['a' 't'-'z']                # [^at-z]

# Unicode
[Greek] U+30F X               # \p{Greek}\u030F\X

# Boundaries
<% %>                         # ^$
% 'hello' !%                  # \bhello\B

# Non-capturing groups
'terri' ('fic' | 'ble')       # terri(?:fic|ble)

# Capturing groups
:('test')                     # (test)
:name('test')                 # (?P<name>test)

# Lookahead/lookbehind
>> 'foo' | 'bar'              # (?=foo|bar)
<< 'foo' | 'bar'              # (?<=foo|bar)
!>> 'foo' | 'bar'             # (?!foo|bar)
!<< 'foo' | 'bar'             # (?<!foo|bar)
```

## Why use this instead of normal regexes?

Normal regexes are very concise, but when they get longer, they get increasingly difficult to
understand. By default, they don't have comments, and whitespace is significant. Then there's the
plethora of sigils and backslash escapes that follow no discernible system:
`(?<=) (?P<>) .?? \N \p{} \k<> \g''` and so on. And with various inconsistencies between regex
implementations, it's the perfect recipe for confusion.

Rulex solves these problems with a new, simpler syntax:

- It's not whitespace sensitive and allows comments
- Text must appear in quotes. This makes expressions longer, but also much easier to read
- There are no backslash escapes
- Non-capturing groups are the default
- More consistent syntax

## Compatibility

Rulex is currently compatible with PCRE, JavaScript, Java, .NET, Python, Ruby and Rust. The regex
flavor must be specified during compilation, so rulex can ensure that the produced regex works as
desired on the targeted regex engine.

**Important note for JavaScript users**: Don't forget to enable the `u` flag. This is required for
Unicode support. All other major regex engines support Unicode by default.

## Diagnostics

Rulex looks for mistakes and displays helpful diagnostics:

- It shows an error if you use a feature not supported by the targeted regex flavor
- It detects syntax errors and shows suggestions how to resolve them
- It parses backslash escapes (which are not allowed in a rulex) and explains what to write instead
- It looks for likely mistakes and displays warnings
- It looks for patterns that can be very slow for certain inputs and are susceptible to
  Denial-of-Service attacks _(coming soon)_

## Usage

## Procedural macro

The Rust procedural macro allows converting a rulex to a regex string literal at compile time:

```rust
use rulex_macro::rulex;

const REGEX: &str = rulex!(r#" 'foo' | 'bar'+ greedy "#);
```

This means that errors from rulex are shown at compile time, too, and are highlighted in an IDE.

## CLI

The CLI currently requires that [Rust](https://www.rust-lang.org/tools/install) is installed.
Install the CLI with

```sh
cargo install rulex-bin
```

Then you can compile rulex expressions to a regex flavor of your choice; the default is PCRE.

```sh
$ rulex --help
rulex 0.1.0
Ludwig Stecher <ludwig.stecher@gmx.de>
Compile rulex expressions, a new regular expression language

USAGE:
    rulex [OPTIONS] [INPUT]

ARGS:
    <INPUT>    Rulex expression to compile

OPTIONS:
    -d, --debug              Show debug information
    -f, --flavor <FLAVOR>    Regex flavor [possible values: pcre, python,
                             java, javascript, dotnet, ruby, rust]
    -h, --help               Print help information
    -p, --path <FILE>        File containing the rulex expression to compile
    -V, --version            Print version information
```

## Roadmap

[You can find the Roadmap here.](https://github.com/users/Aloso/projects/1/views/1)

## Contributing

You can contribute by using rulex and providing feedback. If you find a bug or have a question,
please create an issue.

I also gladly accept code contributions. If you want to contribute, please upvote or comment on
[this issue](https://github.com/Aloso/rulex/issues/9), so I will prioritize documenting the code
and writing a Contributor's Guide.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
