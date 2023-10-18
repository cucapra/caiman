# Overview

Caiman assembly is fairly...unusable.  It's meant to be very precise and
low-level, but this means that it's not really meant to be human-usable outside
of proof-of-concept style testing.

We've been working on a Caiman frontend, in a project led by Mia Daniels.
Currently the frontend looks something like:

```
function class foo_class { foo }

// value spec
value foo() -> i64 {
    let x = 4. 
    let z = 10. 
    let y = (compute x z).
    returns y.
}

// timeline spec
timeline my_time(e : Event) -> Event {
    return e;
}

// spatial spec TBD

schedule foo {
    fn bar() -> &y 
      at time my_time
    {
        let x_slot := x.Prim; // primitives get Prim
        let z_slot := z.Prim;
        let y_slot := y.Call(x_slot, z_slot); // Call whatever `y` is
        return y_slot; // matches by definition
    }
}
```

The most current syntax can be found under the `frontend-two` branch that Mia
[has
maintained](https://github.com/cucapra/caiman/blob/frontend_two/caiman-test/high_level_frontend/external_cpu_wip.caiman).

# New Features

While the frontend has a strong baseline, there are a bunch of features that
need implementation to make it practically usable for non-trivial programs.  The
majority of these features should be fairly straightforward (at least
theoretically) to translate to our assembly representation, and those are the
features we'll be focusing on.  There are some other ideas that are less
straightforward, but worth considering, that I'll refer to in an "extensions"
section.

## Syntactic Details

While more of a warm-up, there have been a few syntactic changes that might make
sense given our current "presentation" of the language.  These are not strictly
necessary, but might be both an easier onboarding stage to the project and would
be "nice to have".  There are some other quibbles that we can talk about, part
of this project is design, afterall.

### Type declarations

Caiman sometimes needs explicit type names, so something like:

```
event my_event;
fence my_fence : cpu;
```

Perhaps with less keywords cause keywords are evil, but it would be nice I
think.

### Explicit Tags

Caiman has a tag system that should be made explicit.  Some of this can be
inferred (of course), but for arguments at least must be explicit:

```
schedule foo {
    fn head(x : i64-have, y : u64-need) {
        ...
    }
}
```

The current names of `have`, `need`, `none`, and `met` will...probably be
changed, so be aware of that.  We're open to suggestions!

### Remove `let`

After looking at this again, we might consider removing `let` from the
specification functions (value, timeline, spatial), both to make them more
distinct and make it clear that these are not operational changes to variables.
These specs are literally defining types, after all.

```
value foo() -> i64 {
    x = 4. 
    z = 10. 
    y = (compute x z).
    returns y.
}
```

### Explicit Spec Language

I've been making `$val` vs `$time` vs `$space` be a thing in assembly, but using
keywords in the scheduling language might make sense.  Anyway, the thing is to
make the spec function explicit.  So something like:

```
let x_slot := value.x.Prim;
let z_slot := value.z.Prim;
let y_slot := value.y.Call(x_slot, z_slot);
return y_slot;
```

Note that here the `Call` is still on `y`, which is a bit odd, but now it's
clear these are `value` types. There might be a cleverer way to do this, could
be interesting to contemplate.

### Timeline Spec

The timeline funclets should have the same syntax as value spec funclets:

timeline my_time(e : Event) -> Event {
    returns e.
}

### Spatial Spec

If it's not already added, having a spatial function added to the requirements
of a schedule is gonna be important:

```
spatial my_space(bs : HostBufferSpace) -> HostBufferSpace {
    returns bs.
}

schedule foo {
    fn bar() -> &y 
      at time my_time
      with space my_space // or whatever
    {
        ...
    }
}
```

Arguably the `space` and `time` requirements should be blocks, same as `schedule
foo`, but this gets complicated potentially?  Might be worth discussing more.

## Explicit allocation/read/copy

To the best of my knowledge, references are not distinguished in the schedule
from raw types, which is gonna be important for CPU/GPU stuff in particular, but
also just getting the types to "make sense".  This makes the typechecking and
reasoning a bit trickier for the user, unfortunately, but hopefully explication
and better error messages can help with this.

```
schedule foo {
    fn bar() -> &y 
      at time my_time
    {
        let x_cpu := alloc i64 cpu; // or whatever
        let z_cpu := alloc i64 cpu; // note that the type is deduced later
        let y_cpu := alloc i64 cpu;
        
        *x_cpu := value.x.Prim; // possibly not :=
        // OR
        x_cpu <- value.x.Prim;
        
        *z_cpu := value.z.Prim;
        *y_cpu := value.y.Call(x_slot, z_slot);
        return y_cpu; // matches by definition
    }
}
```

We also should have some operations to do memory reads and memory copies.  Reads
can probably be done directly through `*`, but a `copy` operation or function is
probably in order:

```
let x_cpu := alloc i64 cpu;
let y_cpu := alloc i64 cpu;
*x_cpu := value.x.Prim;
*y_cpu := copy x_cpu;
return *y_cpu; // note that technically we should be returning the value, not the reference
```

## Filled-out tail edges

Currently we have exactly `return` as best as I know.  We should have a few more
explicit tail edges, both to help with testing and to be explicit if needed.  As
part of this, adding a few more nodes in general might be useful if they come up
(a standard library is also worth discussing).  The main things are:

```
yield ...; // for parallel flow
jump ...; // for being explicit if needed
select ...; // we could skip this and just use `if` statements, but might be nice for testing
// continuations might be tricky here in general, maybe don't make them explicit?
// worth discussing
```

## If statements (control flow)

The main thing to add for the frontend is going to be control flow.  Caiman
frontend should ideally use more procedural-style control flow (even loops
eventually, though I suspect that's gonna take some clever design), rather than
the very IR-y CPS-style of program flow.  What I would like this to look like is
to be able to write a program something like this:

```
value max(x : i64, y : i64) -> i64 {
    test = (> x y).
    result = (if test x y).
    returns result.
}

schedule max {
    fn main(x : value.x.have ni64, y : value.y.have ni64) -> value.result.have {
        test := alloc i64 local;
        *test := value.test.Call(gtn, x, y);
        if (*test) {
            return *x;
        } else {
            return *y;
        }
    }
}
```

## Recursion (testing)

Recursion should "just work" once we have control flow, but it's important to
test.  Here's an example program to work with.  It's obviously...very verbose,
so contemplating how to make this less bad might be worthwhile:

```
value sum(x : i64) -> i64 {
    returns result.
    result = (if test rec 0).
    test = (> x 0).
    rec = (sum (- x 1)).
}

schedule sum {
    fn main(x : value.x.have ni64) -> value.result.have {
        zero := alloc i64 local;
        test := alloc i64 local;
        *zero := value.test.Second.Prim;
        *test := value.test.Call(x, zero);
        if *test {
            rec := alloc i64 local;
            one := alloc i64 local;
            m1 := alloc i64 local;

            *one := value.rec.First.Second.Prim;
            *m1 := value.rec.First.Call(x, one);
            *rec := value.rec.Call(m1);
            return *rec;
        } else {
            // might not work
            return *zero;
        }
    }
}
```

## Timeline

Given the nature of this project, I don't feel like a timeline makes a _ton_ of
sense in the current setup, since there are a bunch of features that would be
nice to have that I didn't get to here.  It's sort of "initial work, let's see
what needs changing and add stuff" now that we have a good baseline and the time
to make something more intensive.