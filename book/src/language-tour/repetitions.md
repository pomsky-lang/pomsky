# Repetitions

When we want to match an expression multiple times, it would be cumbersome to repeat our expression.
Instead, we can specify how often the expression should occur:

```rulex
('r' | 'w' | 'x' | '-'){9}
```

This matches an `r`, `w`, `x` or `-` character 9 times. For example, it would match the string
`rwxr-xr--`, or `xxrr-xr-w`.

What if we want to match strings of different lengths? Repetitions are quite flexible, so we can
specify a lower and upper bound for the number of repetitions:

```rulex
('r' | 'w' | 'x' | '-'){3,9}
```

## Greedy and lazy matching

This matches at least 3 times and at most 9 times. The default repetition mode in rulex is _lazy_,
unlike regexes (which are greedy by default).

This means that rulex always tries to match an expression as few times as possible. This means that,
since rulexes are usually allowed to match only _part_ of the text, the above expression will always
stop after the third repetition.

> I'm considering to change this.

This is obviously not very useful in this case. So we can opt into greedy matching with the `greedy`
keyword:

```rulex
('r' | 'w' | 'x' | '-'){3,9} greedy
```

Now it will greedily match the expression as often as possible, up to 9 times.

## Variants of repetition

If we want to match an expression arbitrarily often, without an upper bound, we can just omit it:

```rulex
'test'{3,} greedy
```

There are three kinds of repetition that are very common: `{0,}` (zero or more), `{1,}` (one or
more) and `{0,1}` (zero or one). These have dedicated symbols, `*`, `+` and `?`:

```rulex
'test'*     # match zero times or more
'test'+     # match one time or more
'test'?     # match zero or one time
```

Note that these also require the `greedy` keyword to opt into greedy matching.
