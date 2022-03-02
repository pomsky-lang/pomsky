<div align="center">

![Crown in double quotes logo](./assets/logo.svg)

# rulex

A new, portable, regular expression language

</div>

## Examples

On the left are rulex expressions (_rulexes_ for short), on the right is the compiled regex:

```regexp
# String
'hello world'                 hello world

# Repetition
'hello'{1,5}                  (?:hello){1,5}?

# Greedy repetition
'hello'{1,5} greedy           (?:hello){1,5}

# Alternation
'hello' | 'world'             hello|world

# Character classes
['aeiou']                     [aeiou]
['p'-'s']                     [p-s]

# Named character classes
[.] [X] [w] [s] [n]           .\X\w\s\n

# Combined
[w 'a' 't'-'z' U+15]          [\wat-z\x15]

# Negated character classes
!['a' 't'-'z']                [^at-z]

# Unicode
[Greek] U+30F                 \p{Greek}\u030F

# Boundaries
<% %>                         ^$
% 'hello' !%                  \bhello\B

# Non-capturing groups
'terri' ('fic' | 'ble')       terri(?:fic|ble)

# Capturing groups
:('test')                     (test)
:name('test')                 (?P<name>test)

# Lookahead/lookbehind
>> 'foo' | 'bar'              (?=foo|bar)
<< 'foo' 'bar'?               (?<=foo(?:bar)??)
!>> [.]* 'awesome'            (?!.*?awesome)
```

## Why use this instead of normal regexes?

Normal regexes are very concise, but when they get longer, they get increasingly difficult to
understand. By default, they don't have comments, and whitespace is significant. Then there's the
plethora of sigils and backslash escapes that follow no discernible system:
`(?<=) (?P<>) .?? \N \p{} \k<> \g''` and so on. Add inconsistencies between regex implementations,
and you have the perfect recipe for confusion.

Rulex solves these problems by introducing a new, simpler and more consistent syntax:

- It's not whitespace sensitive and allows comments.
- Text must appear in quotes. This makes expressions longer, but also much easier to read.
- There are no backslash escapes.
- Non-capturing groups are the default.
- More consistent syntax:
  - Negation is always denoted with an `!` exclamation mark
  - Character classes, shorthands, POSIX classes and Unicode categories share the same syntax
    with `[` brackets `]`.
- Currently compatible with PCRE, JavaScript, Java, .NET, Python, Ruby and Rust.

## Portability

Rulex tries its best to emit regexes with consistent behavior across all regex engines. Not every
feature is supported in every regex flavor, but rulex will kindly show an error if you try to use an
unsupported feature. The aim is that, if a rulex compiles successfully, it works as expected;
there should be no subtle differences between regex engines. If you find an inconsistency,
please file an issue!

**Important note for JavaScript users**: Don't forget to enable the `u` flag. This is required for
Unicode support. All other major regex engines support Unicode by default.

## Usage

There's a Rust library and a CLI. IDE integration and a procedural macro are also planned.

The CLI requires that [Rust](https://www.rust-lang.org/tools/install) is installed.

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
    -d, --debug
            Show debug information

    -f, --flavor <FLAVOR>
            Regex flavor [possible values: pcre, python, java,
            javascript, dotnet, ruby, rust]

    -h, --help
            Print help information

    -p, --path <FILE>
            File containing the rulex expression to compile

    -V, --version
            Print version information
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
