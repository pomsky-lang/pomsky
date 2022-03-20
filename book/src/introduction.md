<div style="text-align: center">

![Rulex Logo](./assets/logo.svg)

# Rulex

</div>

Rulex is a language that compiles to regular expressions. It is currently in an alpha stage and
will likely change substantially in the next few releases.

## Usage

Rulex can be used with a CLI or a Rust macro. See
[installation instructions](installation-instructions.md).

You should also enable Unicode support in your regex engine if it isn't supported by default.
[See instructions](./enabling-unicode-support.md).

## Summary

Here you can see all the features at a glance. Don't worry, they will be explained in more detail
in the [language tour](./language-tour/README.md).

On the left are rulex expressions, on the right are the equivalent regexes:

```rulex
# String
'hello world'                 # hello world

# Lazy repetition
'hello'{1,5}                  # (?:hello){1,5}?
'hello'*                      # (?:hello)*?
'hello'+                      # (?:hello)+?

# Greedy repetition
'hello'{1,5} greedy           # (?:hello){1,5}
'hello'* greedy               # (?:hello)*
'hello'+ greedy               # (?:hello)+

# Alternation
'hello' | 'world'             # hello|world

# Character classes
['aeiou']                     # [aeiou]
['p'-'s']                     # [p-s]

# Named character classes
[.] [w] [s] [n]               # .\w\s\n

# Combined
[w 'a' 't'-'z' U+15]          # [\wat-z\x15]

# Negated character classes
!['a' 't'-'z']                # [^at-z]

# Unicode
[Greek] U+30F Grapheme        # \p{Greek}\u030F\X

# Boundaries
<% %>                         # ^$
% 'hello' !%                  # \bhello\B

# Non-capturing groups
'terri' ('fic' | 'ble')       # terri(?:fic|ble)

# Capturing groups
:('test')                     # (test)
:name('test')                 # (?P<name>test)

# Lookahead/lookbehind
>> 'foo' | 'bar'              # (?=foo|bar)
<< 'foo' | 'bar'              # (?<=foo|bar)
!>> 'foo' | 'bar'             # (?!foo|bar)
!<< 'foo' | 'bar'             # (?<!foo|bar)

# Backreferences
:('test') ::1                 # (test)\1
:name('test') ::name          # (?P<name>test)\k<name>

# Ranges
range '0'-'999'               # 0|[1-9][0-9]{0,2}
range '0'-'255'               # 0|1[0-9]{0,2}|2(?:[0-4][0-9]?|5[0-5]?|[6-9])?|[3-9][0-9]?
```
