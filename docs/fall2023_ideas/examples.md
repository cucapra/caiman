# Example Ideas

The first step for any project we've proposed is going to be to write some
simple examples in Caiman.  Strictly speaking benchmarks in WGPU or really any
setup are still _useful_, but I think Caiman programs are gonna be the most
important/relevant to have while starting out and for us short-term.

The rest of this document outlines some Caiman programs to translate.  I've
allocated 4-6 "warmup" weeks to learn Caiman and familiarize yourself with the
assembly syntax, but it might be faster or slower depending on how much into
this you get.  For these benchmarks, I'll pretty much write the high-level code
in a Rust-ish format, we can go into more detail if needed.  I'd recommend
writing one "easy" program and one "medium" program, and we can talk about
"harder" programs if this is interesting to keep working on.

# Easy

Smaller programs to get you familiar with Caiman and have more glass-box style
testing that I haven't gotten around to (in that they "match" things that need
explicit tests to some degree):

```rs
// just some more "easy" _stuff_ to have
max(x : cpu-i64, y : gpu-i64) -> cpu-i64 {
    if x > y {
        return x; // cpu
    } else {
        return y; // on the cpu
    }
}
```

```rs
// local x, number
gpu_kernel_triangle(x : local-u64, number : local-i64) -> local-i64 {
    if x > 0 {
        return gpu_kernel_1(number);
    }
    else {
        return gpu_kernel_2(number);
    }
}

gpu_kernel_1(number : gpu-i64) -> gpu-i64 {
    return gpu_kernel_2(number) + gpu_kernel_2(number);
}

gpu_kernel_2(number : gpu-i64) -> gpu-i64 {
    return number + 1;
}
```

```rs
// recursion: all cpu for simplicity
collatz(x : u64) -> u64 {
    if x <= 1 {
        return 1;
    }
    if x % 2 == 0 {
        return collatz(x / 2) + 1;
    }
    return collatz(3 * x + 1) + 1;
}
```

```rs
// gpu (or cpu) buffer testing
sum_buffer(buff : gpu_buffer_ref<i64>, size: u64) -> gpu-i64 {
    if size <= 0 {
        return size;
    }
    (num, buff) = next(buff);
    return num + sum_buffer(buff, size - 1);
}
```

```rs
// fence testing
parallel_sum(x : i64, y : i64, z : i64) -> i64 {
    // super fake types, but hopefully clear in intent
    left = Box<i64>(0);
    right = Box<i64>(0);
    // this isn't real
    thread = thread_create();
    // the two foos here must be done in parallel
    if thread {
        *left = foo(x, y);
    } else {
        *right = foo(y, z);
    }
    merge(thread);
    return *left + *right;
}

foo(x : i64, y : i64) -> i64 {
    return x * y;
}
```

## Medium

These programs will be written at a descriptive level, translating them is some
of the work.

### Dot Product

Defined [here](https://en.wikipedia.org/wiki/Dot_product).  Test the logic for a
scalar reduction on CPU and GPU.

### 3x3 Matrix Multiply

[Usual definition](https://en.wikipedia.org/wiki/Matrix_multiplication).  The
goal here would be to have a folded-out definition for moving explicit chunks
between devices.

### 1D Jacobian

[Classic kernel](https://en.wikipedia.org/wiki/Jacobian_matrix_and_determinant).
Useful for checking updated arrays.  C code stolen from polybench, uh, one of
them.

```c
for (int t = 0; t < timesteps; t++)
{
    for (int i = 1; i < size - 1; i++)
    {
        B[i] = 0.33333 * (A[i-1] + A[i] + A[i + 1]);
    }
    
    for (int j = 1; j < size - 1; j++)
    {
        A[j] = B[j];
    }
}
```