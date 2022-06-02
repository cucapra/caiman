# Caiman Frontend Design

## Overview

The objective of this document is to outline a Caiman frontend design initial proposal.  In particular, this document outlines the core and scheduling language syntax and semantics for Caiman and how they may be able to interact.  At the center of this interaction is that there are essentially two languages: the line-by-line value language and the scheduling language associated with managing fences and synchrony.

The value language here operates on data directly, providing a series of expressions to assign to variables.  The value language is fully declarative in the sense that there is no notion of time or mutation.  The scheduling language, on the other hand, has basic control flow and the notion of time, thus allowing the user to specify when things happen within the value language.  These two languages structured so that the value language should have no awareness of the scheduling language, but the scheduling language can use the value language.

The focus of this document, perhaps surprisingly, is on the syntax of these languages.  The reason for this is that _how_ the user expresses scheduling constraints is important how the user thinks about these constraints and, as a result, how Caiman is perceived.  The design of the front-end, in some sense, is very much syntax-first as a result, with some understanding of how the semantics are done based on the backend-design document already written.

### Constraints

We start with an overview of the constraints needed to be satisfied by the language, to give some intuition of the challenges faced when designing the frontend.  In particular, these constraints are incompletely listed as follows:

1. A lack of mutation (can be interpreted as SSA form)
2. Avoidance (for initial design) of inclusion of loops and recursion, except for built-in rendering loops -- a total language
3. Scheduling should be unambiguous and able to be resolved statically when possible
4. Scheduling must be based around existing Caiman implementation when possible to minimize compiler engineering
5. The language ought to "appear" declarative per the guarantees of Caiman

In some sense, these constraints are fairly straightforward, particularly since many of them are what we should _not_ include.  However, resolving this design requires some thought around how, particularly, to manage scheduling and make progress around managing the need for both scheduling and a lack of static ambiguity during transformations.

## Syntax

The syntax of Caiman's front end will be divided into two "parts": the core value language and the scheduling language.  Each of these are distinct (can be thought of as living in separate files), with the intention that they cannot be interleaved, at least initially.  It is perhaps worth giving some thought to how they could be interleaved going forward, and how to minimize programmer effort when specifying a schedule.

### Value Language

The syntax of the value language is as follows:

```
x := {variables}
n := {numbers}
binop := {+,-,*,/,&&,||,==}
unop := {!, -}

program := c
c := c1;c2 | let x = e | if (e) {c1;} c2
e := x | n | true | false | input() | e1 binop e2 | unop e
```

This value language description should look extremely similar to IMP with only one notable exception: the lack of a loop operation (note that recursion will be semantically disallowed), which was discussed as part of constraints.

Finally, it is worth noting that, while loops and recursion are disallowed, a control rendering loop is assumed to wrap code appropriately.  It may be necessary to include `prelude`, `loop`, and `exit` blocks to specify behavior around this core render loop, but that can be resolved later.

### Scheduling

The scheduling language for Caiman should be relatively simple, but syntactically distinct from the core value language to minimize confusion.  As a result, we define the following syntax:

```
x := {variables}

program := b
b := CPU {c;} b | GPU {c;} b | b
c := c1;c2 | x | --- x | sync(x) | if (x1) {c1} c2 | print(x)
```

The objective of this syntax is to be both minimal and unambiguous.  All operations must be in either CPU or GPU "blocks", and a program is simply a series of blocks.

Commands within a block are temporal and consist of one of four operations, namely caching a variable, printing a stored variable, creating a fence at the given point in time with `---`, or syncing to a given fence.  The notion of time is unambiguous -- every assignment made before a given fence is required to happen before that fence at some point, and every assignment after must happen after.  The order of these operations outside of fences is unspecified except for calls to `print`.

One important note about this minimal scheduling language is that control flow is minimized -- this is in conjunction with how the value language handles control flow, but is instead focused on, well, scheduling.  The other primary observation to consider is that the variables used in the scheduling language cannot be defined -- indeed, all variables assigned really ought to be variables on different devices.  It's not immediately clear to me how this should be checked, if at all, so something to think on.

The second note is the inclusion of the `sync` operation.  Tthe goal is to synchronize on a fence defined elsewhere in the scheduling language.  These fences are treated syntactically as just variables syntactically, though they are restricted to fence names semantically.

## High-level Semantics

Broadly speaking, the semantic flow of this language is as follows: the value language defines expressions, which in turn are used by the scheduling language to describe data ownership.  It is perhaps worth noting that currently all operations and variables defined in the value language are not device specific, which may be somewhat surprising.  The rationale for this in particular is that the scheduling language together with the value language should make all device ownership unambiguous through specifying data transfers on the scheduling language end.

In particular, all variable ownership is given by whether that variable is copied from CPU to GPU or vice-versa within a scheduling block -- a variable owned by the local device should be control-flow unambiguous due to the way it's used in the value language.  If this turns out to be false, due to how the semantics of the value language need be instantiated, adding blocks to the value language is a relatively lightweight change without significant conceptual overhead.

Finally, it is worth noting that the semantics of the core language are trivially those of the simple imperative language, with the exceptions of all variables not able to be reassigned and a lack of a notion of time.

### Scheduling and Fences

To explore the details of device data movement, we consider a simple example:

```
// value Code

x = input();
y = x + 5;
z = y;
```

```
// Scheduling Code

CPU {
    x;
    --- write;
    sync(write);
}
GPU {
    y;
    --- read;
    sync(read);
}
CPU {
    z;
    print(z);
}
```

In this example, we note a few key observations.  First, the CPU and GPU code are ordered intentionally to avoid ambiguity between when the given fence operations occur -- if the block order is reversed, this code should likely fail to compile due to the order of synchronizations.

Second, the ownership of x and y is unambiguous due to the specification of writes occuring in scheduling -- namely, the writing of x to y occurs _from_ the CPU _to_ the GPU, and vice-versa on the GPU.  This is despite no device specification in the value language -- simply put, these operations need not be aware of ownership, since each variable can be deduced from the scheduling code, as has been discussed.

Finally, this code should both compile and produce the value `x+5` without error.  Note that threading is not specified anywhere, nor is the exact notion of where in the code each assignment happens, per the declarative guarantees of Caiman.  Whether this simplicity is practical remains to be seen, but should provide a reasonable starting point to defining the Caiman front end.