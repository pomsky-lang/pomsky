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

In JavaScript, set the `u` flag, for example <regexp>`/[\w\s]/u`. This makes it possible to use
Unicode properties (<regexp>`\p{...}`) and code points outside of the BMP (<regexp>`\u{...}`).

However, <rulex>`[w]` and <rulex>`[d]` are _not_ Unicode aware even when the `u` flag is enabled!
<rulex>`[s]` is always Unicode aware.

As an alternative, you may substitute

- <rulex>`[word]` with <rulex>`[Alphabetic Mark Decimal_Number Connector_Punctuation]`
  (or <rulex>`[Alpha M Nd Pc]` for short)
- <rulex>`[digit]` with <rulex>`[Decimal_Number]` (or <rulex>`[Nd]` for short)

To make word boundaries behave correctly, you also need to substitute

```rulex
# %
(<< [Alpha M Nd Pc]) (!>> [Alpha M Nd Pc]) | (!<< [Alpha M Nd Pc]) (>> [Alpha M Nd Pc])

# !%
(<< [Alpha M Nd Pc]) (>> [Alpha M Nd Pc]) | (!<< [Alpha M Nd Pc]) (!>> [Alpha M Nd Pc])
```

Substituting <rulex>`[word]` and <rulex>`[digit]` will be done automatically in the next version
for JavaScript. For word boundaries, we're currently looking for a better alternative.

## PHP

PHP is Unicode-aware if the `u` flag is set, and this also applies to <rulex>`[w]`, <rulex>`[d]`,
<rulex>`[s]` and <rulex>`%`. For example, <rulex>`'/\w+/u'` matches a word in any script.

## PCRE

PCRE supports Unicode, but to make <rulex>`[w]`, <rulex>`[d]`, <rulex>`[s]` and <rulex>`%`
Unicode-aware, you need to enable both `PCRE_UTF8` and `PCRE_UCP`.

## Java

In Java, add <regexp>`(?U)` in front of the regex to make it Unicode-aware. For example,
`"(?U)\\w+"` matches a word in any script.

## Ruby

In Ruby, add <regexp>`(?u)` in front of the regex to make it Unicode-aware. For example,
<regexp>`/(?u)\w+/` matches a word in any script.

## Python

In the Python `re` module, <rulex>`[w]`, <rulex>`[d]`, <rulex>`[s]` and <rulex>`%` are
Unicode-aware since Python 3.

If you're still using Python 2, you can use the [regex](https://pypi.org/project/regex/2021.11.10/)
module from November 2021; releases newer than that don't support Python 2.
