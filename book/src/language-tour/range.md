# Range

Writing a regex matching a number in a certain range can be quite difficult. For example, the
following regex matches a number between 0 and 255:

```regexp
(?:2(?:5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])
```

This has many downsides:

- It's not readable
- It's difficult to come up with
- It's easy to make a mistake somewhere
- It's inefficient; a typical regex engine needs to backtrack in several places

Rulex solves these problems with its `range` syntax:

```rulex
range '0'-'255'
```

Rulex creates a **DFA** (deterministic finite automaton) from this, so the generated regex is
optimal in terms of matching performance. Since the algorithm for creating this regex is extensively
tested, you can also rely on it's correctness. Here's the regex generated from `range '0'-'255'`:

```regexp
0|1[0-9]{0,2}|2(?:[0-4][0-9]?|5[0-5]?|[6-9])?|[3-9][0-9]?
```

## Different bases

Rulex can generate ranges in various bases. For example, to match hexadecimal numbers in a certain
range, you might write:

```rulex
range '10F'-'FFFF' base 16
```

This generates this regex:

```regexp
1(?:0(?:[0-9a-eA-E][0-9a-fA-F]|[fF][0-9a-fA-F]?)|[1-9a-fA-F][0-9a-fA-F]{1,2})|[2-9a-fA-F][0-9a-fA-F]{2,3}
```
