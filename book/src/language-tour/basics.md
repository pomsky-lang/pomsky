# Basics

First, let's get familiar with the basic building blocks of the language.

Rulex expressions (_rulexes_ for short) describe the syntactical structure of a text. There are
several kinds of expressions, which will be explained now.

In Rulex, whitespace is insignificant, except between quotes. This means that we can add spaces
and line breaks to make the code look clearer. We can also add comments to explain what the
expressions are doing. They start with a `#` and span until the end of the line:

```rulex
# this is a comment
# comments are ignored by rulex!
```

## Strings

In Rulex, characters that should be matched as-is, are always wrapped in quotes. We can use
double quotes (<rulex>`""`) or single quotes (<rulex>`''`). Text wrapped in quotes we call a
_string_. It matches the exact content of the string:

```rulex
"test"
```

In double quoted strings (<rulex>`"..."`), double quotes can be escaped by prepending a backslash.
Backslashes also must be escaped:

```rulex
"\"C:\\windows\""
# is equivalent to
'"C:\windows"'
```

## Concatenate expressions

Rulex consists of _expressions_. For example, a string is an expression. If we write several
expressions in a row, they are matched one after the other:

```rulex
'hello' 'world' '!'     # matches the string "helloworld!"
```

## Alternatives

What if we want to match multiple strings? In a regex, we can enumerate multiple alternatives,
divided by a <rulex>`|`:

```regexp
one|two|three|four|five
```

The same works in Rulex:

```rulex
'one' | 'two' | 'three' | 'four' | 'five'
```

This type of expression is called an _alternation_.
