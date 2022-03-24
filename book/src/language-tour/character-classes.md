# Character classes

What if we want to match an arbitrary word? Enumerating every single word is obviously not feasible,
so what to do instead? We can simply enumerate the characters and repeat them:

```rulex
('a' | 'b' | 'c' | 'd' | 'e' |
 'f' | 'g' | 'h' | 'i' | 'j' |
 'k' | 'l' | 'm' | 'n' | 'o' |
 'p' | 'q' | 'r' | 's' | 't' |
 'u' | 'v' | 'w' | 'x' | 'y' | 'z')+
```

But this very verbose and still only matches lowercase letters. We programmers tend to be lazy, so
there must be a more convenient solution!

## Character ranges

This expression matches words that can contain uppercase and lowercase letters:

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

This is equivalent to <rulex>`('$' | '_' | ['a'-'z' 'A'-'Z'])`, but it's shorter and may be
more efficient.

### Character ranges and Unicode

What is a range, exactly? Let's see with an example:

```rulex
['0'-'z']
```

This doesn't seem to make sense, but does work. If you compile it to a regex and
[try it out](https://regexr.com/6hagq), you'll notice that it matches numbers, lowercase and uppercase
letters. However, it also matches a few other characters, e.g. the question mark `?`.

The reason is that rulex uses Unicode, a standard that assigns every character a numeric value.
When we write <rulex>`'0'-'z'`, rulex assumes that we want to match any character whose
numeric value is somewhere between the value of <rulex>`'0'` and the value of <rulex>`'z'`.
This works well for letters (e.g. <rulex>`'a'-'Z'`) and numbers (<rulex>`'0'-'9'`), because
these have consecutive numbers in Unicode. However, there are some special characters
between digits, uppercase letters and lowercase letters:

| Character | Unicode value |
| --------- | ------------- |
| `'0'`     | 48            |
| `'1'`     | 49            |
| `'2'`     | 50            |
|           | ...           |
| `'9'`     | 57            |
| `':'`     | 58            |
| `';'`     | 59            |
| `'<'`     | 60            |
| `'='`     | 61            |
| `'>'`     | 62            |
| `'?'`     | 63            |
| `'@'`     | 64            |
| `'A'`     | 65            |
| `'B'`     | 66            |
|           | ...           |
| `'Z'`     | 90            |
| `'['`     | 91            |
| `'\'`     | 92            |
| `']'`     | 93            |
| `'^'`     | 94            |
| `'_'`     | 95            |
| `` '`' `` | 96            |
| `'a'`     | 97            |
|           | ...           |
| `'z'`     | 122           |

Why, you might ask? This is for [historical](https://en.wikipedia.org/wiki/ASCII#Overview)
[reasons](https://en.wikipedia.org/wiki/Unicode#History).

### Unicode properties

The reason why Unicode was invented is that most people in the world don't speak English, and many
of them use languages with different alphabets. To support them, Unicode includes 144,697 characters
covering 159 different scripts. Since we have a standard that makes it really easy to support
different languages, there's no excuse for not use it.

The character class <rulex>`['a'-'z' 'A'-'Z']` only recognizes Latin characters. What should we
do instead? We should use a
[Unicode category](https://en.wikipedia.org/wiki/Unicode_character_property#General_Category).
In this case, `Letter` seems like an obvious candidate. Rulex makes it very easy to use Unicode
categories:

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

## Negation

Character classes are negated by putting a <rulex>`!` in front of it. For example,
<rulex>`!['a'-'f']` matches anything except a letter in the range from `a` to `f`.

It's also possible to negate Unicode properties individually. For example, `[Latin !Alphabetic]`
matches a code point that is either in the Latin script, or is not alphabetic.

## Shorthand character classes

There are a few _shorthand character classes_: `word`, `digit`, `space`, `horiz_space` and
`vert_space`. They can be abbreviated with their first, letter: `w`, `d`, `s`, `h` and `v`. Like
Unicode properties, they must appear in square brackets.

- `word` matches a _word character_, i.e. a letter, digit or underscore. It's equivalent to
  <rulex>`[Alphabetic Mark Decimal_Number Connector_Punctuation Join_Control]`.
- `digit` matches a digit. It's equivalent to `Decimal_Number`.
- `space` matches whitespace. It's equivalent to `White_Space`.
- `horiz_space` matches horizontal whitespace (tabs and spaces). It's equivalent to
  <rulex>`[U+09 Space_Separator]`.
- `vert_space` matches vertical whitespace. It's equivalent to
  <rulex>`[U+0A-U+0D U+85 U+2028 U+2029]`.

Note that `word`, `digit` and `space` only match ASCII characters, if the regex engine isn't
configured to be Unicode-aware. How to enable Unicode support is
[described here](../enabling-unicode-support.md).

There are two more shorthands: <rulex>`[codepoint]` (or <rulex>`[cp]` for short), matches
any Unicode code point; <rulex>`[.]` matches any Unicode code point, _except_ the ASCII
line break `\n`. These two shorthands are special, because they have to be the only thing
in a character class; for example, <rulex>`[. 'x']` would be illegal, but also kind of useless.

### What if I don't need Unicode support?

You don't have to use Unicode-aware character classes such as <rulex>`[word]` if you know
that the input is only ASCII. Unicode-aware matching can be considerably slower. For example,
the <rulex>`[word]` character class includes more than 100,000 code points, so matching a
<rulex>`[ascii_word]`, which includes only 63 code points, is faster.

Rulex supports a number of ASCII-only shorthands:

| Character class         | Equivalent                                     |
| ----------------------- | ---------------------------------------------- |
| <rulex>`[ascii]`        | <rulex>`[U+00-U+7F]`                           |
| <rulex>`[ascii_alpha]`  | <rulex>`['a'-'z' 'A'-'Z']`                     |
| <rulex>`[ascii_alnum]`  | <rulex>`['0'-'9' 'a'-'z' 'A'-'Z']`             |
| <rulex>`[ascii_blank]`  | <rulex>`[' ' U+09],`                           |
| <rulex>`[ascii_cntrl]`  | <rulex>`[U+00-U+1F U+7F]`                      |
| <rulex>`[ascii_digit]`  | <rulex>`['0'-'9']`                             |
| <rulex>`[ascii_graph]`  | <rulex>`['!'-'~']`                             |
| <rulex>`[ascii_lower]`  | <rulex>`['a'-'z']`                             |
| <rulex>`[ascii_print]`  | <rulex>`[' '-'~']`                             |
| <rulex>`[ascii_punct]`  | <rulex>`` ['!'-'/' ':'-'@' '['-'`' '{'-'~'] `` |
| <rulex>`[ascii_space]`  | <rulex>`[' ' U+09-U+0D]`                       |
| <rulex>`[ascii_upper]`  | <rulex>`['A'-'Z']`                             |
| <rulex>`[ascii_word]`   | <rulex>`['0'-'9' 'a'-'z' 'A'-'Z' '_']`         |
| <rulex>`[ascii_xdigit]` | <rulex>`['0'-'9' 'a'-'f' 'A'-'F']`             |

Using them can improve performance, but be careful when you use them. If you aren't sure if the
input will ever contain non-ASCII characters, it's better to err on the side of correctness, and
use Unicode-aware character classes.

## Non-printable characters

Characters that can't be printed should be replaced with their hexadecimal Unicode code point. For
example, you may write <rulex>`U+FEFF` to match the
[Zero Width No-Break Space](https://www.compart.com/en/unicode/U+FEFF).

There are also 6 non-printable characters with a name:

- <rulex>`[n]` is equivalent to <rulex>`[U+0A]`, the `\n` line feed.
- <rulex>`[r]` is equivalent to <rulex>`[U+0D]`, the `\r` carriage return.
- <rulex>`[f]` is equivalent to <rulex>`[U+0C]`, the `\f` form feed.
- <rulex>`[a]` is equivalent to <rulex>`[U+07]`, the "alert" or "bell" control character.
- <rulex>`[e]` is equivalent to <rulex>`[U+0B]`, the "escape" control character.

Other characters have to be written in their hexadecimal form. Note that you don't need to write
leading zeroes, i.e. <rulex>`U+0` is just as ok as <rulex>`U+0000`. However, it is conventional
to write ASCII characters with two digits and non-ASCII characters with 4, 5 or 6 digits
depending on their length.

## Examples

Let's say we need to match a character that is either a letter, digit, underscore, dot or colon.
We can use the `word` shorthand, which includes everything we need except the dot and colon:

```rulex
[word '.:']
```

What if we want to match a letter or digit, but not an underscore? We can list just the things we
need:

```rulex
[Letter digit]
```

Another solution is to use negation to exclude the underscore from the `word` shorthand:

```rulex
![!word '_']
```

How does this work? Since the character class is negated, the part within the square bracket has
to match anything _except_ the things we want: Letters and digits. Since <rulex>`!word` also
doesn't match underscores, we add <rulex>`'_'` to get the desired result.
This "double negation trick" can be used to remove some things from a shorthand.
