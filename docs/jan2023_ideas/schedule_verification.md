The main idea of this project is to formalize the optimizations discovered
through benchmarking.  The key idea here is that SMT solvers should be able to
compare two schedules (if the semantics are nailed down enough) under a given
value "semantics".  So this project is building such a framework.

To give a bit more detail, this will entail formalizing exactly what schedules
can be given / writing down the formalism that Oliver has developed as SMT
terms.  Part of this work will be narrowing schedule equivalence under a value
language.  This is something I would like to focus on myself a bit more, but 1)
it's going to be a while and 2) the machinery is going to take enough work there
will be several things to do within that.

### First Steps

Since this is a broader idea, outlining the very initial work seems important.

1. Familiarize oneself with the Rust SMT bindings (re: Alexa + Adrian)

2. Lay out the semantics for equivalence (I can do a lot of this initial work)

3. Write down a trivial value equivalence checker for two schedules (keep it as
   simple as possible)
OR
3. Write down a simple program optimization and verify equivalence by hand