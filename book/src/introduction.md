<div style="text-align: center">

![Rulex Logo](./assets/logo.svg)

# Rulex

</div>

Rulex is a language that compiles to regular expressions. It is currently in an alpha stage and
will likely change substantially in the next few releases.

## Usage

Rulex can be used with a CLI or a Rust macro. See
[installation instructions](installation-instructions.md).

## Basics

Rulex expressions (_rulexes_ for short) describe the syntactical structure of a text. There are
several kinds of expressions, which will be explained now.

This introduction assumes basic knowledge of regexes. If you aren't familiar with them, I highly
recommend [this introduction](https://www.regular-expressions.info/quickstart.html).

### Table of contents:

- [Summary](#summary)
- [Strings](#strings)
- [Concatenate expressions](#concatenate-expressions)
- [Alternatives](#alternatives)
- [Groups](#groups)
- [Repetitions](#repetitions)
  - [Greedy and lazy matching](#greedy-and-lazy-matching)
  - [Variants of repetition](#variants-of-repetition)
- [Character classes](#character-classes)
  - [About Unicode ranges](#about-unicode-ranges)
- [Unicode support](#unicode-support)
- [Boundaries](#boundaries)
- [Lookaround](#lookaround)
- [Grapheme](#grapheme)

### Summary

Here you can see all the features at a glance. Don't worry, they will be explained in more detail
below.

On the left are rulex expressions, on the right are the equivalent regexes:

```rulex
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
[Greek] U+30F Grapheme        # \p{Greek}\u030F\X

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

# Backreferences
:('test') ::1                 # (test)\1
:name('test') ::name          # (?P<name>test)\k<name>
```

### Strings

In Rulex, characters that should be matched as-is, are always wrapped in quotes. We can use
double quotes (`""`) or single quotes (`''`). Text wrapped in quotes is an expression we call a
_string_. It matches the exact content of the string:

```rulex
"test"
```

### Concatenate expressions

If we write several expressions in a row, they are matched one after the other:

```rulex
'hello' 'world' '!'     # matches the string "helloworld!"
```

In Rulex, whitespace is insignificant, except between quotes. This means that can add spaces
and line breaks to make it look clearer. We can also add comments to explain what the expressions
are doing. They start with a `#` and span until the end of the line:

```rulex
# this is a comment
'hello'     # this is also a comment
'world'     # and this
```

### Alternatives

What if we want to match multiple strings? In a regex, we can enumerate multiple alternatives,
divided by a `|`:

```regexp
one|two|three|four|five
```

The same works in Rulex:

```rulex
'one' | 'two' | 'three' | 'four' | 'five'
```

### Groups

Multiple expressions can be grouped together by wrapping them in `()`. This is useful when we have
multiple alternatives that all start or end with the same thing:

```rulex
'tang' ('ible' | 'ent' | 'o')
```

This matches the words _tangible_, _tangent_ and _tango_.

Groups can also be used to _capture_ their content, e.g. to replace it with something else. In
regexes, every group is a capturing group by default. This is not the case in rulex: Capturing
groups must be prefixed with `:`.

```rulex
:('foo')
```

Capturing groups are consecutively numbered, to be able to refer to them later:

```rulex
:('Max' | 'Laura') (' is ' | ' was ') :('asleep' | 'awake')
```

The first group, containing the name, has index **1**, the third group with the adverb has the index
**2**. The second group is skipped because it isn't capturing (it isn't prefixed with `:`).

This means that you can add non-capturing groups freely without accidentally changing the capturing
group numbers. However, it's usually better to use _named capturing groups_, so you don't need to
count groups and instead refer to each group by a name:

```rulex
:name('Max' | 'Laura') (' is ' | ' was ') :adverb('asleep' | 'awake')
```

### Repetitions

When we want to match an expression multiple times, it would be cumbersome to repeat our expression.
Instead, we can specify how often the expression should occur:

```rulex
('r' | 'w' | 'x' | '-'){9}
```

This matches an `r`, `w`, `x` or `-` character 9 times. For example, it would match the string
`rwxr-xr--`, or `xxrr-xr-w`.

What if we want to match strings of different lengths? Repetitions are quite flexible, so we can
specify a lower and upper bound for the number of repetitions:

```rulex
('r' | 'w' | 'x' | '-'){3,9}
```

#### Greedy and lazy matching

This matches at least 3 times and at most 9 times. The default repetition mode in rulex is _lazy_,
unlike regexes (which are greedy by default).

This means that rulex always tries to match an expression as few times as possible. This means that,
since rulexes are usually allowed to match only _part_ of the text, the above expression will always
stop after the third repetition.

> I'm considering to change this.

This is obviously not very useful in this case. So we can opt into greedy matching with the `greedy`
keyword:

```rulex
('r' | 'w' | 'x' | '-'){3,9} greedy
```

Now it will greedily match the expression as often as possible, up to 9 times.

#### Variants of repetition

If we want to match an expression arbitrarily often, without an upper bound, we can just omit it:

```rulex
'test'{3,} greedy
```

There are three repetitions that are very common: `{0,}` (zero or more), `{1,}` (one or more) and
`{0,1}` (zero or one). These have dedicated symbols, `*`, `+` and `?`:

```rulex
'test'*     # match zero times or more
'test'+     # match one time or more
'test'?     # match zero or one time
```

Note that these also require the `greedy` keyword to opt into greedy matching.

### Character classes

What if we want to match an arbitrary word? Enumerating every single word is obviously not feasible,
so what to do instead? We can simply enumerate the characters and repeat them:

```rulex
('a' | 'b' | 'c' | 'd' | 'e' |
 'f' | 'g' | 'h' | 'i' | 'j' |
 'k' | 'l' | 'm' | 'n' | 'o' |
 'p' | 'q' | 'r' | 's' | 't' |
 'u' | 'v' | 'w' | 'x' | 'y' | 'z')+
```

This is pretty verbose, but it could be worse. But this only matches lowercase letters. Also, we
programmers tend to be lazy, so there's a more convenient solution:

```rulex
['a'-'z' 'A'-'Z']+
```

What is this? The square brackets indicate that this is a _character class_. A character class
always matches exactly 1 character (more precisely, a Unicode code point). This character class
contains two ranges, one for lowercase letters and one for uppercase letters. Together, this
matches any character that is either a lowercase or uppercase letter.

It's also possible to add single characters, for example:

```rulex
['$' '_' 'a'-'z' 'A'-'Z']
```

When we have several characters in a character class that aren't part of a range, we can simply
put them into the same quotes:

```rulex
['$_' 'a'-'z' 'A'-'Z']
```

#### About Unicode ranges

What is a range, exactly? Let's see with an example:

```rulex
['0'-'z']
```

This doesn't seem to make sense, but does work. If you compile it to a regex and
[try it out](https://regexr.com/6hagq), you'll notice that it matches numbers, lowercase and uppercase
letters. However, it also matches a few other characters, e.g. the question mark `?`.

The reason is that rulex uses Unicode, a standard that assigns every character a numeric value.
When we write `'0'-'z'`, rulex assumes that we want to match any character whose numeric value
is somewhere between the value of `'0'` and the value of `'z'`. This works well for letters (e.g.
`'a'-'Z'`) and numbers (`'0'-'9'`), because these have consecutive numbers in Unicode. However,
there are some special characters between digits, uppercase letters and lowercase letters:

```rulex
Character       Unicode value
=============================
'0'             48
'1'             49
'2'             50
      ...
'9'             57
':'             58
';'             59
'<'             60
'='             61
'>'             62
'?'             63
'@'             64
'A'             65
'B'             66
      ...
'Z'             90
'['             91
'\'             92
']'             93
'^'             94
'_'             95
'`'             96
'a'             97
      ...
'z'             122
```

Why, you might ask? This is for [historical](https://en.wikipedia.org/wiki/ASCII#Overview)
[reasons](https://en.wikipedia.org/wiki/Unicode#History).

### Unicode support

The reason why Unicode was invented is that most people in the world don't speak English, and many
of them use languages with different alphabets. To support them, Unicode includes 144,697 characters
covering 159 different scripts. Since we have a standard that makes it really easy to support
different languages, there's no excuse for not use it.

The character class `['a'-'z' 'A'-'Z']` only recognizes Latin characters. What should we do instead?
We should use a
[Unicode category](https://en.wikipedia.org/wiki/Unicode_character_property#General_Category).
In this case, the obvious candidate is `Letter`. Rulex makes it very easy to use Unicode categories:

```rulex
[Letter]
```

That's it. This matches any letter from all 159 scripts! It's also possible to match any character
in a specific script:

```rulex
[Cyrillic Hebrew]
```

This matches a Cyrillic or Hebrew character. Not sure why you'd want to do that.

Some regex engines can also match Unicode properties other than categories and scripts. Probably
the most useful ones are

- `Alphabetic` (includes letters and marks that can appear in a word)
- `White_Space`
- `Uppercase`, `Lowercase`
- `Emoji`

You can see the full list of Unicode properties [here](./unicode-properties.md).

#### Negation

TODO

#### Special character classes

TODO

<!-- Mention w, d, s, v, h, l, [.] and [cp] -->
<!-- Mention ascii_* POSIX classes -->

#### Special characters

TODO

<!-- Mention n, r, t, a, e, f -->
<!-- Mention U+XXXX -->

### Boundaries

TODO

### Lookaround

TODO

### Grapheme

TODO
