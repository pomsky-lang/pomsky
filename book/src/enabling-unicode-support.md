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

Since `\w` and `\d` are _not_ Unicode aware even when the `u` flag is enabled, rulex polyfills them.
However, word boundaries aren't Unicode aware, and there's no straightforward solution for this.
To make word boundaries behave correctly, you need to substitute <rulex>`%` and <rulex>`!%` with

```rulex
(<< [w]) (!>> [w]) | (!<< [w]) (>> [w])
# and
(<< [w]) (>> [w]) | (!<< [w]) (!>> [w])
```

respectively.

## PHP

PHP is Unicode-aware if the `u` flag is set, and this also applies to <regexp>`\w`, <regexp>`\d`,
<regexp>`\s` and <regexp>`\b`. For example, <regexp>`'/\w+/u'` matches a word in any script.

## PCRE

PCRE supports Unicode, but to make <regexp>`\w`, <regexp>`\d`, <regexp>`\s` and <regexp>`\b`
Unicode-aware, you need to enable both `PCRE_UTF8` and `PCRE_UCP`.

## Java

In Java, add <regexp>`(?U)` in front of the regex to make it Unicode-aware. For example,
`"(?U)\\w+"` matches a word in any script.

## Ruby

In Ruby, add <regexp>`(?u)` in front of the regex to make it Unicode-aware. For example,
<regexp>`/(?u)\w+/` matches a word in any script.

## Python

In the Python `re` module, <regexp>`\w`, <regexp>`\d`, <regexp>`\s` and <regexp>`\b` are
Unicode-aware since Python 3.

If you're still using Python 2, you can use the [regex](https://pypi.org/project/regex/2021.11.10/)
module from November 2021; releases newer than that don't support Python 2.
