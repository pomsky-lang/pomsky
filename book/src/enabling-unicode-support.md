# Enabling Unicode support

Rulex has good Unicode support, but you might still have to enable Unicode support in your regex
engine. This document explains how to do that for various regex engines.

If some information here is missing, outdated or needs clarification, I would greatly appreciate
your help! You can
[edit this file](https://github.com/Aloso/rulex/tree/main/book/src/enabling-unicode-support.md) on
GitHub.

## Rust

The Rust `regex` crate is Unicode-aware by default. There's nothing you need to do.

## JavaScript

In JavaScript, set the `u` flag, for example `/[\w\s]/u`. This makes it possible to use Unicode
properties (`\p{...}`) and code points outside of the BMP (`\u{...}`).

However, `[w]` and `[d]` are _not_ Unicode aware even when the `u` flag is enabled! `[s]` is always
Unicode aware.

As an alternative, you may substitute

- `[word]` with `[Alphabetic Mark Decimal_Number Connector_Punctuation]` (or `[Alpha M Nd Pc]`
  for short)
- `[digit]` with `[Decimal_Number]` (or `[Nd]` for short)

To make word boundaries behave correctly, you also need to substitute

```rulex
# %
(<< [Alpha M Nd Pc]) (!>> [Alpha M Nd Pc]) | (!<< [Alpha M Nd Pc]) (>> [Alpha M Nd Pc])

# !%
(<< [Alpha M Nd Pc]) (>> [Alpha M Nd Pc]) | (!<< [Alpha M Nd Pc]) (!>> [Alpha M Nd Pc])
```

Substituting `[word]` and `[digit]` will be done automatically in the next version for JavaScript.
For word boundaries, we're currently looking for a better alternative.

## PHP

PHP is Unicode-aware if the `u` flag is set, and this also applies to `[w]`, `[d]`, `[s]` and `%`.
For example, `'/\w+/u'` matches a word in any script.

## PCRE

PCRE supports Unicode, but to make `[w]`, `[d]`, `[s]` and `%` Unicode-aware, you need to enable
both `PCRE_UTF8` and `PCRE_UCP`.

## Java

In Java, add `(?U)` in front of the regex to make it Unicode-aware. For example, `"(?U)\\w+"`
matches a word in any script.

## Ruby

In Ruby, add `(?u)` in front of the regex to make it Unicode-aware. For example, `/(?u)\w+/` matches
a word in any script.

## Python

In the Python `re` module, `[w]`, `[d]`, `[s]` and `%` are Unicode-aware since Python 3.

If you're still using Python 2, you can use the [regex](https://pypi.org/project/regex/2021.11.10/)
module from November 2021; releases newer than that don't support Python 2.
