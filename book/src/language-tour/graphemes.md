# Graphemes

Some regex engines support matching a single Unicode grapheme cluster. A grapheme cluster is what
comes closest to what we perceive as a character. It can consist of more than one code point.
For example, emojis with modifiers (such as different skin tones, genders, etc.) usually consist of
multiple code points.

In Rulex, matching a grapheme cluster is done with <rulex>`Grapheme`. For example, to match
a text with at most 400 characters, you can write

```rulex
Grapheme{0,400}
```
