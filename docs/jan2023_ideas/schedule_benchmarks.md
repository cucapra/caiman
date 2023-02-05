The idea here is to expand and develop benchmarks for Caiman, ideally based on
existing benchmark suites of course.  I won't link any (though I can do some
initial research) since I simply don't know what to focus on, that's part of the
project!

Broadly we will want computational benchmarks, though the initial stages of
those I will be developing myself of course.  Very briefly, I intend to find and
implement a stock-standard benchmark within the next month, and this project
will be to add more benchmarks to help with narrowing down the application of
scheduling (and when it matters).

Some sample program spaces we might be interested in finding benchmarks for:
1. baseline CUDA benchmarks (probably what I'll be using)
2. broadly scientific computing and modelling benchmarking
3. parallel programming benchmarks (maybe?)
4. checking out what, say, futhark or openCL use for benchmarking, and comparing
5. baseline graphics benchmarks

Overall this list feels vague, there does need to be work done on narrowing
program space and what would be worth focusing on.

To narrow on technical details a bit more, the goal is (since we are in
computational rather than graphics land) to simply measure time to complete the
computation.  We can use these to measure scheduling optimizations, but ideally
we can also use these benchmarks to narrow the future optimization space.

As a result, programs should be implemented fully by hand at first, or perhaps a
version with the naive scheduler.  For this project, that will really be the
bulk of the work, the hand-optimization work I'm interested in would want a few
benchmarks ideally, and those benchmarks are non-trival themselves I would
expect.  I'll include hand-writing optimizations in another proposal, in short.