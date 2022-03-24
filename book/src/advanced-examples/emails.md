# Emails

[This StackOverflow answer](https://stackoverflow.com/a/201378) contains a massive regular
expression for matching any RFC 5322 compliant email address:

```regexp
(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])
```

If your regex engine supports insiginificant whitespace mode <regexp>`(?x)`, it can be written like
this:

```regexp
(?x)

(?:
  [a-z0-9!#$%&'*+/=?^_`{|}~-]+
  (?: \. [a-z0-9!#$%&'*+/=?^_`{|}~-]+ )*
| "
  (?:
    [\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]
  | \\ [\x01-\x09\x0b\x0c\x0e-\x7f]
  )*
  "
)
@
(?:
  (?: [a-z0-9] (?: [a-z0-9-]* [a-z0-9] )? \. )+
  [a-z0-9]
  (?: [a-z0-9-]* [a-z0-9] )?
| \[
  (?:
    (?: (2 (5 [0-5] | [0-4] [0-9]) | 1 [0-9] [0-9] | [1-9]? [0-9]) )
    \.
  ){3}
  (?:
    (2 (5 [0-5] | [0-4] [0-9]) | 1 [0-9] [0-9] | [1-9]? [0-9])
  | [a-z0-9-]*
    [a-z0-9]
    :
    (?:
      [\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]
    | \\ [\x01-\x09\x0b\x0c\x0e-\x7f]
    )+
  )
  \]
)
```

Here's a straightforward translation into rulex:

```rulex
(
  ['a'-'z' '0'-'9' "!#$%&'*+/=?^_`{|}~-"]+
  ('.' ['a'-'z' '0'-'9' "!#$%&'*+/=?^_`{|}~-"]+ )*
| '"'
  (
    [U+01-U+08 U+0b U+0c U+0e-U+1f U+21 U+23-U+5b U+5d-U+7f]
  | '\' [U+01-U+09 U+0b U+0c U+0e-U+7f]
  )*
  '"'
)
'@'
(
  ( ['a'-'z' '0'-'9'] ( ['a'-'z' '0'-'9' '-']* ['a'-'z' '0'-'9'] )? '.' )+
  ['a'-'z' '0'-'9']
  ( ['a'-'z' '0'-'9' '-']* ['a'-'z' '0'-'9'] )?
| '['
  (:(range '0'-'255') '.'){3}
  (
    :(range '0'-'255')
  | ['a'-'z' '0'-'9' '-']*
    ['a'-'z' '0'-'9']
    ':'
    (
      [U+01-U+08 U+0b U+0c U+0e-U+1f U+21-U+5a U+53-U+7f]
    | '\' [U+01-U+09 U+0b U+0c U+0e-U+7f]
    )+
  )
  ']'
)
```

Notice how the complex logic for matching a number between '0' and '255' is replaced by a simple
`range` expression in rulex.

Rulexes this complicated would also profit from a feature I have planned, but not yet implemented:
Variables.

With variables, we could write the above as follows:

```rulex
CharBeforeAt = ['a'-'z' '0'-'9' "!#$%&'*+/=?^_`{|}~-"];
QuotedCharBeforeAt = [U+01-U+08 U+0b U+0c U+0e-U+1f U+21 U+23-U+5b U+5d-U+7f];
EscapedCharBeforeAt = '\' [U+01-U+09 U+0b U+0c U+0e-U+7f];

Lower_Digit = ['a'-'z' '0'-'9'];
Lower_Digit_Dash = ['a'-'z' '0'-'9' '-'];

PortDigit = [U+01-U+08 U+0b U+0c U+0e-U+1f U+21-U+5a U+53-U+7f];
EscapedPortChar = '\' [U+01-U+09 U+0b U+0c U+0e-U+7f];


(
  CharBeforeAt+ ('.' CharBeforeAt+)*
| '"' (QuotedCharBeforeAt | EscapedCharBeforeAt)* '"'
)
'@'
(
  (Lower_Digit (Lower_Digit_Dash* Lower_Digit)? '.')+
  Lower_Digit
  (Lower_Digit_Dash* Lower_Digit)?
| '['
  (:(range '0'-'255') '.'){3}
  (
    :(range '0'-'255')
  | Lower_Digit_Dash*
    Lower_Digit
    ':'
    (PortDigit | EscapedPortChar)+
  )
  ']'
)
```
