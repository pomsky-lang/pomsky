# Rulex grammar

## Summary

This document uses rulex syntax. Here's an incomplete summary of the syntax, which should be enough
to read the grammar:

- Variables are declared as <rulex>`let var_name = expression;`. This assigns `expression` to the
  variable `var_name`.

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
let Expression = Statement* OrExpression;

let Statement = LetDeclaration | Modifier;

let LetDeclaration = 'let' VariableName '=' OrExpression ';';
let Modifier = ('enable' | 'disable') BooleanSetting ';';
let BooleanSetting = 'lazy';
```

### OrExpression

```rulex
let OrExpression = Alternative ('|' Alternative)*;

let Alternative = FixExpression+;
```

### FixExpression

An expression which can have a prefix or suffix.

```rulex
let FixExpression = LookaroundPrefix Expression
                  | AtomExpression RepetitionSuffix;
```

### Lookaround

```rulex
let LookaroundPrefix = '!'? ('<<' | '>>');
```

### Repetitions

```rulex
let RepetitionSuffix = ('*' | '+' | '?' | RepetitionBraces) Quantifier?;

let RepetitionBraces = '{' Number '}'
                     | '{' Number ',' Number '}'
                     | '{' Number ',' '}'
                     | '{' ',' Number '}';

let Number = '1'-'9' ('0'-'9')*;

let Quantifier = 'greedy' | 'lazy';
```

### AtomExpression

```rulex
let AtomExpression = Group
                   | String
                   | CharacterClass
                   | Grapheme
                   | Boundary
                   | Reference
                   | CodePoint
                   | NumberRange
                   | VariableName;
```

### Group

```rulex
let Group = Capture? '(' Expression ')';

let Capture = ':' Name?;

let Name = [w]+;
```

### String

```rulex
let String = '"' !['"']* '"'
           | "'" !["'"]* "'";
```

### CharacterClass

```rulex
let CharacterClass = '!'? '[' CharacterGroup ']';

let CharacterGroup = '.' | 'cp' | CharacterGroupMulti+;

let CharacterGroupMulti = Range
                        | Characters
                        | CodePoint
                        | NonPrintable
                        | Shorthand
                        | UnicodeProperty
                        | PosixClass;

let Range = Character '-' Character;

let Characters = '"' !['"']* '"'
               | "'" !["'"]* "'";

let Character = '"' !['"'] '"'
              | "'" !["'"] "'"
              | CodePoint
              | NonPrintable;

let NonPrintable = 'n' | 'r' | 't' | 'a' | 'e' | 'f';

let Shorthand = '!'? ('w' | 'word' |
                      'd' | 'digit' |
                      's' | 'space' |
                      'h' | 'horiz_space' |
                      'v' | 'vert_space' |
                      'l' | 'line_break');

let PosixClass = 'ascii_alpha' | 'ascii_alnum' | 'ascii' | 'ascii_blank'
               | 'ascii_cntrl' | 'ascii_digit' | 'ascii_graph' | 'ascii_lower'
               | 'ascii_print' | 'ascii_punct' | 'ascii_space' | 'ascii_upper'
               | 'ascii_word'  | 'ascii_xdigit';
```

### CodePoint

```rulex
let CodePoint = 'U+' ['0'-'9' 'a'-'f' 'A'-'F']{1,6}
              | 'U' ['0'-'9' 'a'-'f' 'A'-'F']{1,6};
```

Note that the second syntax exists mainly to be compatible with Rust tokenization.

### UnicodeProperty

Details about supported Unicode properties can be [found here](unicode-properties.md).

```rulex
let UnicodeProperty = '!'? [w]+;
```

### Grapheme

```rulex
let Grapheme = 'Grapheme' | 'X';
```

### Boundary

```rulex
let Boundary = '%' | '!' '%' | '<%' | '%>';
```

### NumberRange

```rulex
let NumberRange = 'range' String '-' String Base?;
let Base = 'base' Number;
```

### VariableName

```rulex
let VariableName = [w]+;
```

## Note about this grammar

Even though this grammar is written using rulex syntax, it isn't actually accepted by the rulex
compiler, because it uses cyclic variables.
