use super::*;
use std::collections::hash_map::{Entry, HashMap, VacantEntry};
use std::hash::Hash;
use std::rc::Rc;

/*
* The egraph -> IR conversion is an adaptation of Cranelift's [scoped elaboration][].
* Most of the differences are due to differences between Cranelift's IR and ours.

* Cranelift only emits instructions when necessary to preserve side effects. Our only
* "side effects" are tail edges, so any value referenced by a tail edge is preserved.

* Not every value referenced by a tail edge is actually *needed*, however, and sometimes the
* values which *are* needed can be safely moved into successor funclets. We use argument inlining
* (defined as follows) to take advantage of this fact.

* Suppose we have a funclet f.
* Let S = {s1, s2, ... sn} be the set of all predessors of f (possibly including f itself.)
* Let p be a parameter of f with index i.
* Let N = { n1, n2, ... nn } where nk = sk.tail_edge.{the jump to funclet f}.args[i].
* Suppose all nodes in N are in the same eclass. Then p must take on the same value, regardless
* of how f was reached. This means that p itself should be added to the eclass of N. Then
* for each k, sk.tail_edge.{the jump to funclet f}.args[i] should be "deleted" (marked as inlined).

* If we inline an argument, we obviously can't have any Phi nodes referencing the corresponding
* input in the generated IR. To ensure correctness we enforce that a param node has the maximum
* cost. (i.e. it will never be selected to represent its eclass unless there are no alternatives)

* We maintain the arguments via an out-of-band list of tail edges. When an argument inlining
* transformation occurs on the egraph, the rule indexes into the list to find the affected tail
* edge and "deletes" the corresponding argument by setting it to `None`. The actual indices
* remain stable until elaboration.

* The out-of-band list is also used to accomplish:
*   - **Dead branch elimination.** After each transformation pass, we iterate through the list and,
*       for each branch funclet, search its' selector node's eclass for a constant boolean node.
*       If one exists, then the selector can be constant-folded: we do this, then replace the
*       branch funclet with a jump funclet.
*   - **Redundant branch elimination.** This occurs when both edges out of a branch go to the
*       same funclet (and the arguments are equivalent.) This isn't too useful on it's own,
*       since people don't typically write useless branches, but can help "clean up" other
*       transformations which render branches redundant.
*   - **"Generalized jump threading."** (I'm sure there's a proper name for this, but I don't know
*       what it is.) If a funclet has only one predecessor, and that predecessor ends in an
*       unconditional jump to that funclet, the funclet and its predecessor can be merged into a
*       single block. Assuming that argument inlining is working properly it's sufficient to replace
*       the predecessor's tail with a copy of the successor's tail. You can think of this as
*       inlining, but on the basic block level rather than the function level.

* One final note: before I said that any value referenced by a tail edge is preserved because
* tail edges are considered side effects. That's true, but we don't count inlined arguments
* as referenced. (This subsumes unused funclet input/output elimination.)
* [scoped elaboration]: https://github.com/cfallin/rfcs/blob/cranelift-egraphs/accepted/cranelift-egraph.md
*/

enum ShmRecord<K> {
    Scope,
    Insert(Rc<K>),
    Replace(Rc<K>),
}
struct ShmValue<V> {
    value: V,
    depth: usize,
}
pub struct ScopedHashMap<K, V> {
    /// The actual hashmap. Keys are reference countied to avoid potentially costly `clone`-s,
    /// since keys must live in both the hashmap and the `keys` structure.
    hashmap: HashMap<Rc<K>, ShmValue<V>>,
    /// A journal of the actions performed. This allows us to "rewind" when a scope is popped and
    /// undo all the changes we made in the reverse order.
    journal: Vec<ShmRecord<K>>,
    /// A stack of replaced values.
    replaced: Vec<ShmValue<V>>,
    /// Tracks the current depth. This is used to optimize replacing values at the current depth.
    /// The user doesn't expect to recover the old values so they can be discarded.
    depth: usize,
}
impl<K: Eq + Hash, V> ScopedHashMap<K, V> {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            journal: Vec::new(),
            replaced: Vec::new(),
            depth: 0,
        }
    }
    pub fn push_scope(&mut self) {
        self.journal.push(ShmRecord::Scope);
        self.depth += 1;
    }
    /// # Panics
    /// Panics if no scope has been pushed.
    pub fn pop_scope(&mut self) {
        loop {
            match self.journal.pop().expect("no scopes") {
                ShmRecord::Scope => return,
                ShmRecord::Insert(k) => {
                    let removed = self.hashmap.remove(&k);
                    assert!(removed.is_some(), "inserted key not in map");
                }
                ShmRecord::Replace(k) => {
                    let val = self.hashmap.get_mut(&k).expect("replaced key not in map");
                    *val = self.replaced.pop().expect("out of sync w/ journal");
                }
            }
        }
        self.depth -= 1;
    }
    pub fn get(&self, key: &K) -> Option<&'_ V> {
        self.hashmap.get(key).map(|sv| &sv.value)
    }
    pub fn insert(&mut self, key: K, value: V) {
        let key = Rc::new(key);
        let mut value = ShmValue {
            value,
            depth: self.depth,
        };
        match self.hashmap.entry(Rc::clone(&key)) {
            Entry::Occupied(mut existing) => {
                std::mem::swap(&mut value, existing.get_mut());
                // replaced within same depth: no expecation that old val is saved,
                // so no action needs to be taken
                if value.depth != self.depth {
                    self.replaced.push(value);
                    self.journal.push(ShmRecord::Replace(key));
                }
            }
            Entry::Vacant(spot) => {
                spot.insert(value);
                self.journal.push(ShmRecord::Insert(key));
            }
        }
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
}

type NodeMemo = ScopedHashMap<Node, ()>;
fn codegen_funclet(id: ir::FuncletId, memo: &mut NodeMemo) {
    todo!()
}
fn elaborate_funclet(id: ir::FuncletId, memo: &mut NodeMemo, domtree: &ir::utils::DomTree) {
    memo.push_scope();

    codegen_funclet(id, memo);
    for &next in domtree.immediately_dominated(id) {
        elaborate_funclet(next, memo, domtree);
    }

    memo.pop_scope();
}
pub fn the_thing_that_generates_code(graph: &GraphInner) {
    let bdoms = graph.analysis.bake_dominators();
    let domtree = bdoms.dominator_tree();
    let mut memo = NodeMemo::new();
    elaborate_funclet(graph.analysis.head(), &mut memo, &domtree)
}
