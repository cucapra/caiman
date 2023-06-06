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

To start 

# Appendix

mm2.caiman full program:

```
const $N 64;

value vadd2(v1 : array<i32, $N>, v2 : array<i32, $N>, 
v3 : array<i32, $N>) -> array<i32, $N> {
    tmp := @vadd(v1, v2).
    result := @vadd(tmp, v3).
    returns result.
}

function @vadd(array<i32, $N>, array<i32, $N>) -> array<i32, $N>;

external-cpu[impl @vadd] extern_vadd;

value[impl default @vadd] vadd(v1, v2) {
    rec := (@vadd v1.tl v2.tl).
    val := (+ v1.hd v2.hd).
    result := (if (@empty v1) [] (@append rec val)).
    returns result.
}

slot-cpu arrc : array<i32, $N>;
slot-gpu arrg : array<i32, $N>;

schedule[value $vadd2]
vadd2_cpu(v1_slot : arrc, v2_slot : arrc, v3_slot : arrc) -> arrc {
    tmp_slot = $vadd2.tmp[vadd_cpu](v1_slot, v2_slot);
    result_slot = $vadd2.result[vadd_cpu](tmp_slot, v3_slot);
    return result_slot;
}

schedule[value $vadd]
vadd_cpu(v1_slot : arrc, v2_slot : arrc) -> arrc {
    result_slot = allocate $result;
    for (i : indices(v1_slot)) {
        result_slot[i] = 
    }
    return result_slot;
}
```

mm2.cair full program:

```
```