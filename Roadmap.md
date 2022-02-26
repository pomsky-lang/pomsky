# Roadmap

This is the list of planned features. Some of these are quite ambitious, so it will take some time
to implement them.

## Error handling

Parse errors are already acceptable. They aren't beautiful, but they provide all the necessary
context. Compile errors still need some improvement.

The next step is to switch to [miette](https://crates.io/crates/miette) to make the errors more
beautiful in the CLI.

## Language features

### Backreferences

Backreferences (and forward references) will look like this:

```regexp
# Reference by index:
:(['"]) [.]* ::1            (['"]).*?\1

# Reference by name:
:quote(['"]) [.]* ::quote   (?P<quote>['"]).*?\k<quote>
```

### Explicit index backreferences

Inline backreferences (and inline forward references) will look like this:

```regexp
:1(['"]) [.]* ::1           (['"]).*?\1
```

This emits an error if the group marked with `:1` isn't the first group, and makes it easier to
find the group referenced with `::1`. This is valuable if there are many, possibly nested,
capturing groups.

Note that named capturing groups are usually better, but JavaScript doesn't support them, although
it does support backreferences.

### Better Unicode support

- Add the `Is` prefix to scripts (e.g. `IsLatin`) and `In`/`Is` to blocks as required by the engine

- Rulex should be able to detect when an invalid character class is provided.

### Flags we might want to support

- `i`: case insensitive
- `m`: multi-line mode (`<%` and `%>` match begin/end of line)
- `s`: allow `[.]` to match `\n` (not needed since we have `[cp]`)
- `U`: make regexes greedy (not lazy) by default. Repetitions require `lazy` suffix to opt out
- `J`: allow duplicate group names (PCRE only)
- `d`: opt-out of Unicode support for `[d]`, so it only matches ASCII `\d`

Note that `i`, `m` and `s` are available in JS, but they can only be applied to the entire regex.

Possible syntax:

```regexp
set ignore_case, multi_line, single_line, greedy, reuse_group_names, ascii_n

'hello'* lazy [.] 'world'
(set not ignore_case, not greedy
  '!'*
)
```

### Atomic groups

Enabled with the `atomic` keyword: `('foo' | 'bar') atomic` compiles to `(?>foo|bar)` where
supported.

I don't intend to implement possessive quantifiers, because wrapping the rulex in an atomic group
has the same effect.

### Variables

Variables is a concept that doesn't exist in regular expression engines, so variables are inlined
during compilation. Variables can't have cyclic dependencies, since that would mean that the
expression is no longer regular. Still, this is a powerful feature that makes regular expressions
much easier to understand:

```regexp
WS      :=  [s n]* greedy
WORD    :=  % [w] [w d]* greedy %
DIGITS0 := [d]+ greedy
DIGITS1 := '0' | ['1'-'9'] [d]* greedy
NUMBER  :=  '-'? DIGITS1 ('.' DIGITS0)? greedy

'let' WS WORD WS '=' WS NUMBER
```

The above compiles to

```regexp
/let[\s\n]*\b\w[\w\d]*\b[\s\n]*=[\s\n]*-?(?:0|[1-9]\d*(?:\.\d+)?)/
```

Note that backreferences within variable definitions are counted from the start of the definition:

```regexp
STR  :=  :(['"]) [.]* ::1

STR " - " STR " - " STR
```

The above compiles to

```regexp
(['"]).*?\1 - (['"]).*?\2 - (['"]).*?\3
```

Note that the backreference is adjusted depending on where the variable is inserted. If named groups
are used, they clash, which might cause problems. If this isn't desired, a possible solution is to
append a `$` when group names clash:

```regexp
STR  :=  :quotes$(['"]) [.]* ::quotes

STR " - " STR " - " STR
```

Which compiles to

```regexp
(?P<quotes1>['"]).*?\k<quotes1> - (?P<quotes2>['"]).*?\k<quotes2> - (?P<quotes3>['"]).*?\k<quotes3>
```

### Conditionals

Most regex engines support conditionals à là `(?(?=condition)then|else)` or
`(?P<foo>condition)? (?(foo)then|else)`. The syntax varies slightly between implementations.

A subset of this feature can be be made available in engines like JavaScript, Python and Rust that
don't support conditionals, by negating the condition in the `else` branch. This only works for
character classes and lookaround, because they can be negated.

The syntax I have in mind:

```regexp
if >> 'foo' then
  :([.]{10})
else
  :([d w]{15})
```

When targeting PCRE, this can compile to

```regexp
(?(?=foo).{10}|[\d\w]{15})
```

When targeting engines that don't support conditionals:

```regexp
(?=foo).{10}|(?!foo)[\d\w]{15}
```

PCRE would also support the following, whereas JavaScript/Python/Rust can't:

```regexp
<% (:person('To' | 'From') | 'Subject') ': '
(if ::person exists then
  [w]+ '@' [w]+ '.' ['a'-'z']+
else
  [.]+
)
```

### Recursion

Using the keyword `recursion`. For example:

```regexp
'(' recursion* ')' | [not '()']+
```

Compiles to

```regexp
\((?R)*\)|[^()]+?
```

Which matches any text with balanced parentheses.

## Optimization

Allow optimizing expressions with an `optimize` keyword. This finds common prefixes in alternatives,
e.g. `optimize ('school' | 'schooling' | 'scholar')` would be compiled to
`scho(?:(?:ol(?:ing)??)|lar)`. This regex is more efficient in conventional regex engines that don't
optimize the regex themselves and use backtracking by default.
