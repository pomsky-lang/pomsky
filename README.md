<div align="center">

![Crown in double quotes logo](./assets/logo.svg)

# rulex

A new, portable, regular expression language

Read **[the book](https://rulex-rs.github.io/docs/)** to get started!

</div>

## Examples

On the left are rulex expressions (_rulexes_ for short), on the right is the compiled regex:

```py
# String
'hello world'                 # hello world

# Greedy repetition
'hello'{1,5}                  # (?:hello){1,5}
'hello'*                      # (?:hello)*
'hello'+                      # (?:hello)+

# Lazy repetition
'hello'{1,5} lazy             # (?:hello){1,5}?
'hello'* lazy                 # (?:hello)*?
'hello'+ lazy                 # (?:hello)+?

# Alternation
'hello' | 'world'             # hello|world

# Character classes
['aeiou']                     # [aeiou]
['p'-'s']                     # [p-s]

# Named character classes
[word] [space] [n]            # \w\s\n

# Combined
[w 'a' 't'-'z' U+15]          # [\wat-z\x15]

# Negated character classes
!['a' 't'-'z']                # [^at-z]

# Unicode
[Greek] U+30F Grapheme        # \p{Greek}\u030F\X

# Boundaries
Start End                     # ^$
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

# Backreferences
:('test') ::1                 # (test)\1
:name('test') ::name          # (?P<name>test)\1

# Ranges
range '0'-'999'               # 0|[1-9][0-9]{0,2}
range '0'-'255'               # 0|1[0-9]{0,2}|2(?:[0-4][0-9]?|5[0-5]?|[6-9])?|[3-9][0-9]?
```

## Variables

```rust
let operator = '+' | '-' | '*' | '/';
let number = '-'? [digit]+;

number (operator number)*
```

## Usage

Read **[the book](https://rulex-rs.github.io/docs)** to get started, or check out the
[CLI program](./rulex-bin/), the [Rust library](./rulex-lib/) and the
[procedural macro](./rulex-macro/).

## Why use this instead of normal regexes?

Normal regexes are very concise, but when they get longer, they get increasingly difficult to
understand. By default, they don't have comments, and whitespace is significant. Then there's the
plethora of sigils and backslash escapes that follow no discernible system:
`(?<=) (?P<>) .?? \N \p{} \k<> \g''` and so on. And with various inconsistencies between regex
implementations, it's the perfect recipe for confusion.

Rulex solves these problems with a new, simpler but also more powerful syntax:

- It's not whitespace sensitive and allows comments
- Text must appear in quotes. This makes expressions longer, but also much easier to read
- Non-capturing groups are the default
- More intuitive, consistent syntax
- Variables to make expressions DRY

## Compatibility

Rulex is currently compatible with PCRE, JavaScript, Java, .NET, Python, Ruby and Rust. The regex
flavor must be specified during compilation, so rulex can ensure that the produced regex works as
desired on the targeted regex engine.

**Note**: You should enable Unicode support in your regex engine, if it isn't enabled by default.
This is [explained here](https://rulex-rs.github.io/docs/get-started/enable-unicode/).

## Security

**Never compile or execute an untrusted rulex expression on your critical infrastructure**.
This may make you vulnerable for denial of service attacks, like the
[Billion Laughs attack](https://en.wikipedia.org/wiki/Billion_laughs_attack).

[Read more](https://rulex-rs.github.io/docs/reference/security/)

## Diagnostics

Rulex looks for mistakes and displays helpful diagnostics:

- It shows an error if you use a feature not supported by the targeted regex flavor
- It detects syntax errors and shows suggestions how to resolve them
- It parses backslash escapes (which are not allowed in a rulex) and explains what to write instead
- It looks for likely mistakes and displays warnings
- It looks for patterns that can be very slow for certain inputs and are susceptible to
  Denial-of-Service attacks _(coming soon)_

## Roadmap

[You can find the Roadmap here.](https://github.com/orgs/rulex-rs/projects/1)

## Contributing

You can contribute by using rulex and providing feedback. If you find a bug or have a question,
please create an issue.

I also gladly accept code contributions. To make sure that CI succeeds, please run `cargo fmt`,
`cargo clippy` and `cargo test` before creating a pull request.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the
[Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
