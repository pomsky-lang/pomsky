<div align="center">

![Crown in double quotes logo](./assets/logo.svg)

# rulex

A new, portable, regular expression language

</div>

## Examples

On the left are rulex expressions (_rulexes_ for short), on the right is the compiled regex, enclosed in `//`:

```regexp
# String
'hello world'                 /hello world/

# Repetition
'hello'{1,5}                  /(?:hello){1,5}?/

# Greedy repetition
'hello'{1,5} greedy           /(?:hello){1,5}/

# Alternation
'hello' | 'world'             /hello|world/

# Character classes/ranges
['aeiou' 'p'-'s']             /[aeioup-s]/

# Named character classes
[.] [X] [w] [s] [n]           /.\X\w\s\n/

# Mixed and negated character classes
[not w 'a' 't'-'z']           /[^\wat-z]/

# Unicode
[Greek] U+30F                 /\p{Greek}\u030F/

# Boundaries
<% %>                         /^$/
% 'hello' not %               /\bhello\B/

# Capturing groups
:('test')                     /(test)/
:name('test')                 /(?P<name>test)

# Lookahead/lookbehind
>> 'foo' | 'bar'              /(?=foo|bar)/
<< 'foo' 'bar'?               /(?<=foo(?:bar)??)/
(not >> ['foo']) 'bar'        /(?![fo])bar/
```

## Why use this instead of normal regexes?

POSIX regexes are very concise, simple and easy to parse. However, they quickly get very long and
difficult to understand. Also, it's not always clear which characters need to be escaped, and
repetitions are greedy by default. This can cause bugs that are difficult to track down.

Rulex is designed to be much easier to understand even when the regular expression is long.
It doesn't have escape sequences, instead using single or double quotes for raw text.
Repetitions in rulex are non-greedy by default. The language is designed to be intuitive:

Rulex allows you to specify the regex flavor. It can compile to regexes that are compatible with
PCRE, JavaScript, Java, .NET, Python, Ruby or Rust.

## Portability

Rulex tries its best to emit regexes with consistent behavior across regex engines. Not all features
are supported for every regex flavor. However, if a rulex compiles, it should work as expected on
every regex engine. If you find an inconsistency, please file an issue!

When you use rulex for JavaScript, don't forget to enable the `u` flag. This is required for
Unicode support.

## Usage

Requires that [Rust](https://www.rust-lang.org/tools/install) is installed.

Install the rulex CLI tool with

```sh
cargo install rulex-bin
```

Then you can compile rulex expressions. Input can be provided from a CLI argument, from a file or
from stdin:

```sh
$ rulex "'foo' | 'bar' | 'baz'"
$ rulex --path ./file.rulex
$ cat ./file.rulex | rulex
```

You can also specify the regex flavor:

```sh
rulex --flavor js --path ./file.rulex
```

## TODO (short-term)

- Compilation (generating regexes) is currently untested.
- Provide an ASCII mode (escaping all non-ASCII characters)
- Add backreferences and forward references.

## Roadmap

[You can find the Roadmap here.](./Roadmap.md)

## Contributing

You can contribute by filing issues or sending pull requests. If you have questions, please create an issue.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the [Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
