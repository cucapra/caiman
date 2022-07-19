# Caiman Frontend Value Language (Version 2)

## Overview

In this document, I outline an updated (but still early) version of the 
Caiman core value language. I primarily focus on its syntax, with some 
trivial semantics included.

The general plan for Caiman's frontend design has been to create two 
languages: a core value language, and a scheduling language. 
This document focuses on changes to the value language, and can be seen 
as a partial follow-up to the document that originally outlined 
the two languages, `docs/frontend.md`. 

The initial design of the value language was a simple declarative language
that has no awareness of the scheduling language. The latter details when and 
with which device each computation in the former happens. The value language
will remain unaware of scheduling and device use, but will work better as a
standalone language by adding state, functions, and loops. 

## Constraints Adjustment

The previous document outlined five constraints, three of which were about
the value language. Two among those are contradicted by the new design:
a lack of mutation, and avoidance of loops and recursion. Although 
state and loops will be added, the new design should still make sure that 
programmers can be aware of and keep use of these features to a minimum.
This is in the spirit of the latter constraint, which is that the 
language should "appear" declarative. 

## Syntax

```
x, y, f, l := {variables}
n := {numbers}
b := true | false
binop := {+, -, *, /, &&, ||, ==}
unop := {!, -}

program := c*

c := | if e { c } 
     | while e { c }
     | print(e);
     | let x = e;
     | let mut x = e;
     | x = e;
     | fn f(x, y, ...) { c }
     | f(e1, e2, ...);
     | return e;

e := | x | n | b | input() | e1 binop e2 | unop e 
     | f(e1, e2, ...) | { l: e }
```

The language is now slightly larger than IMP! It features the while loop,
print, and functions. It also has two extra features of note. One is mutable
variable declarations, which require a `mut` (like Rust), so that the 
programmer will be aware when they are utilizing state. The other is 
labeling expressions, which is for the scheduling language's use.

It does not include anything in particular that would make the programmer 
aware of a recursive function they were writing. We could include 
the `rec` keyword or something of the sort, but I believe that 
recurring in a function by accident is very unlikely. Similarly, 
we can assume the programmer is aware that the while loop construct 
creates a loop.

The Python-like part of this syntax, where commands are interspersed 
with functions, may be ill-fit. It was made this way because the
previous value language was, and it can be argued that it keeps the 
language in a more declarative style. However, the original value
language was meant to be implicitly contained within a loop, and the way it 
was designed was also, perhaps, for temporary simplicity. Furthermore, a lot 
of languages don't do this, specifically Rust and C, both of which this 
language aesthetically resembles. 

The syntax for labeling expressions was chosen arbitrarily, so it is also 
likely to change.

## Semantics

Of course, mutating a variable that hasn't been declared `mut` is illegal.
Recursion will also not be allowed in this first version of the
language because it is challenging to translate.

