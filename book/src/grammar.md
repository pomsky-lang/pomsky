# Rulex grammar

## Summary

This document uses rulex syntax, extended with variables. A variable assigns a rule to a name.
For example, <rulex>`Hello = 'world'` assigns the <rulex>`'world'` rule to the name <rulex>`Hello`.

Here's an incomplete summary of the syntax, which should be enough to read the grammar:

- Verbatim text is wrapped in double quotes (<rulex>`""`) or single quotes (<rulex>`''`).

- A <rulex>`*` after a rule indicates that it repeats 0 or more times.

- A <rulex>`+` after a rule indicates that it repeats 1 or more times.

- A <rulex>`?` after a rule indicates that the rule is optional.

- Consecutive rules can be grouped together by wrapping them in parentheses (<rulex>`()`).

- Alternative rules are separated with a vertical bar (<rulex>`|`).

- Character classes are wrapped in square brackets (<rulex>`[]`).
  A character class matches exactly one code point. It can contain

  - sequences of characters (e.g. <rulex>`'abdf'`, which matches either `a`, `b`, `d` or `f`)
  - Unicode ranges (e.g. <rulex>`'0'-'9'`, which is equivalent to <rulex>`'0123456789'`)
  - shorthands (e.g. <rulex>`w`, which matches a letter, digit or the ASCII underscore `_`)

  An exclamation mark (<rulex>`!`) in front of the character class negates it. For example,
  <rulex>`![w]` matches anything _except_ a letter, digit or ASCII underscore.

### Whitespace

Comments start with `#` and end at the end of the same line.

Comments and whitespace are ignored; they can be added anywhere, except in strings, in tokens
(such as <rulex>`>>`), in words, numbers and code points (such as <rulex>`U+306F`). For example,
<rulex>`>>` can't be written as <rulex>`> >`, but <rulex>`!>>'test'+` can be written as
<rulex>`! >> 'test' +`.

Whitespace is required between consecutive words and code points, e.g. <rulex>`[a n Latin U+50]`.

## Formal grammar

### Expression

```rulex
Expression = Modifier? OrExpression

Modifier = ('enable' | 'disable') BooleanSetting ';'
BooleanSetting = 'lazy'
```

### OrExpression

```rulex
OrExpression = Alternative ('|' Alternative)*

Alternative = FixExpression+
```

### FixExpression

An expression which can have a prefix or suffix.

```rulex
FixExpression = LookaroundPrefix Expression
              | AtomExpression RepetitionSuffix
```

### Lookaround

```rulex
LookaroundPrefix = '!'? ('<<' | '>>')
```

### Repetitions

```rulex
RepetitionSuffix = ('*' | '+' | '?' | RepetitionBraces) Quantifier?

RepetitionBraces = '{' Number '}'
                 | '{' Number ',' Number '}'
                 | '{' Number ',' '}'
                 | '{' ',' Number '}'

Number = '1'-'9' ('0'-'9')*

Quantifier = 'greedy' | 'lazy'
```

### AtomExpression

```rulex
AtomExpression = Group
               | String
               | CharacterClass
               | Grapheme
               | Boundary
               | Reference
               | CodePoint
               | NumberRange;
```

### Group

```rulex
Group = Capture? '(' Expression ')'

Capture = ':' Name?

Name = [w]+
```

### String

```rulex
String = '"' !['"']* '"'
       | "'" !["'"]* "'"
```

### CharacterClass

```rulex
CharacterClass = '!'? '[' CharacterGroup ']'

CharacterGroup = '.' | 'cp' | CharacterGroupMulti+

CharacterGroupMulti = Range
                    | Characters
                    | CodePoint
                    | NonPrintable
                    | Shorthand
                    | UnicodeProperty
                    | PosixClass

Range = Character '-' Character

Characters = '"' !['"']* '"'
           | "'" !["'"]* "'"

Character = '"' !['"'] '"'
          | "'" !["'"] "'"
          | CodePoint
          | NonPrintable

NonPrintable = 'n' | 'r' | 't' | 'a' | 'e' | 'f'

Shorthand = '!'? ('w' | 'word' |
                  'd' | 'digit' |
                  's' | 'space' |
                  'h' | 'horiz_space' |
                  'v' | 'vert_space' |
                  'l' | 'line_break')

PosixClass = 'ascii_alpha' | 'ascii_alnum' | 'ascii' | 'ascii_blank'
           | 'ascii_cntrl' | 'ascii_digit' | 'ascii_graph' | 'ascii_lower'
           | 'ascii_print' | 'ascii_punct' | 'ascii_space' | 'ascii_upper'
           | 'ascii_word'  | 'ascii_xdigit'
```

### CodePoint

```rulex
CodePoint = 'U+' ['0'-'9' 'a'-'f' 'A'-'F']{1,6}
          | 'U' ['0'-'9' 'a'-'f' 'A'-'F']{1,6}
```

Note that the second syntax exists mainly to be compatible with Rust tokenization.

### UnicodeProperty

Details about supported Unicode properties can be [found here](unicode-properties.md).

```rulex
UnicodeProperty = '!'? [w]+
```

### Grapheme

```rulex
Grapheme = 'Grapheme' | 'X'
```

### Boundary

```rulex
Boundary = '%' | '!' '%' | '<%' | '%>'
```

### NumberRange

```rulex
NumberRange = 'range' String '-' String Base?
Base = 'base' Number
```
