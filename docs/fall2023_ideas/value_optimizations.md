# Project

Caiman-style explication relies on a fixed set of specification functions.  Once
the value language definitions are locked, they can't change, even if there are
"obvious problems" like dead code or duplicate calls.

These duplications can cause problems, both in terms of performance (obviously)
and in terms of errors!  If you have duplicate definitions, the explicator can
be borked depending on the order it resolves those definitions (this is a bug,
to be clear, that occured due to lack of design time to fix it, but there could
be others lurking.)

The actual optimizations to be done are "non-interesting" in that they are
literally E-graph optimizations, but the mechanics of implementing this and
making it work with value functions (or more likely the frontend) require some
engineering work and creativity.

# Examples

For the purpose of having some concrete examples, I've written some extremely
boring value funclets in Caiman assembly (untested), along with the "corrected"
version below:

## Unused Code

```
// Original
value[impl default @main] %orig() -> i64 {
    %x = constant 4i64;
    %y = constant 5i64;
    return %x;
}

// Optimized
value[impl default @main] %opt() -> i64 {
    %x = constant 4i64;
    return %x;
}
```

## CSE

```
// Original
value[impl default @main] %orig() -> i64 {
    %one = constant 1i64;
    %two = @add(%one, %one);
    %two_two = @add(%one, %one);
    %four = @add(%two, %two_two);
    return %four;
}

// Optimized
value[impl default @main] %opt() -> i64 {
    %one = constant 1i64;
    %two = @add(%one, %one);
    %four = @add(%two, %two);
    return %four;
}
```

If you wanna reason about basic properties, you can also do stuff like:

```
// Original
value[impl default @main] %orig() -> i64 {
    %one = constant 1i64;
    %two = @add(%one, %one);
    %three = @add(%one, %two);
    %four = @add(%one, %three);
    return %four;
}

// Optimized
value[impl default @main] %opt() -> i64 {
    %one = constant 1i64;
    %two = @add(%one, %one);
    %four = @add(%two, %two);
    return %four;
}
```

## Dead Select

```
// Original
value[impl default @main] %orig() -> i64 {
    %x = constant 0i64;
    %y = constant 1i64;
    %z = constant 2i64;
    %result = select %x %y %z 
    return %result;
}

// Optimized
value[impl default @main] %opt() -> i64 {
    %result = constant 2i64;
    return %result;
}
```

# State

One thing to be aware of when implementing this is being careful with your
definitions of equality, particularly wrt state. Caiman assembly is fairly coy
about this problem. Consider, for example, the following code:

```
value[impl default @main] %orig(%arr : array<i64>) -> i64 {
    %zero = constant 0i64;
    %one = constant 1i64;
    %_ = call %set %arr %zero %zero;
    %y = call %set %arr %one %one;
    %result = extract %y 0;
    return %result;
}
```

The correct thing to do here is to label `set` as impure, but if this isn't
reasoned about correctly in the reduction, considering `set` to be pure would
result in the first call being deleted.  So, ya know, test that carefully.

# Timeline

An extremely rough 16-week in-semester timeline for a project like this.  The
estimates vary wildly depending on how much we end up having to do "from
scratch", as the cost for tooling is fairly up-front while the cost for writing
things ourselves is more backloaded.

* 4-6 weeks: warmup benchmarks for Caiman (see `benchmarks.md`)
* 2-4 weeks: get egraph tooling setup / figure out what framework to use.
  There's some previous work on this topic done by Mateo that we can probably
  pull from.
* 2-4 weeks: setup testing framework with examples
* 2-8 weeks: workout local optimizations (time variance here depends on whether
  you're doing things from scratch or not)
* 1-3 weeks: clean up tests and do a writeup on technical results.