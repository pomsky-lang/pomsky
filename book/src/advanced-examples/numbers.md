# Numbers

This regular expression matches rational numbers in decimal notation
with optional separating commas:

```regexp
[-+]??\b(?:0|[1-9](?:,??[0-9])*)(?:\.[0-9]+)?\b
```

Equivalent rulex:

```rulex
['-+']?
%
('0' | ['1'-'9'] (','? ['0'-'9'])* greedy)
('.' ['0'-'9']+ greedy)? greedy
%
```
