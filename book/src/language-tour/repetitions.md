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

This matches at least 3 times and at most 9 times. The default repetition mode in rulex is _greedy_,
like regexes. This means that rulex always tries to match an expression as many times as possible.

In situations where this is not desired, you can opt into non-greedy matching with the `lazy`
keyword, for example:

```rulex
('r' | 'w' | 'x' | '-'){3,9} lazy '--'
```

When given the string `rwxr--r--`, rulex will first repeat the group 3 times (the minimum number of
repetitions). Since there aren't two dashes after 3 characters, it is forced to repeat a 4th time.
`rwxr` is followed by two dashes, so rulex finds the match `rwxr--` and returns. The other possible
match, which is the entire string, isn't found, because the repetition is "too lazy".

## Variants of repetition

If we want to match an expression arbitrarily often, without an upper bound, we can just omit it:

```rulex
'test'{3,}
```

There are three kinds of repetition that are very common: `{0,}` (zero or more), `{1,}` (one or
more) and `{0,1}` (zero or one). These have dedicated symbols, `*`, `+` and `?`:

```rulex
'test'*     # match zero times or more
'test'+     # match one time or more
'test'?     # match zero or one time
```

You can also add the `lazy` keyword to them to opt into lazy matching.

## Enable lazy matching globally

If you enable the `lazy` mode, lazy repetition becomes the default, so it's necessary to opt into
greedy repetition with the `greedy` keyword:

```rulex
enable lazy;
'test'+         # this is lazy
'test'+ greedy
```

Lazy matching can be enabled or disabled in arbitrarily nested groups:

```rulex
(enable lazy;
  'test'+ # this is lazy
  (disable lazy;
    'test'+ # this is greedy
  )
  'test'+ # this is lazy
)
```
