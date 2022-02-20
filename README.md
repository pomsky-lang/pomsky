<div align="center">

# rulex

A new regular expression language

</div>

## Examples

On the left are rulex expressions, on the right is the compiled regex:

```python
# String
'hello world'               /hello world/

# Repetition
'hello'{1,5}                /(?:hello){1,5}?/

# Greedy repetition
'hello'{1,5} greedy         /(?:hello){1,5}/

# Alternation
'hello' | 'world'           /hello|world/

# Character classes/ranges
[aeiou] | ']' | 'p'-'s'     /[aeiou\]p-s]/

# Named character classes
<all> <.> <w> <s>           /[^].\w\s/

# Negation
<all>! <.>! <w>! <s>!       /[]\n\W\S/

# Unicode
<Greek> U+205               /\p{Greek}\u{0205}/

# Boundaries
%- -%                       /^$/
% 'hello' %!                /\bhello\B/

# Capturing groups
:('test')                   /(test)/
:name('test')               /(?P<name>test)
```

## Why use this instead of normal regexes?

POSIX regexes are very concise, simple and easy to parse. However, they quickly get very long and difficult to understand. Also, it's not always clear which characters need to be escaped, and repetitions are greedy by default. This can cause bugs that are difficult to track down.

Rulex is designed to be much easier to understand even when the regular expression is long. It doesn't have escape sequences, instead using single or double quotes for raw text. Repetitions in rulex are non-greedy by default.

Rulex compiles to PCRE-compatible regexes, but in the future it will be possible to specify the regex dialect.

## Usage

Requires that [Rust](https://www.rust-lang.org/tools/install) is installed.

Install the rulex CLI tool with

```sh
cargo install rulex-bin
```

Then you can compile rulex expressions like this:

```sh
rulex "'foo' | 'bar' | 'baz'"
```

Or from a file:

```sh
rulex --path ./file.rulex
```

Or from another command:

```
cat ./file.rulex | rulex
```

## TODO

Soon rulex will support

- Lookahead/lookbehind
- Backreferences
- Variables

Variables is a concept that doesn't exist in regular expression engines, so variables are inlined during compilation. Variables can't have cyclic dependencies, since that would mean that the expression is no longer regular. Still, this is a powerful feature that makes regular expressions much easier to understand:

```python
WS      :=  (<s> | <n>)* greedy
WORD    :=  % <w> (<w> | <d>)* greedy %
DIGITS0 := <d>+ greedy
DIGITS1 := '0' | '1'-'9' <d>* greedy
NUMBER  :=  '-'? DIGITS1 ("." DIGITS0)? greedy

"let" WS WORD WS "=" WS NUMBER
```

The above compiles to

```
/let[\s\n]*\b\w[\w\d]*\b[\s\n]*=[\s\n]*-?(?:0|[1-9]\d*(?:\.\d+)?)/
```

## Contributing

You can contribute by filing issues or sending pull requests. If you have questions, please create an issue.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the [Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
