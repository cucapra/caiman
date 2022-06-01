# Caiman Frontend Design

## Overview

The objective of this document is to outline a Caiman frontend design initial proposal.  In particular, this document outlines the core and scripting language syntax and semantics for Caiman and how they may be able to interact.  At the center of this interaction is that there are essentially two languages: the line-by-line operational language and the scheduling language associated with managing fences and synchrony.

The focus of this document, perhaps surprisingly, is on the syntax of these languages.  The reason for this is that _how_ the user expresses scheduling constraints is important how the user thinks about these constraints and, as a result, how Caiman is perceived.  The design of the front-end, in some sense, is very much syntax-first as a result, with some understanding of how the semantics are done based on the backend-design document already written.

### Constraints

We start with an overview of the constraints needed to be satisfied by the language, to give some intuition of the challenges faced when designing the frontend.  In particular, these constraints are incompletely listed as follows:

1. A lack of variable state (can be interpreted as SSA form)
2. Avoidance (for initial design) of inclusion of loops and recursion, except for built-in rendering loops
3. Scheduling should be unambiguous and able to be untangled statically when possible
4. Scheduling must be based around existing Caiman implementation when possible to minimize compiler engineering
5. The language ought to "appear" declarative per the guarantees of Caiman

In some sense, these constraints are fairly straightforward, particularly since many of them are what we should _not_ include.  However, resolving this design requires some thought around how, particularly, to manage scheduling and make progress around managing the need for both scheduling and a lack of static ambiguity during transformations.

## Syntax

The syntax of Caiman's front end will be divided into two "parts": the core operational language and the scheduling language.  Each of these are distinct (can be thought of as living in separate files), with the intention that they cannot be interleaved, at least initially.  It is perhaps worth giving some thought to how they could be interleaved going forward, and how to minimize programmer effort when specifying a schedule.

### Core Language

The syntax of the core operational language is as follows:

```
x := {variables}
f := {functions}
n := {numbers}
binop := {+,-,*,/,&&,||,==}
unop := {!, -}

program := c;EOF
c := c1;c2 | let x = e | print(e) | f(e1, ...) | if (e) {c1;} c2 | fn f(e1, ...) {c1;} c2 | sync(x)
e := x | n | true | false | f(e1, ...) | input() | e1 binop e2 | unop e
```

This core language description should look extremely similar to IMP with two notable exceptions.  The first, as discussed with respect to constraints, is the lack of a loop operation (note that recursion will be semantically disallowed).  The second is the inclusion of the `sync` operation.  This will be discussed in more detail in our discussion of the scheduling language, but essentially the goal is to synchronize on a fence defined in the scheduling language.  These fences are treated syntactically as just variables, though general expressions in `sync` are syntactically disallowed for simplicity.

Finally, it is worth noting that, while loops and recursion are disallowed, a control rendering loop is assumed to wrap code appropriately.  It may be necessary to include `prelude`, `loop`, and `exit` blocks to specify behavior around this core render loop, but that can be resolved later.

### Scheduling

The scheduling language for Caiman should be relatively simple, but syntactically distinct from the core operational language to minimize confusion.  As a result, we define the following syntax:

```
x := {variables}

program := b | EOF
b := CPU {c;} b | GPU {c;} b | b EOF
c := c1;c2 | x1 <- x2 | fence x
```

The objective of this syntax is to be both minimal and unambiguous.  All operations must be in either CPU or GPU "blocks", and a program is simply a series of blocks.

Commands within a block are temporal and consist of one of two operations, namely assigning one variable to another or creating a fence at the given point in time.  The notion of time is unambiguous -- every assignment made before a given fence is required to happen before that fence at some point, and every assignment after must happen after.  The order of these operations outside of fences is unspecified.

One important note about this minimal scheduling language is that control flow is not allowed -- instead, control flow is managed by the core operational language through synchronization operations.  The other primary observation to consider is that the variables used in the scheduling language cannot be defined -- indeed, all variables assigned really ought to be variables on different devices.  It's not immediately clear to me how this should be checked, if at all, so something to think on.

## High-level Semantics

Broadly speaking, the semantic flow of this language is as follows: the operational language defines control flow, which uses synchronization of fences, which in turn are defined by the scheduling language to include some number of data transfers.  It is perhaps worth noting that currently all operations and variables defined in the operational language are not device specific, which may be somewhat surprising.  The rational for this in particular is that the scheduling language together with the operational language should make all device ownership unambiguous through specifying data transfers on the scheduling language end.

In particular, all variable ownership is given by whether that variable is copied from CPU to GPU or vice-versa within a scheduling block -- a variable owned by the local device should be control-flow unambiguous due to the way it's used in the operational language.  If this turns out to be false, due to how the semantics of the operational language need be instantiated, adding blocks to the operational language is a relatively lightweight change without significant conceptual overhead.

Finally, it is worth noting that the semantics of the core language are trivially those of the simple imperative language, with the exceptions of all variables not able to be reassigned and that synchronization operations specify data movement between devices.

### Scheduling and Fences

To explore the details of device data movement, we consider a simple example:

```
// Operational Code

x = 3;
sync(write);
y = y + 5;
sync(read);
print(x);
```

```
// Scheduling Code

CPU {
    y <- x;
    fence write;
}
GPU {
    x <- y;
    fence read;
}
```

In this example, we note a few key observations.  First, the CPU and GPU code are ordered intentionally to avoid ambiguity between when the given fence operations occur -- if the block order is reversed, this code should likely fail to compile due to the order of synchronizations.

Second, the ownership of x and y is unambiguous due to the specification of writes occuring in scheduling -- namely, the writing of x to y occurs _from_ the CPU _to_ the GPU, and vice-versa on the GPU.  This is despite no device specification in the operational language -- simply put, these operations need not be aware of ownership, since each variable can be deduced from the scheduling code, as has been discussed.

Finally, this code should both compile and produce the value `8` without error.  Note that threading is not specified anywhere, nor is the exact notion of where in the code each assignment happens, per the declarative guarantees of Caiman.  Whether this simplicity is practical remains to be seen, but should provide a reasonable starting point to defining the Caiman front end.