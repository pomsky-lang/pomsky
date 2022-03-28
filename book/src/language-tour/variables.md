# Variables

Variables are a powerful feature that is exclusive to rulex; because no regex engine offers this
functionality, variables in rulex are "inlined", i.e. substituted with their value recursively.

This means that variables don't allow recursion, because otherwise the generated regular expression
would have infinite size. But even without recursion, variables are a powerful and useful tool to
create more complex expressions.

Variables are declared with the <rulex>`let` keyword:

```rulex
let x = 'hello' | 'world';
```

The above will emit nothing, because the variable is declared, but not used. It could be used like
this:

```rulex
let x = 'hello' | 'world';
x '!'
```

This compiles to

```regexp
(?:hello|world)!
```

There can be multiple variable declarations. They can appear in any order, but the rulex expression
using the variables must come last. For example, this is _not_ allowed:

```rulex
# doesn't work!
x '!'
let x = 'hello' | 'world';
```

Declarations can depend on each other, as long as there is no cyclic dependency:

```rulex
let c = 'test';
let a = b b;
let b = c '!';

a
```

Declarations can be nested within a group; in that case, they can only be used within this group.
However, variables can be used within a group even if they were declared outside:

```rulex
let name = 'Max';
(
    let greeting = 'Hello';
    greeting ', ' name
)
greeting  # error!
```

In this example, `greeting` can't be used in the last line because it is only accessible within the
group where it was declared.

Nested declarations can have the same name as a declaration outside of the group:

```rulex
let name = 'Max';
(
    let name = 'Sophia';
    'Hello, ' name
)
' and ' name
```

This compiles to

```regexp
Hello, Sophia and Max
```
