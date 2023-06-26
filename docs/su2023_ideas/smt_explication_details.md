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
here](./smt_explication_project.md#input-format).  I'll take the load on getting
that setup in a branch once I get parsing updated, should be fairly quick I
think?

# Proposed Timeline

I've made the timeline assuming 8 weeks with some flex built in.  Extending to
10 weeks is not terribly difficult.  Note that this is a bit more concrete, but
still not...perfect.  Tried to fit in something within 5-8 hour weeks, we'll see
how it goes:

* Write a simple program in Rust (1-2 weeks)
* Setup easy-SMT and write a simple program (1-2 weeks)
* Write some fantasy examples in the syntax provided in the project (1-2 weeks)
  - For actually running these examples, we can do the RON-style thing
* Hook up the Rust representation to easy-SMT and get an output (1-2 weeks)
* Setup and test examples with one hole (1-2 weeks)
* Setup and test examples without control flow but user-defined order (1 week)
* Writeup (1 week)

## "Simple program" details:

For Rust programs, let's do three:
1. Print out the numbers from 1-n
2. Given two numbers, print out all primes between them
3. Given a string, construct a hashmap that maps from letters to how many times
   they appear in that string

For easy-SMT, let's setup the intuition: Given a list of pairs of strings, like
`[("x", "y"), ("x", "z"), ("y", "w")]`, where the first letter in a pair is
greater than the second, have Z3 give you an order of strings that satisfies
this requirement (if possible).  So this example could be `xywz`, for example,
or `xyzw`