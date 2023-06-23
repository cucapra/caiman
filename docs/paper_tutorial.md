## Notes

This writeup is intended to be read from the perspective of an engineer /
programmer using Caiman.  Still written academically and intended to be compact
(that is, I would recommend giving more "basic" examples), but should give a
reasonable practical overview of what features are in Caiman.  This document is
written in the fantastical Caiman frontend that I expect will be used in the
paper.  I've included associated full programs and (WIP) assembly programs in
the [Appendix](#appendix) for reference.

# Caiman Overview

We will work through a concrete example to understand the motivation and design
of the Caiman language.  Our example of choice is to apply two vector additions
in sequence, allocating and producing a fourth (new) vector.  This could,
written in C, might look something like:

```c
#define N 64
int* vadd2(int[N] v1, int[N] v2, int[N] v3) {
    int* temp = malloc(...);
    for (int i = 0; i < N; i++) {
        temp = v1[i] + v2[i];
    }
    // explicitly splitting up the additions
    int* result = malloc(...);
    for (int i = 0; i < N; i++) {
        result = temp[i] + v3[i];
    }
    return result; 
}
```

This example is (obviously) somewhat synthetic, but having two explicit compute
chunks is both common with more complicated operations and is helpful when
illustrating some of the descriptive power provided by Caiman.

We start by defining a constant `$N` and the value function for `vadd2`:

```
const $N 64;

value vadd2(v1 : array<i32, $N>, v2 : array<i32, $N>, 
v3 : array<i32, $N>) -> array<i32, $N> {
    tmp = (vadd v1 v2).
    result = (vadd tmp v3).
    returns result.
}
```

Function headers look like a "standard" imperative language, here we take in
three arguments (arguments must be typed) and provide a return type.  We do need
to state that this is a `value` function, named `vadd2`.  Arrays in Caiman must
be fixed-length, though the length can be a compile-time constant like `$N`.

The body of this function is a bit more unusual, as we use a logic-like syntax
to represent operations.  Value-language statements are unordered, so this
function body (including the `returns` statement) can be rearranged with no
change to the specification.

## Equivalence Classes

We next provide two definitions of vector addition, unified under the
equivalence class `vadd`:

```
function vadd(array<i32, $N>, array<i32, $N>) -> array<i32, $N>;

external-cpu[impl vadd] vadd_extern;

value[impl default vadd] local_vadd(v1 : array<i32, $N>, v2 : array<i32, $N>) {
    rec = (vadd (tail v1) (tail v1)).
    val = (+ (head v1) (head v2)).
    result = (if (empty v1) [] (append rec val)).
    returns result.
}
```

To unpack this code in more detail, there are three declarations happening.

First, the function class `vadd` is being declared (we use the term "function
class" to emphasize this is not proven to be an equivalence class, and the user
simply is declaring anything in it is equivalent).  When calling a function in a
local value function, we must call a function class rather than a raw value
function, which is why `vadd2` calls `vadd` and not `vadd_local` (indeed, the
value function alone is not aware of the distinction between a local and
external function).

Second, the external function `vadd_extern` is declared to live on the CPU, and
has the same argument types as the equivalence class `vadd`, which it
implements.  Finally, the local value function `local_vadd` is also declared to
be a member of that equivalence class.

Local value functions must implement an equivalence class.  If there is explicit
`[impl]` block, then the compiler creates an equivalence class of that name and
only that value function as a member.  If `default` is included in the `impl`
block, as is done here, then the compiler will assume that a function call to
this equivalence class uses the `default`.

Within the local value function itself, we observe that there are a few more
introduced functions, mostly meant to break down an array.  The code here
resembles that of a declarative functional program, breaking down the array
recursively (with `head` and `tail`) until the input is empty and the function
can return.  

Note that, while the recursion is explicitly stated, this function calls the
equivalence class (as usual), meaning that the scheduler can choose whether the
recursion should instead use, say, an external (a decision that can be made
dynamically).  Note also that, while the recursion explicitly comes before the
branching logic of `if`, the declarative and unordered nature of the value
function means that the recursion need not be called, thus allowing an
implementation of this function to terminate.

## Scheduling VAdd2

The schedule for `vadd2` resembles imperative code much more than the
declarative value functions:

```
schedule vadd2 {
    fn vadd2_extern(
    v1_ref : &array_cpu<i32, $N>, 
    v2_ref : &array_cpu<i32, $N>, 
    v3_ref : &array_cpu<i32, $N>) -> &array_cpu<i32, $N> {
        let tmp_ref <- vadd_extern[tmp](v1_ref, v2_ref);
        let result_ref <- vadd_extern[result](tmp_ref, v3_ref);
        return result_ref;
    }
}
```

The syntactic boilerplate pieces are formally non-interesting.  We create a
schedule block, indicating that the member schedules are implementing the
schedule `vadd2`.  We define our function `vadd2_cpu` with three `array_cpu`
arguments and the same size return type (note that the syntax `_cpu` is a
convention and not meaningful; the declaration of the `array_cpu` type is not
shown here for space reasons).

The body of this schedule is fairly straightforward imperative code.  There are,
however, a few details of note.  First, the use of `<-` is meant to emphasize
the imperative nature of the schedule (compared to the declarative `=` used in
the value function).  Second, each operation is associated with a value function
variable, such as `[tmp]` and `[result]` -- this amounts to giving these
operations an explicit type from the value function.  Finally, function calls
are to functions rather than equivalence classes (in this case external
functions, but we will see calls to local schedule functions as well), with
references as arguments, since the data lives on the cpu rather than in the
local schedule function.

## Scheduling VAdd

In our implementation of `vadd2_extern`, we relied on a completely external
definition for vector addition, namely `vadd_extern`.  While this is valid
Caiman code, it is not ideal for providing the decomposability that makes the
Caiman typechecker powerful.  

To be able to reason about `vadd` with the Caiman typechecker (and thus have the
ability to control heterogeneity and synchronizing properties), we should
instead call a Caiman scheduling function.  Fortunately, such a function
definition need not be at all more complicated than the external definition,
outside of some slightly differing boilerplate:

```
schedule vadd2 {
    fn vadd_cpu(v1_ref : &array<i32, $n>, 
    v2_ref : &array<i32, $n>) -> &array<i32, $n> {
        ???;
    }
}
```

This definition, consisting only of `???`, provides a complete valid Caiman
schedule for `vadd_cpu`.  `???` is a precise syntactic construction in Caiman
schedules, indicating a hole which the compiler may replace with any number of
scheduling instructions (including potentially creating a new schedule
function).  This replacement, called explication, will be discussed in more
detail in the explication section of the paper.  For reference, the compiled
result of explicating this schedule can be found in the appendix.

While this function can be explicated and run, the point of using Caiman is that
now we can decompose this `???` that we just defined, allowing us to specify the
body of this function beyond trusting the compiler to work it out.  This will
allow for the performance exploration introduced earlier, but for now, let's
break down this particular function more explicitly.

To break down the explication into concrete code, we start by allocating an
array to store the result of adding `v1_ref` and `v2_ref`:

```
let new_arr_ref <- new_arr(&array<i32, $n>, $N);
```

Since we don't want to allocate for each level of recursion, now we need to
perform some trickery (with the help of the caiman typechecker).  In particular,
while the value function type definition for `vadd` uses a fairly standard
declarative approach to constructing a list (recursing to the base and building
the list piece-by-piece), the caiman type system allows us to rewrite this
concretely in a more imperative style, specifically by reversing the memory
operation to split rather than allocate.

What this looks like in practice is through calling a recursive helper function:

```
let result_ref <- result[vadd_rec_cpu]
    (v1_ref, v2_ref, new_arr_ref);
return result_ref;
```

Now we will declare this helper function, which is also within the `schedule`
block for `vadd` (information needed for helping the typechecker ensure we did
the operation we promised we would do):

```
fn vadd_cpu_rec(
v1_ref : &array<i32, $n>, 
v2_ref : &array<i32, $n>,
result_ref: &array<i32, $n>) -> &array<i32, $n> {
    ???;
}
```

# Appendix

vadd2.caiman full program:

```
const $N 64;

value vadd2(v1 : array<i32, $N>, v2 : array<i32, $N>, 
v3 : array<i32, $N>) -> array<i32, $N> {
    tmp = (vadd v1 v2).
    result = (vadd tmp v3).
    returns result.
}

function vadd(array<i32, $N>, array<i32, $N>) -> array<i32, $N>;

external-cpu[impl vadd] vadd_extern;

value[impl default vadd] local_vadd(v1 : array<i32, $N>, v2 : array<i32, $N>) {
    rec = (vadd (tail v1) (tail v1)).
    val = (+ (head v1) (head v2)).
    result = (if (empty v1) [] (append rec val)).
    returns result.
}

schedule vadd2 {
    fn vadd2_extern(
    v1_ref : &array<i32, $N>, 
    v2_ref : &array<i32, $N>, 
    v3_ref : &array<i32, $N>) -> &array<i32, $N> {
        let tmp_ref <- vadd_extern[tmp](v1_ref, v2_ref);
        let result_ref <- vadd_extern(tmp_ref, v3_ref);
        return result_ref;
    }

    fn vadd2_cpu(
    v1_ref : &array<i32, $N>, 
    v2_ref : &array<i32, $N>, 
    v3_ref : &array<i32, $N>) -> &array<i32, $N> {
        let tmp_ref <- vadd_cpu[tmp](v1_ref, v2_ref);
        let result_ref <- vadd_cpu(tmp_ref, v3_ref);
        return result_ref;
    }
}

schedule vadd {
    fn vadd_cpu(v1_ref : &array<i32, $n>, 
    v2_ref : &array<i32, $n>) -> &array<i32, $n> {
        // allocate
        let new_arr_ref <- new_arr(&array<i32, $n>, $N);
        let result_ref <- result[vadd_rec_cpu]
            (v1_ref, v2_ref, new_arr_ref);
        return result_ref;
    }

    fn vadd_cpu_rec(
    v1_ref : &array<i32, $n>, 
    v2_ref : &array<i32, $n>,
    result_ref: &array<i32, $n>) -> &array<i32, $n> {
        let length_ref <- alloc-cpu u32;
        let zero_ref <- alloc_cpu u32; 
        length_ref <- length(v1_ref);
        zero_ref <- const_cpu(0, u32);
        if (eq_cpu(length_ref, zero_ref)) {
            result_ref <- result_ref; // satisfies the empty list
        } else {
            let v1_head;
            let v2_head;
            v1_head, v1_ref <- split(v1_ref);
            v2_head, v2_ref <- split(v2_ref);

            let val_ref = alloc_cpu i32;
            val_ref <- val[add_cpu](v1_head, v2_head);
            result_ref <- rec[vadd_cpu_rec](v1_ref, v2_ref, result_ref);
        }
        return result_ref;
    }
}
```

vadd2.cair full program:

```
```