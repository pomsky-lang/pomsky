# Rulex grammar

## Summary

This document uses rulex syntax, extended with variables. A variable assigns a rule to a name.
For example, `Hello = 'world'` assigns the `'world'` rule to the name `Hello`.

Here's an incomplete summary of the syntax, which should be enough to read the grammar:

- Verbatim text is wrapped in double quotes (`""`) or single quotes (`''`).

- A `*` after a rule indicates that it repeats 0 or more times.

- A `+` after a rule indicates that it repeats 1 or more times.

- A `?` after a rule indicates that the rule is optional.

- Consecutive rules can be grouped together by wrapping them in parentheses (`()`).

- Alternative rules are separated with a vertical bar (`|`).

- Character classes are wrapped in square brackets (`[]`).
  A character class matches exactly one code point. It can contain

  - sequences of characters (e.g. `'abdf'`, which matches either `a`, `b`, `d` or `f`)
  - Unicode ranges (e.g. `'0'-'9'`, which is equivalent to `'0123456789'`)
  - shorthands (e.g. `w`, which matches a letter, digit or the ASCII underscore `_`)

  An exclamation mark (`!`) in front of the character class negates it. For example, `![w]` matches
  anything _except_ a letter, digit or ASCII underscore.

### Whitespace

Comments start with `#` and end at the end of the same line.

Comments and whitespace are ignored; they can be added anywhere, except in strings, in tokens
(such as `>>`), in words, numbers and code points (such as `U+306F`). For example, `>>` can't be
written as `> >`, but `!>>'test'+` can be written as `! >> 'test' +`.

Whitespace is required between consecutive words and code points, e.g. `[a n Latin U+50]`.

## Formal grammar

### Expression

```
Expression = Alternative ('|' Alternative)*

Alternative = FixExpression+
```

### FixExpression

An expression which can have a prefix or suffix.

```
FixExpression = LookaroundPrefix Expression
              | AtomExpression RepetitionSuffix
```

### Lookaround

```
LookaroundPrefix = '!'? ('<<' | '>>')
```

### Repetitions

```
RepetitionSuffix = ('*' | '+' | '?' | RepetitionBraces) Quantifier?

RepetitionBraces = '{' Number '}'
                 | '{' Number ',' Number '}'
                 | '{' Number ',' '}'
                 | '{' ',' Number '}'

Number = '1'-'9' ('0'-'9')*

Quantifier = 'greedy'
```

### AtomExpression

```
AtomExpression = Group
               | String
               | CharacterClass
               | Grapheme
               | Boundary
```

### Group

```
Group = Capture? '(' Expression ')'

Capture = ':' Name?

Name = [w]+
```

### String

```
String = '"' !['"']* '"'
       | "'" !["'"]* "'"
```

### CharacterClass

```
CharacterClass = '!'? '[' CharacterGroup ']'

CharacterGroup = '.' | 'cp' | CharacterGroupMulti+

CharacterGroupMulti = Range
                    | Characters
                    | CodePoint
                    | NonPrintable
                    | Shorthand
                    | UnicodeCategory
                    | PosixClass

Range = Character '-' Character

Characters = '"' !['"']* '"'
           | "'" !["'"]* "'"

Character = '"' !['"'] '"'
          | "'" !["'"] "'"
          | CodePoint
          | NonPrintable

CodePoint = 'U+' ['0'-'9' 'a'-'f' 'A'-'F']{1,5}

NonPrintable = 'n' | 'r' | 't' | 'a' | 'e' | 'f'

Shorthand = '!'? ('w' | 'd' | 's' | 'h' | 'v' | 'R')

UnicodeCategory = '!'? ['A'-'Z'] [w]*

PosixClass = 'alpha' | 'alnum' | 'ascii' | 'blank'
           | 'cntrl' | 'digit' | 'graph' | 'lower'
           | 'print' | 'punct' | 'space' | 'upper'
           | 'word'  | 'xdigit'
```

### Grapheme

```
Grapheme = 'Grapheme'
```

### Boundary

```
Boundary = '%' | '!' '%' | '<%' | '%>'
```
