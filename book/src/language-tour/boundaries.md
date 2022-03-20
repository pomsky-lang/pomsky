# Boundaries

Boundaries match a position in a string without consuming any code points. There are 4 boundaries:

- `%` matches a word boundary. It matches successfully if it is preceded, but not succeeded by a
  word character, or vice versa. For example, `[cp] % [cp]` matches `A;` and `;A`, but not `AA` or
  `;;`.
- `!%` matches a position that is _not_ a word boundary. For example, `[cp] !% [cp]` matches `aa`
  and `::`, but not `a:` or `:a`.
- `<%` matches the start of the string.
- `%>` matches the end of the string.

A word character is anything that matches `[word]`. If the regex engine is Unicode-aware, this is
`[Alphabetic Mark Decimal_Number Connector_Punctuation]`. For some regex engines, Unicode-aware
matching has to be enabled first ([see here](./enabling-unicode-support.md)).

In JavaScript, `%` and `!%` is _never_ Unicode-aware, even when the `u` flag is set.
[See here](./enabling-unicode-support.md#javascript) for more information.
