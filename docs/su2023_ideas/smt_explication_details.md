# Tooling

Working with raw Z3 is fine, though it would be nice to embed something like
[Rust's easy-SMT bindings](https://docs.rs/easy-smt/latest/easy_smt/) to help
with communication down the line.  I've heard there are issues with these
potentially?  Might be worth fiddling with a bit.

Input/output formats should be specified ASAP.  I've ignored a huge amount
of Caiman semantics, and one of the hard problems will be translating
the...trickier semantics and dependencies in a reasonable way.

Working from the raw Caiman assembly AST would be ideal, but probably a bit
tricky.  Having a custom miniature library would probably be wise, making some
assumptions about what you get as input.  A custom language is probably
overkill, but it would not be unreasonable to make something that states order
(you could even use the UML diagram format I'm using for this document, then
you'd get pretty pictures?)

I decided to add a section to the project referencing the above thoughts in more
detail, to nail down something concrete. [Specifically
here](./smt_explication_project.md#input-format)

# Proposed Timeline

I've made the timeline assuming 10 weeks with some flex built in.  Extending to
12 weeks is not terribly difficult.  Note that this is a _very_ initial plan:

* Write some simple examples by hand / figure out input format (1-1.5 week)
* Code one or two by hand in z3 to get started (.5-1 week)
* Generalize the case of unspecified schedule with no control flow (1-2 weeks)
* Extend with arbitrary user requirements (1-2 weeks)
* Extend with schedule control flow -- could swap with the above goal (2-3
  weeks)
* Bonus extension (1-3 weeks)
* Writeup (1 week)

The bonus extensions I had in mind are one of the following to try out:

1. CPU/GPU selection heuristic
2. Expand timing rules (currently they are pretty simplistic)
3. Hook up the system to explication (likely with my help)