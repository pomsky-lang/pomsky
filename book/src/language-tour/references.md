# References

Sometimes it's useful to match the same text as we matched before. For example, to match strings
in single or double quotes, we can write

```rulex
:(['"' "'"]) !['"' "'"]* ::1
```

This consists of three parts: First, there's a capturing group matching a quote. We then match an
arbitrary number of characters that aren't quotes. Finally, there's a `::1` reference. This
matches the same text as was captured in capturing group number 1. In other words, if the string
started with `"`, it also has to end with `"`, and if it started with `'`, it has to end with `'`.

Another application is XML tags:

```rulex
'<' :([word]+) '>' !['<']* '</' ::1 '>'
```

This is by no means a complete XML parser, but it recognizes an XML tag (without attributes) that
doesn't contain other XML tags. For example, it correctly matches `<span>Hallo Welt</span>`. With a
backreference, it ensures that the closing tag is the same as the opening tag.

Rulex has three kinds of references:

- Numeric references, e.g. `::3`, match a capturing group by its number.
- Named references, e.g. `::name`, match a named capturing group by its name.
- Relative references, e.g. `::-1` or `::+2`, match a capturing group relative to the current
  position. For example, `::-1` matches the previous capturing group, `::+1` matches the next one.

Note that some regex engines only support backreferences, not forward references. And even when
forward references are supported, the referenced group must have been already matched. I.e., this
is not allowed:

```rulex
# doesn't work!
::1 :('test')
```

However, forward references can be used in repetitions to match what the referenced group captured
in the previous repetition:

```rulex
(::forward | :forward('test')  '!')*
```

This matches the text `test!test`, for example. In the first repetition, the second alternative
matches `test!`, and the text `test` is captured by the `forward` capturing group. In the second
iteration, the forward reference matches the text `test`.
