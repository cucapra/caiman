# Caiman Scheduling Theory

## Overview

At the time of this writing, Caiman is in a state where it has a somewhat working backend, enough codegen to run (simple) examples, and working middle-end translations.  What this state of model does not have in any reasonable sense, however, is the ability to reason about actual scheduling decisions.  In particular, Caiman currently finds the simplest solution to resolving scheduling, in that each node attempts to connect to each other node without making an significant decisions.

What this document outlines is a proposal and timetable to realize these scheduling decisions practically.  This document is intended to be an incomplete and short overview, more of a project proposal than a serious dive into the technical decisions that need be made.  As a result, while some technical details are included, the majority of the document outlines what to think about when imagining this project from a high level rather than diving into how to actually implement these ideas. 

### Proposal

The primary objective of this project is to find a general solution (with potential annotation help) to the problem of scheduling a series of time-independent operations using a minimal number of operations, syncs, and fences.  The precise heuristic for which insturctions to use and where to move operations is dependent on some cost model, which will remained broadly undefined for the moment.  In particular, whether to favor fewer fences for more instructions, for example, will be left as a user-defined feature as much as possible.

Practically, given the divide between the scheduling language and value language defined in frontend.md, we are interested in primarily the value language alone (with the scheduling language serving both as a set of annotations and the target of generation).  More concretely, given the following value-language code, what schedule ought we derive?

```
b = input();
x = input();
y = input();
if (b > 0) {
    z = x + y;
}
else {
    z = x - y;
}
```

Given no other information, it would, of course, be perfectly reasonable to derive the schedule:

```
// Naive schedule
CPU {
  b;
  x;
  y;
  z;
}
```

However, suppose the user requires that the calculation of z be on the GPU:

```
// User schedule
GPU {
    z;
}
```

Then, while b, x, and y clearly need be on the CPU (due to requiring user input), we can improve the schedule slightly by processing the potential z operations in parallel:

```
// Calculated schedule
CPU {
    b;
    x;
    y;
    --- write
    sync(write);
}
GPU {
    if (b) {
        z;
    }
    else {
        z'; // we use z' to denote an alternate possible value for z -- this may not be very...general
    }
}
```

How this scheduling should be done for more complicated annotations and value algorithms (and what should be disallowed) is the main open question here, though part of this work relies on extending the existing Caiman backend to support loops.

## Calendar Outline

Here we outline a rough plan of what a 12-week summer of research could look like and roughly what to hope to produce:

1-2  - review Caiman backend, value, and scheduling language
3-4  - develop core AST tools from the frontend / communicate about frontend design
5-6  - may overlap with previous, but design scheduling algorithm and research necessary tools
7-10 - design and implement the resulting middle-end algorithm
11   - implement codegen for backend, may need more time
12   - writeup results

## Technical Details

A few important notes about the scheduling algorithm which needs to be derived:

1. Syncs and fences need be inserted optimally when possible -- specifically, a sync should never sync on a fence that comes before an already-synced fence
2. If an annotation is ambiguous or impossible, the algorithm should produce a compile-time error, either in the middle-end or backend
3. Order of value operations can be rewritten except for fences to optimize data dependencies (using the usual dataflow graph)
4. An existing scheduling algorithm would make a lot of sense here, and seems a reasonable starting point

To help motivate this project a bit more, we consider the more complicated cost-model based problem:

```
// Initial value code

x = input();
y = input();
z = input();
a = x + y;
b = x * y;
c = x - z;
d = y / z;
```

Here we have 4 calculations derived from 3 inputs.  Again, this would be straightforward without any CPU/GPU constraints, but let's suppose the user specifies they want a and b calculated on the GPU but c and d calculated on the GPU, such that a and b are calculated before c and d.  Now we also suppose that the user specifies that a be printed _before_ d is calculated, creating an implicit fence.  In other words, we have the following schedule code:

```
// User Schedule

GPU {
    a;
    b;
}
CPU {
    c;
    print(a); // note that we must print on the CPU
    d;
}
```

Clearly, this schedule requires that we workout x, y, and z before invoking any of the above, but we can do a bit better.  In particular, we would like to sync the CPU and GPU only twice -- once to send the information of x and y to the GPU and once to recover the value of a from the GPU.  This "optimized" schedule would look something like the following:

```
// Optimized Schedule

CPU {
    x;
    y;
    --- write;
    sync(write);
}

GPU {
    a;
    ---read;
    b;
    sync(read);
}

CPU {
    c;
    print(a);
    d;
}
```

It is perhaps interesting that the structure of the code barely changed, and operations were only inserted.  Whether this is generally the case is unclear to me.  Also, I noted this as being "optimized" with quotation marks precisely because it's so difficult to tell if the arrangement of fences and synchronizations are ideal -- motivating even further the need for compiler help.