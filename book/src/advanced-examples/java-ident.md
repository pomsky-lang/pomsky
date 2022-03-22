# Java Identifiers

Regex matching a Java identifier:

```regexp
[\p{Connector_Punctuation}\p{Currency_Symbol}\p{Mark}\p{Alphabetic}][\p{Connector_Punctuation}\p{Currency_Symbol}\p{Mark}\p{Alphabetic}\p{Numeric}]*
```

With abbreviations:

```regexp
[\p{Pc}\p{Sc}\p{M}\p{Alphabetic}][\p{Pc}\p{Sc}\p{M}\p{Alphabetic}\p{Numeric}]*
```

And as a rulex:

```rulex
[Pc Sc M Alphabetic]
[Pc Sc M Alphabetic Numeric]*
```
