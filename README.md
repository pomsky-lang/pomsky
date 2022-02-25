<div align="center">

![Crown in double quotes logo](./assets/logo.svg)

# rulex

A new, portable, regular expression language

</div>

## Examples

On the left are rulex expressions (_rulexes_ for short), on the right is the compiled regex, enclosed in `//`:

```python
# String
'hello world'                 /hello world/

# Repetition
'hello'{1,5}                  /(?:hello){1,5}?/

# Greedy repetition
'hello'{1,5} greedy           /(?:hello){1,5}/

# Alternation
'hello' | 'world'             /hello|world/

# Character classes/ranges
['aeiou]' 'p'-'s']            /[aeiou\]p-s]/

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

## TODO (short-term)

- Compilation (generating regexes) is currently untested.
- Provide an ASCII mode (escaping all non-ASCII characters)
- Add backreferences and forward references.

## Roadmap

### Backreferences

Backreferences will look like this:

```
# Reference by index:
:(['"]) <.>* ::1            (['"]).*?\1

# Reference by name:
:quote(['"]) <.>* ::quote   (?P<quote>['"]).*?\k<quote>
```

I'm also considering an option or syntax to remove the group names during compilation; they would
still be useful to reference a group by its name (similar to loop labels in programming languages).
This has the advantage that we could support regex engines like JavaScript that support
backreferences but not named groups. But this can't be the default behavior, because we want
to support search/replace by group name in engines that support it, e.g. Rust.

### Better support for character classes

- Most importantly: `<X> -> \X`.

- Add the `Is` prefix to scripts (e.g. `IsLatin`) and `In`/`Is` to blocks as required by the engine

- Convert Unicode categories and scripts into ranges to support engines like JS

- Rulex should be able to detect when an invalid character class is provided.

### Variables

Variables is a concept that doesn't exist in regular expression engines, so variables are inlined
during compilation. Variables can't have cyclic dependencies, since that would mean that the
expression is no longer regular. Still, this is a powerful feature that makes regular expressions
much easier to understand:

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

Note that backreferences within variable definitions are counted from the start of the definition:

```
STR  :=  :(['"]) <.>* ::1

STR " - " STR " - " STR
```

The above compiles to

```
(['"]).*?\1 - (['"]).*?\2 - (['"]).*?\3
```

Note that the backreference is adjusted depending on where the variable is inserted. If named groups
are used, they clash, which might cause problems. If this isn't desired, a possible solution is to
append a `$` when group names clash:

```
STR  :=  :quotes$(['"]) <.>* ::quotes

STR " - " STR " - " STR
```

Which compiles to

```
(?P<quotes1>['"]).*?\k<quotes1> - (?P<quotes2>['"]).*?\k<quotes2> - (?P<quotes3>['"]).*?\k<quotes3>
```

### Atomic groups

Enabled with the `atomic` keyword: `('foo' | 'bar') atomic`

### Possessive matching

Enabled with the `possessive` keyword: `'foo'* possessive`

## Optimization

Allow optimizing expressions with an `optimize` keyword. This finds common prefixes in alternatives,
e.g. `optimize ('school' | 'schooling' | 'scholar')` would be compiled to
`scho(?:(?:ol(?:ing)??)|lar)`. This regex is more efficient in conventional regex engines that don't
optimize the regex themselves and use backtracking by default.

## Contributing

You can contribute by filing issues or sending pull requests. If you have questions, please create an issue.

## License

Dual-licensed under the [MIT license](https://opensource.org/licenses/MIT) or the [Apache 2.0 license](https://opensource.org/licenses/Apache-2.0).
