# Passwords

Here's a regular expression that tests if a string contains at least one uppercase letter,
lowercase letter, digit and punctuation/symbol code point, and is at least 8 code points long:

```regexp
^(?=[\S\s]*?\d)(?=[\S\s]*?\p{Ll})(?=[\S\s]*?\p{Lu})(?=[\S\s]*?[\pP\pS])[\S\s]{8}
```

Equivalent rulex:

```rulex
<%
(>> [cp]* [digit])
(>> [cp]* [Ll])
(>> [cp]* [Lu])
(>> [cp]* [P S])
[cp]{8}
```
