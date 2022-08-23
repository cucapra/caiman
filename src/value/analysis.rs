//! Due to egg's interface, lifetime restrictions, and efficiency concerns, there are some complex
//! global invariants which must be maintained.
//!
//! - [`ClassAnalysis`] has a [`output_types`][ClassAnalysis] field which is stored as an `Option`.
//!   However, when node unions are performed, this field **MUST** contain a value.
//!
//!   Explanation: Consider rewriting `a + b` to `a - (-b)`. This might be useful, especially when
//!   combined with other optimizations. But you're only able to negate `b` if it's signed, so it's
//!   necessary to have type information available in the rewrite stage.
//!
//!   Type information should be stored in [`ClassAnalysis`] because all nodes in the same
//!   equivalence class must have the same output type. We could force the class analysis to
//!   *always* hold a type ID. (That is, `TypeId`, not `Option<TypeId>`.)
//!
//!   The problem is that *egraph node creation* and *eclass merging* are two different steps. When
//!   we create a node as the result of a rewrite, we need to give it an associated class analysis,
//!   so we need to choose *something* for it's type ID. But the only information that `egg` gives
//!   us is the egraph as a whole and the newly created node. We don't know its "source" nodes, so
//!   we can't use them to derive the new node's type ID. (We could if we smuggled parent info via
//!   the [`Node`][super::Node] itself, but that's gross and would require a custom applier.) We
//!   could create a dummy "any" type ID which is replaced during eclass merging, but wait, that's
//!   just a worse version of `Option`.
//!
//!   There's actually nothing to worry about during rewrites. By induction we assume the invariant
//!   holds at the start of the rewrite. Then the "source" node(s) have a type ID. The "derived"
//!   node(s) are given no type ID when their node is created; the source & derived eclasses are
//!   then merged, and the source's type ID is retained in the resulting eclass.
//!
//!   Instead, the area of concern is graph creation. The invariant will hold at the time of the
//!   first rewrite *if and only if* each eclass has been assigned a type ID by that point. Don't
//!   use [`EGraph::add`](egg::EGraph) unless you've thoroughly thought it through, use
//!   [`create_node`] instead.
//!
//!   Hashconsing complicates matters since two "identical" nodes can have different type IDs... or
//!   can they? As it turns out, this is only true for constants. All other value nodes derive their
//!   types from their inputs. The "fix" is retaining type IDs in constant nodes in the graph, even
//!   though it may seem redundant with the class analysis data.
use super::*;
use crate::collections::Arena;
use crate::ir;
use constant::Constant;
use std::collections::BTreeSet;
use std::collections::HashMap;

mod validate;
pub use validate::validate;

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    /// The constant-folded value of this output, if one exists.
    pub folded: Option<Constant>,
    pub type_id: ir::TypeId,
}
impl Output {
    fn merge(&mut self, other: Self) -> egg::DidMerge {
        assert_eq!(self.type_id, other.type_id, "mismatched types");
        egg::merge_option(&mut self.folded, other.folded, |a, b| {
            assert_eq!(*a, b, "differing constants");
            egg::DidMerge(false, false)
        })
    }
}
#[derive(Debug, PartialEq)]
enum OutputSet {
    Single(Output),
    Multiple(Box<[Output]>),
}
impl OutputSet {
    fn compute(node: &Node, egraph: &Graph, type_id: ir::TypeId) -> Self {
        if let Some(folded) = node.to_constant(egraph) {
            return OutputSet::Single(Output {
                type_id,
                folded: Some(folded),
            });
        }
        match &node.kind {
            NodeKind::IdList => {
                let os = node
                    .deps
                    .iter()
                    .map(|id| {
                        egraph[*id]
                            .data
                            .single()
                            .expect("can't have an idlist of idlists")
                    })
                    .map(|o| Output::clone(o))
                    .collect::<Box<[Output]>>();
                OutputSet::Multiple(os)
            }
            NodeKind::Param { funclet_id, index } => {
                let type_id = egraph.analysis.blocks[&funclet_id].input_types[*index];
                OutputSet::Single(Output {
                    type_id,
                    folded: None,
                })
            }
            NodeKind::Operation { kind } => {
                use OperationKind as Ok;
                match kind {
                    Ok::ExtractResult { index } => {
                        let src = node.deps[0];
                        let src_types = egraph[src]
                            .data
                            .multiple()
                            .expect("can't extract from single value");
                        OutputSet::Single(src_types[*index].clone())
                    }
                    Ok::Binop { kind: _ } => {
                        let src0 = node.deps[0];
                        let src0_type = egraph[src0]
                            .data
                            .single()
                            .expect("can't use binop on aggregate");
                        let src1 = node.deps[1];
                        let src1_type = egraph[src1]
                            .data
                            .single()
                            .expect("can't use binop on aggregate");

                        // if we were to do implicit casts, this is where it would happen
                        assert_eq!(src0_type, src1_type, "incompatible binop types");
                        // if we're here clearly it couldn't be constant folded :(
                        OutputSet::Single(Output {
                            type_id: src0_type.type_id,
                            folded: None,
                        })
                    }
                    Ok::Unop { kind: _ } => {
                        // if we were to do implicit casts, this is where it would happen
                        // (we would have to match on kind)
                        let src = node.deps[0];
                        let src_type = egraph[src]
                            .data
                            .single()
                            .expect("can't use unop on aggregate");
                        // if we're here clearly it couldn't be constant folded :(
                        OutputSet::Single(Output {
                            type_id: src_type.type_id,
                            folded: None,
                        })
                    }
                    // all constant nodes were handled above...
                    // TODO: this is really ugly
                    Ok::ConstantBool { .. }
                    | Ok::ConstantInteger { .. }
                    | Ok::ConstantUnsignedInteger { .. } => unreachable!(),
                    // TODO: blocked on ffi interface
                    Ok::CallExternalCpu { .. } => todo!(),
                    // TODO: blocked on ffi interface
                    Ok::CallExternalGpuCompute { .. } => todo!(),
                    // TODO: This needs mega analysis and really doesn't work in the value
                    // tail control flow branch right now.
                    Ok::CallValueFunction { .. } => todo!(),
                }
            }
        }
    }
    fn merge(&mut self, other: Self) -> egg::DidMerge {
        match (self, other) {
            (Self::Single(a), Self::Single(b)) => a.merge(b),
            (Self::Multiple(a), Self::Multiple(b)) => {
                assert_eq!(a.len(), b.len(), "multi-outputs have different num types");
                a.iter_mut()
                    .zip(b.into_vec().drain(..))
                    .fold(egg::DidMerge(false, false), |acc, (a, b)| acc | a.merge(b))
            }
            _ => panic!("can't merge a single-output and multiple-output eclass"),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
struct BlockArgs {
    args: Vec<Option<GraphId>>,
}
impl BlockArgs {
    fn from_ir(cvt: &from_ir::FuncletConverter, ids: &[ir::NodeId]) -> Result<Self, FromIrError> {
        let mut args = Vec::new();
        for &ir_id in ids {
            let graph_id = cvt.convert_node_id(ir_id, ir::Dependent::Tail)?;
            args.push(Some(graph_id));
        }
        Ok(Self { args })
    }
    fn get(&self, index: usize) -> Option<&GraphId> {
        self.args.get(index)?.as_ref()
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut GraphId> {
        self.args.get_mut(index)?.as_mut()
    }
    fn delete(&mut self, index: usize) {
        self.args[index] = None;
    }
    // An iterator over the current arguments (that is, all which haven't been deleted)
    fn args(&self) -> impl Iterator<Item = &'_ GraphId> {
        self.args.iter().filter_map(Option::as_ref)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Jump {
    dest: ir::FuncletId,
    args: BlockArgs,
}
impl Jump {
    fn from_ir(cvt: &from_ir::FuncletConverter, jump: &ir::Jump) -> Result<Self, FromIrError> {
        Ok(Self {
            dest: jump.target,
            args: BlockArgs::from_ir(cvt, &jump.args)?,
        })
    }
}
#[derive(Clone)]
enum BlockTail {
    Return(BlockArgs),
    Jump(Jump),
    Branch { cond: GraphId, j0: Jump, j1: Jump },
}
impl BlockTail {
    fn jumps(&self) -> impl Iterator<Item = &'_ Jump> {
        match self {
            BlockTail::Return(_) => BlockJumps::Return,
            BlockTail::Jump(jump) => BlockJumps::Jump(std::iter::once(jump)),
            BlockTail::Branch { j0, j1, .. } => {
                BlockJumps::Branch(std::iter::once(j0).chain(std::iter::once(j1)))
            }
        }
    }
    fn jumps_mut(&mut self) -> impl Iterator<Item = &'_ mut Jump> {
        match self {
            BlockTail::Return(_) => BlockJumpsMut::Return,
            BlockTail::Jump(jump) => BlockJumpsMut::Jump(std::iter::once(jump)),
            BlockTail::Branch { j0, j1, .. } => {
                BlockJumpsMut::Branch(std::iter::once(j0).chain(std::iter::once(j1)))
            }
        }
    }
}
enum BlockJumps<'a> {
    Return,
    Jump(std::iter::Once<&'a Jump>),
    Branch(std::iter::Chain<std::iter::Once<&'a Jump>, std::iter::Once<&'a Jump>>),
}
impl<'a> Iterator for BlockJumps<'a> {
    type Item = &'a Jump;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Return => None,
            Self::Jump(ref mut iter) => iter.next(),
            Self::Branch(ref mut iter) => iter.next(),
        }
    }
}
enum BlockJumpsMut<'a> {
    Return,
    Jump(std::iter::Once<&'a mut Jump>),
    Branch(std::iter::Chain<std::iter::Once<&'a mut Jump>, std::iter::Once<&'a mut Jump>>),
}
impl<'a> Iterator for BlockJumpsMut<'a> {
    type Item = &'a mut Jump;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Return => None,
            Self::Jump(ref mut iter) => iter.next(),
            Self::Branch(ref mut iter) => iter.next(),
        }
    }
}
struct BlockSkeleton {
    // This should *really* be a set - but since most nodes only have a few predecessors,
    // I suspect a vector (with linear search) will be faster.
    // Invariant: No duplicates.
    preds: Vec<ir::FuncletId>,
    input_types: Box<[ir::TypeId]>,
    tail: BlockTail,
}
impl BlockSkeleton {
    fn remove_pred(&mut self, pred: ir::FuncletId) {
        // we use a linear search since blocks probably don't have too many preds
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                self.preds.swap_remove(i);
                break;
            }
        }
    }
    fn add_pred(&mut self, pred: ir::FuncletId) {
        // make sure it doesn't already exist
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                return;
            }
        }
        self.preds.push(pred);
    }
    fn preds(&self) -> impl ExactSizeIterator + Iterator<Item = &'_ ir::FuncletId> {
        self.preds.iter()
    }
}

pub fn create(program: &ir::Program, head: ir::FuncletId) -> Result<GraphRunner, FromIrError> {
    const OPAQUE: ir::FuncletId = usize::MAX;
    let mut runner = GraphRunner::new(Analysis {
        head,
        blocks: HashMap::new(),
        block_ids: BTreeSet::new(),
        thread_candidates: Vec::new(),
        native_interface: program.types.clone(),
    });
    let mut stack = vec![(head, OPAQUE)];
    while let Some((id, pred)) = stack.pop() {
        if let Some(block) = runner.egraph.analysis.blocks.get_mut(&id) {
            block.add_pred(pred);
            continue;
        }
        let preds = vec![pred];
        let mut cvt = from_ir::FuncletConverter::new(&mut runner.egraph, id);
        for (node_id, node) in program.funclets[&id].nodes.iter().enumerate() {
            cvt.add_node(node, node_id)?;
        }
        let tail = match &program.funclets[&id].tail_edge {
            ir::TailEdge::Return { return_values } => {
                BlockTail::Return(BlockArgs::from_ir(&cvt, &return_values)?)
            }
            ir::TailEdge::Jump(jump) => {
                stack.push((jump.target, id));
                let cvt_jump = Jump::from_ir(&cvt, jump)?;
                runner.egraph.analysis.thread_candidates.push(id);
                BlockTail::Jump(cvt_jump)
            }
            ir::TailEdge::Branch { cond, j0, j1 } => {
                stack.push((j0.target, id));
                stack.push((j1.target, id));
                BlockTail::Branch {
                    cond: cvt.convert_node_id(*cond, ir::Dependent::Tail)?,
                    j0: Jump::from_ir(&cvt, &j0)?,
                    j1: Jump::from_ir(&cvt, &j1)?,
                }
            }
        };
        let input_types = program.funclets[&id].input_types.clone();
        runner.egraph.analysis.block_ids.insert(id);
        let prev = runner.egraph.analysis.blocks.insert(
            id,
            BlockSkeleton {
                preds,
                input_types,
                tail,
            },
        );
        assert!(prev.is_none());
    }
    runner
        .egraph
        .analysis
        .blocks
        .get_mut(&head)
        .unwrap()
        .remove_pred(OPAQUE);
    // We don't bother updating the runner roots, since they're not used anywhere.
    Ok(runner)
}
pub struct Analysis {
    head: ir::FuncletId,
    blocks: HashMap<ir::FuncletId, BlockSkeleton>,
    // this is used to speed up collapse_branches... it really shouldn't be necessary,
    // except iter_mut on a hashmap holds a mutable reference on *values* as well as keys,
    // which means we can't mutate other entries while using iter_mut...
    block_ids: BTreeSet<ir::FuncletId>,
    /// A list of candidates for jump threading.
    /// Each entry in this list is guaranteed to have a jump tail edge.
    thread_candidates: Vec<ir::FuncletId>,

    /// TODO: Post-merge, make this of type ffi::NativeInterface and adjust accordingly
    /// TODO: This should be a reference, but that's a nightmare...
    native_interface: Arena<ir::Type>,
}
impl Analysis {
    /// Attempts to collapse branches into jumps using constant folding.
    fn collapse_branches(&mut self, graph: &Graph) {
        for id in self.block_ids.iter() {
            let block = self.blocks.get_mut(id).unwrap();
            if let BlockTail::Branch { cond, j0, j1 } = &block.tail {
                if let Some(Constant::Bool(cond)) = graph[*cond].data.constant() {
                    let (tail, other) = if *cond { (j1, j0.dest) } else { (j0, j1.dest) };
                    // collapse the tail, and update the other destination's branches
                    block.tail = BlockTail::Jump(tail.clone());
                    self.blocks.get_mut(&other).unwrap().remove_pred(*id);
                    self.thread_candidates.push(*id);
                } else if j0 == j1 {
                    // Both branches are the same. Note that it's not sufficient for the
                    // targets to be the same, the arguments and their orderings must also match.
                    block.tail = BlockTail::Jump(j0.clone());
                    self.thread_candidates.push(*id);
                }
            }
        }
    }
    /// Attempts to apply "generalized jump threading"
    fn thread_jumps(&mut self) {
        while let Some(candidate) = self.thread_candidates.pop() {
            // according to dirty list invariant, this should never fail
            let target = match &self.blocks[&candidate].tail {
                BlockTail::Jump(jump) => jump.dest,
                _ => unreachable!("thread_candidates invariant violation"),
            };
            // if the target has a single predecessor, it *must* be the candidate
            if let &[pred] = self.blocks[&target].preds.as_slice() {
                assert!(candidate == pred);
                // fold target into candidate by replacing candidate's tail
                let cloned = self.blocks[&target].tail.clone();

                let block = self.blocks.get_mut(&candidate).unwrap();
                block.tail = cloned;

                // keep preds correct -- this depends on the fact that the target
                // is now truly "dead" (i.e. unreferenced)
                match &block.tail {
                    &BlockTail::Return(_) => (),
                    &BlockTail::Jump(Jump { dest, .. }) => {
                        let preds = self.blocks.get_mut(&dest).unwrap();
                        preds.remove_pred(target);
                        preds.add_pred(candidate);
                        // we still end in a jump, so we'll want to check this block again
                        self.thread_candidates.push(candidate);
                    }
                    &BlockTail::Branch {
                        j0: Jump { dest: d0, .. },
                        j1: Jump { dest: d1, .. },
                        ..
                    } => {
                        let preds = self.blocks.get_mut(&d0).unwrap();
                        preds.remove_pred(target);
                        preds.add_pred(candidate);
                        let preds = self.blocks.get_mut(&d1).unwrap();
                        preds.remove_pred(target);
                        preds.add_pred(candidate);
                    }
                }
            }
        }
    }
    pub fn bake_dominators(&self) -> ir::utils::BakedDoms {
        ir::utils::bake_dominators(self.head, |id| {
            self.blocks[&id].tail.jumps().map(|jump| jump.dest)
        })
    }
    pub fn head(&self) -> ir::FuncletId {
        self.head
    }
    pub fn lookup_type(&self, type_id: ir::TypeId) -> Option<&'_ ir::Type> {
        self.native_interface.get(&type_id)
    }
    /// Returns an iterator over the nodes which the given funclet references. This
    /// includes all nodes *currently* referenced by a tail edge and their recursive dependencies.
    /// Funclet outputs which were inlined/deleted are *not* considered required.
    pub fn referenced(&self, id: ir::FuncletId) -> impl Iterator<Item = &'_ GraphId> {
        let jumps = self.blocks[&id].tail.jumps();
        jumps.map(|j| j.args.args()).flatten()
    }
}

pub fn create_node(egraph: &mut Graph, node: Node, type_id: ir::TypeId) -> GraphId {
    let outputs = OutputSet::compute(&node, egraph, type_id);
    let id = egraph.add(node);
    egraph[id].data = ClassAnalysis {
        outputs: Some(outputs),
    };
    <Analysis as egg::Analysis<Node>>::modify(egraph, id);
    id
}

#[derive(Debug)]
pub struct ClassAnalysis {
    outputs: Option<OutputSet>,
}
impl ClassAnalysis {
    /// # Panics
    /// Panics if `[Self::finished]` returns false.
    pub fn single(&self) -> Option<&'_ Output> {
        // By the global invariant, we assume that the eclass has already been assigned
        // output information.
        // If it hasn't been assigned output information then either
        //  1. We're in `create_node`, right in between adding the node & manually setting
        //     it's analysis data. But `create_node` doesn't call this method, so this isn't it.
        //  2. We're not in `create_node`. All nodes created by `create_node` have output info,
        //     and if *any* node in an eclass has output info that output info will propagate to
        //     the eclass. So *none* of the nodes in the eclass were created using `create_node`.
        //     Since rewrites always add to existing eclasses, there must be at least one
        //     "root" node in the eclass that all other nodes were derived from. That is,
        //     at least one node in the eclass was manually added to the egraph.
        //     That's a BUG since manually adding nodes must always be done through `create_node`.
        assert!(self.finished(), "unfinished analysis");
        match self.outputs.as_ref().unwrap() {
            OutputSet::Single(s) => Some(s),
            OutputSet::Multiple(_) => None,
        }
    }
    /// # Panics
    /// Panics if `[Self::finished]` returns false.
    pub fn multiple(&self) -> Option<&'_ [Output]> {
        // See the comment in `single`
        assert!(self.finished(), "unfinished analysis");
        match self.outputs.as_ref().unwrap() {
            OutputSet::Single(_) => None,
            OutputSet::Multiple(m) => Some(m),
        }
    }
    /// # Panics
    /// Panics if `[Self::finished]` returns false.
    pub fn constant(&self) -> Option<&'_ Constant> {
        self.single().map(|o| o.folded.as_ref()).flatten()
    }
    pub fn finished(&self) -> bool {
        self.outputs.is_some()
    }
}
impl egg::Analysis<Node> for Analysis {
    type Data = ClassAnalysis;
    fn make(egraph: &Graph, enode: &Node) -> Self::Data {
        Self::Data { outputs: None }
    }
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> egg::DidMerge {
        egg::merge_option(&mut a.outputs, b.outputs, OutputSet::merge)
    }
    fn modify(egraph: &mut egg::EGraph<Node, Self>, id: egg::Id) {
        let data = &egraph[id].data;
        if let Some(OutputSet::Single(Output {
            folded: Some(c),
            type_id,
        })) = data.outputs
        {
            // TODO: assert the types are compatible?
            let folded_id = egraph.add(c.to_node(type_id));
            egraph.union(id, folded_id);
        }
    }
}

pub mod inline_args {
    use super::*;
    use std::str::FromStr;

    /// If it's possible to inline paramater #`index` of funclet `id`, returns the ID of
    /// the equivalent eclass.
    fn inline_param(id: ir::FuncletId, index: usize, egraph: &Graph) -> Option<egg::Id> {
        let mut possible_ids: Vec<egg::Id> = egraph.analysis.blocks[&id]
            .preds()
            // find all incoming jumps - we may have multiple from the same predecessor
            // (i.e. both branches of a branch tail, but potentially with different arguments)
            .map(|id| egraph.analysis.blocks[id].tail.jumps())
            .flatten()
            .filter(|jump| jump.dest == id)
            // get the eclass corresponding to this param for each jump
            .map(|jump| *jump.args.get(index).expect("parameter with invalid index"))
            // canonicalize the eclass
            .map(|eclass| egraph.find(eclass))
            // collect into vector & dedup - there's only one unique predecessor eclass,
            // if and only if the resulting vector has one element
            .collect();
        possible_ids.dedup();

        // 0 eclasses: no predecessors, so we can't inline the param
        // 1 eclass: we can inline
        // 2+ eclasses: multiple predecessors with non-equivalent args, can't inline
        match possible_ids.as_slice() {
            &[single_id] => Some(single_id),
            _ => None,
        }
    }
    struct Searcher {}
    impl egg::Searcher<Node, Analysis> for Searcher {
        fn search_eclass(
            &self,
            egraph: &Graph,
            eclass: egg::Id,
        ) -> Option<egg::SearchMatches<Node>> {
            // An eclass must have at least one node. If it has more than one, then either:
            //  1. it has no param nodes. Obviously not a candidate for param inlining.
            //  2. it contains a param node and at least one other node. Then the param must
            //     have already been inlined, since param inlining is the only way params
            //     can have other nodes in their equivalence class.
            if let &[Node {
                kind: NodeKind::Param { funclet_id, index },
                ..
            }] = egraph[eclass].nodes.as_slice()
            {
                let inlined_eclass = inline_param(funclet_id, index, egraph)?;
                // HACK: smuggle the node ID into the applier via (ab)using a variable
                // substitution
                let mut subst = egg::Subst::with_capacity(1);
                subst.insert(egg::Var::from_str("?inlined").unwrap(), inlined_eclass);
                return Some(egg::SearchMatches {
                    eclass,
                    substs: vec![subst],
                    ast: None,
                });
            }
            return None;
        }
        fn vars(&self) -> Vec<egg::Var> {
            Vec::new()
        }
    }
    struct Applier {}
    impl egg::Applier<Node, Analysis> for Applier {
        fn apply_one(
            &self,
            egraph: &mut egg::EGraph<Node, Analysis>,
            eclass: egg::Id,
            subst: &egg::Subst,
            _searcher_ast: Option<&egg::PatternAst<Node>>,
            _rule_name: egg::Symbol,
        ) -> Vec<egg::Id> {
            let (id, index) = match egraph[eclass].nodes.as_slice() {
                &[Node {
                    kind: NodeKind::Param { funclet_id, index },
                    ..
                }] => (funclet_id, index),
                _ => unreachable!("we just checked this in the searcher"),
            };

            // ugly hack b/c lifetime issues
            let preds: Vec<_> = egraph.analysis.blocks[&id].preds().copied().collect();
            for pred_id in preds.iter() {
                // if this fails, we have a corrupted analysis
                let pred = egraph.analysis.blocks.get_mut(pred_id).unwrap();
                for jump in pred.tail.jumps_mut() {
                    jump.args.delete(index);
                }
            }

            let inlined_eclass = subst
                .get(egg::Var::from_str("?inlined").unwrap())
                .copied()
                .expect("we added this variable in the searcher");
            vec![eclass, inlined_eclass]
        }
    }
    pub fn rewrite() -> egg::Rewrite<Node, Analysis> {
        let name = egg::Symbol::new("Parameter Rewrite (argument inlining)");
        let searcher = Searcher {};
        let applier = Applier {};
        egg::Rewrite::new(name, searcher, applier).unwrap()
    }
}
