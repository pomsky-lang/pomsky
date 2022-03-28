# Boundaries

Boundaries match a position in a string without consuming any code points. There are 4 boundaries:

- <rulex>`%` matches a word boundary. It matches successfully if it is preceded,
  but not succeeded by a word character, or vice versa.
  For example, <rulex>`[cp] % [cp]` matches `A;` and `;A`, but not `AA` or `;;`.
- <rulex>`!%` matches a position that is _not_ a word boundary.
  For example, <rulex>`[cp] !% [cp]` matches `aa` and `::`, but not `a:` or `:a`.
- <rulex>`<%` matches the start of the string.
- <rulex>`%>` matches the end of the string.

A word character is anything that matches <rulex>`[word]`. If the regex engine is Unicode-aware,
this is <rulex>`[Alphabetic Mark Decimal_Number Connector_Punctuation]`. For some regex engines,
Unicode-aware matching has to be enabled first ([see here](../enabling-unicode-support.md)).

In JavaScript, <rulex>`%` and <rulex>`!%` is _never_ Unicode-aware, even when the `u` flag is set.
[See here](../enabling-unicode-support.md#javascript) for more information.
