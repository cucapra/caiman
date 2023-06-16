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

value vadd2(
v1 : array<i32, $N>, 
v2 : array<i32, $N>, 
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
change to the semantics.



# Appendix

vadd2.caiman full program:

```
const $N 64;

value vadd2(v1 : array<i32, $N>, v2 : array<i32, $N>, 
v3 : array<i32, $N>) -> array<i32, $N> {
    tmp = vadd(v1, v2).
    result = vadd(tmp, v3).
    returns result.
}

function vadd(array<i32, $N>, array<i32, $N>) -> array<i32, $N>;

external-cpu[impl vadd] extern_vadd;

value[impl default vadd] vadd_imp(v1 : array<i32, $N>, v2 : array<i32, $N>) {
    rec = (vadd (tail v1) (tail v1)).
    val = (+ (head v1) (head v2)).
    result = (if (empty v1) [] (append rec val)).
    returns result.
}

ref-cpu arrc : array<i32, $N>;
ref-gpu arrg : array<i32, $N>;

schedule vadd2 {
    fn vadd2_cpu(v1_ref : arrc, v2_ref : arrc, v3_ref : arrc) -> arrc {
        let tmp_ref <- vadd_cpu[tmp](v1_ref, v2_ref);
        let result_ref <- vadd_cpu(tmp_ref, v3_ref);
        return result_ref;
    }
}

schedule vadd {
    fn vadd_cpu(v1_ref : arrc, v2_ref : arrc) -> arrc {
        // allocate
        let new_arr_ref <- new_arr(arrc, $N);
        let result_ref <- result[vadd_rec_cpu]
            (v1_ref, v2_ref, new_arr_ref);
        return result_ref;
    }

    fn vadd_cpu_rec(
    v1_ref : arrc, 
    v2_ref : arrc,
    result_ref: arrc) -> arrc {
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