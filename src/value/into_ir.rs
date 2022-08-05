/*
* The egraph -> IR conversion is an adaptation of Cranelift's [scoped elaboration][].
* Most of the differences are due to differences between Cranelift's IR and ours.
*
* Cranelift only emits instructions when necessary to preserve side effects. Our only
* "side effects" are tail edges, so any value referenced by a tail edge is preserved.

* Not every value referenced by a tail edge is actually *needed*, however, and sometimes the
* values which *are* needed can be safely moved into successor funclets. We use argument deletion
* (defined as follows) to take advantage of this fact.

* Suppose we have a funclet f.
* Let S = {s1, s2, ... sn} be the set of all predessors of f (possibly including f itself.)
* Let p be a parameter of f with index i.
* Let N = { n1, n2, ... nn } where nk = sk.tail_edge.{the jump to funclet f}.args[i].
* Suppose all nodes in N are in the same eclass. Then p must take on the same value, regardless
* of how f was reached. This means that p itself should be added to the eclass of N. Then
* for each k, sk.tail_edge.{the jump to funclet f}.args[i] should be deleted.
*
* If we delete an argument, we obviously can't have any Phi nodes referencing the corresponding
* input in the generated IR. To ensure correctness we enforce that a param node has the maximum
* cost. (i.e. it will never be selected to represent its eclass unless there are no alternatives)

* We maintain the arguments via an out-of-band list of tail edges. When an argument deletion
* transformation occurs on the egraph, the rule indexes into the list to find the affected tail
* edge and "deletes" the corresponding argument by setting it to `None`. The actual indices
* remain stable until elaboration.

* The out-of-band list is also used to accomplish dead branch elimination. After each
* transformation pass, we iterate through the list and, for each branch funclet, search its'
* selector node's eclass for a constant boolean node. If one exists, then the selector can be
* constant-folded: we do this, then replace the branch funclet with a jump funclet.

* The out-of-band list is *also* used for "generalized jump threading" (I'm sure there's a
* proper name for this, but I don't know what it is.) If a funclet has only one predecessor,
* and that predecessor ends in an unconditional jump to that funclet, the funclet and its
* predecessor can be merged into a single block. Assuming that argument deletion is working
* properly it's sufficient to replace the predecessor's tail with a copy of the successor's
* tail. This can be done as part of the same iteration pass used for dead branch elimination.

* [scoped elaboration]: https://github.com/cfallin/rfcs/blob/cranelift-egraphs/accepted/cranelift-egraph.md
*/
