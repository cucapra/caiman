use super::*;
use crate::collections::ScopedHashMap;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntoIrError {}
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

// We treat IR funclets as immutable. Instead of modifying the existing funclets, we simply
// generate new funclets and return an updated head ID. This slightly simplifies the code
// and allows multiple pipelines to reference the same funclets safely.
// This isn't a fundamental design constraint and can be changed if necessary.

/// TODO: I think I might have to move to intra-funclet control flow
/// A "psuedo-IR node" with a concrete operation which refers to an "IR location".
/// Crucially, this location may be in a different funclet, which is why IR nodes aren't
/// emitted directly.
/// TODO: How to handle the entry funclet? We must keep its args EXACTLY the same.
/// TODO: Should we convert IdLists back into arrays here, or should we wait?
/// - advantages: it makes sense
/// - disadvantages: how do we choose between arrays and stuff now?
/// Conclusion: DO NOT convert IdLists back to arrays here.
/// Wait - but then, what's the point of the psuedo IR layer, if the locations don't actually
/// correspond? Let's just go *directly* into the IR.
/// What happens if an IDlist gets chosen as an argument? Can that even happen?
struct PsuedoIrNode {
    kind: OperationKind,
    from: egg::Id,
    deps: Box<[Location]>,
}

struct FuncletBuilder {
    /// A map from global node locations to a local location within the funclet.
    /// Clearly, this map is partial, and the local location may refer to the node itself
    /// (if it's truly local) or a Phi node (if it's passed in as an argument).
    local_memo: HashMap<Location, ir::NodeId>,
    /// Formal parameter `i` has type `input_types[i]`
    input_types: Vec<ir::TypeId>,
}
impl FuncletBuilder {
    fn add_node(&mut self, node: ir::Node) -> ir::NodeId {
        todo!()
    }
}
/// The "global location" of an IR node within the generated program.
struct Location {
    /// The ID of the funclet containing this IR node.
    funclet_id: ir::FuncletId,
    /// The ID of the IR node within the above funclet's node array.
    node_id: ir::NodeId,
}

type NodeMemo = ScopedHashMap<GraphId, Location>;

struct ElaborationCtx<'a> {
    graph: &'a Graph,
    bdoms: ir::utils::BakedDoms,
    domtree: ir::utils::DomTree,
}
impl<'a> ElaborationCtx<'a> {
    fn new(graph: &'a Graph) -> Self {
        let bdoms = graph.analysis.bake_dominators();
        let domtree = bdoms.dominator_tree();
        Self {
            graph,
            bdoms,
            domtree,
        }
    }
    fn elaborate_node(&self, memo: &mut NodeMemo, gid: GraphId) -> Location {
        // canonicalize GID, so gid1 == gid2 <=> node1 == node2
        let gid = self.graph.find(gid);
        if let Some(location) = memo.get(&gid) {
            // if this node is in a different funclet, we need to pass the value through the call
            // graph so that it arrives as an input, and then emit a param...
            // UNLESS we've already done that, in which case we can just use the cached param.
            todo!()
        }
        todo!()
    }
    fn elaborate_funclet(&self, memo: &mut NodeMemo, id: ir::FuncletId) {
        memo.push_scope();
        for &gid in self.graph.analysis.referenced(id) {
            self.elaborate_node(memo, gid);
        }
        for &next in self.domtree.immediately_dominated(id) {
            self.elaborate_funclet(memo, next);
        }
        memo.pop_scope();
    }
}

pub fn elaborate(graph: &Graph, program: &mut ir::Program) {
    let sctx = ElaborationCtx::new(graph);
    let mut memo = NodeMemo::new();
    sctx.elaborate_funclet(&mut memo, graph.analysis.head());
}
