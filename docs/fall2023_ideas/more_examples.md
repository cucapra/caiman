# More Examples

A more carefully made list of some examples we're missing, building from the "ground up".  In other words, taking a look at a more complete set of examples, both for raw caiman and explication, that start with just very basic variations of existing programs.

For a concrete goal, I will propose some number, like 10 "trivial examples", 3-4 "simple examples", and 1 "big example" program.  The alternative is to help develop the actual benchmarking software, but I worry about piling on extra tooling with little payoff.

## "Trivial" Examples

Written in psuedo-Rust code (basically Rust, but with `host`, `gpu`, and `cpu` attached to the types).  Note that any function definition not written can be assumed to be "external", which includes arithmetic operations like `+`.  Only functions and operations defined in the example should be implemented (for all of our sanities).

```rs
fn int_arith(x: i64-host) -> i64-host {
    // note that having no `;` means that we return
    x - 5i64 * (-3i64)
}
```

```rs
fn gpu_setup(x : i64-host) -> i64-host {
    let x_cpu = put_on_gpu(x);
    let res_cpu = gpu_call(x_cpu);
    put_on_host(res_cpu)
}

fn gpu_call(x : i64-gpu) -> i64-gpu {
    x + 1
}
```

```rs
// CPU doesn't exist yet, so skip this one
fn dispatch(x : i64-host, y : i64-host) -> i64-host {
    let x_cpu = put_on_cpu(x);
    let y_gpu = put_on_gpu(y);
    let res_cpu = cpu_call(x_cpu);
    let res_gpu = gpu_call(y_gpu);
    put_on_host(res_cpu) + put_on_host(res_gpu)
}

fn cpu_call(v : i64-cpu) -> i64-cpu {
    v + 1
}

fn gpu_call(v : i64-gpu) -> i64-gpu {
    v - 1
}
```

```rs
function_class double(i64-host) -> i64-host;

// first implementation of op
fn double_1(v : i64-host) -> i64-host {
    v + v
}

// second implemenation of op
fn double_2(v : i64-host) -> i64-host {
    2 * v
}

fn main(x : i64-host) -> i64-host {
    // spec is double(x) + double(x)
    double_1(x) + double_2(x)
}
```

```rs
fn jump(v : i64-host) -> i64-host {
    let x = v + 1;
    // returns the result of jumping to target
    // note that the result of calling is not stored here!
    jump target x 
}

fn target(v: i64-host) -> i64-host {
    v + 2
}
```

```rs
fn ref_stuff(x: &i64-host) -> i64-host {
    let y = *x + 1;
    let z = copy y;
    let w = &z;
    let out = *w + 1;
    return out;
}
```

```rs
fn yield_test(x : i64-host, y : i64-host) {
    let x_gpu = put_on_gpu(x);
    let y_gpu = put_on_gpu(y);
    let result = alloc-gpu buff;
    *result = val_gpu + 1;
}
```

## "Simple" Examples



## "Big" Examples

The "medium" examples from [examples.md](./examples.md).  Pick your favorite.