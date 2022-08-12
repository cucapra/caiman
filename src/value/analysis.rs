use super::*;
use crate::ir;
use crate::operations::{BinopKind, UnopKind};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CFoldError {
    // TODO: something about upcasting? upcastable?
    // probably not, we don't need Rust-quality diagnostics here :P
    // It would be a good idea to improve these eventually though...
    // somehow trace the chain of constant evaluations
    // The math example for egg does this, but it's not friendly at all
    #[error("constant folding operands have incompatible types")]
    IncompatibleTypes,
    #[error("constant folding produced an out-of-range value")]
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constant {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}
macro_rules! impl_std_unop {
    ($trait:path, $name:ident, $out0:ident, $tok:tt for {$($variant:ident),*}) => {
        impl $trait for Constant {
            type Output = Result<Constant, CFoldError>;
            fn $name(self) -> Self::Output {
                match self {
                    $(Self::$variant($out0) => Ok(Self::$variant $tok),)*
                    _ => return Err(CFoldError::IncompatibleTypes)
                }
            }
        }
    };
}

impl_std_unop!(std::ops::Neg, neg, x, (-x) for {I8, I16, I32, I64});

fn cfold_unop(egraph: &GraphInner, unop: UnopKind, deps: &[egg::Id]) -> Option<Constant> {
    assert!(deps.len() == 1, "unop has one argument");
    let x = egraph[deps[0]].data.constant?;
    match unop {
        UnopKind::Neg => Some((-x).unwrap()),
    }
}

macro_rules! impl_std_binop {
    ($trait:path, $name:ident, $out0:ident, $out1:ident, $tok:tt for {$($variant:ident),*}) => {
        impl $trait for Constant {
            type Output = Result<Constant, CFoldError>;
            fn $name(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    $((Self::$variant($out0), Self::$variant($out1)) => Ok(Self::$variant $tok),)*
                    _ => return Err(CFoldError::IncompatibleTypes)
                }
            }
        }
    };
}

impl_std_binop!(std::ops::Add, add, a, b, (a.checked_add(b).ok_or(CFoldError::OutOfRange)?)
    for {U8, U16, U32, U64, I8, I16, I32, I64}
);
impl_std_binop!(std::ops::Sub, sub, a, b, (a.checked_sub(b).ok_or(CFoldError::OutOfRange)?)
    for {U8, U16, U32, U64, I8, I16, I32, I64}
);

fn cfold_binop(egraph: &GraphInner, binop: BinopKind, deps: &[egg::Id]) -> Option<Constant> {
    assert!(deps.len() == 2, "binop has two arguments");
    let a = egraph[deps[0]].data.constant?;
    let b = egraph[deps[1]].data.constant?;
    match binop {
        // I'm not sure how to handle errors here... we could silently ignore them by using
        // (a + b).ok() but that seems like a disservice to a user wondering why their
        // value function is buggy, and the errors will almost certainly pop up later
        // in codegen or at runtime. egg doesn't have a way to do fallible analyses AFAIK
        BinopKind::Add => Some((a + b).unwrap()),
        BinopKind::Sub => Some((a - b).unwrap()),
    }
}
impl Node {
    fn constant_fold(&self, egraph: &GraphInner) -> Option<Constant> {
        match self.operation()? {
            OperationKind::ConstantBool { value } => Some(Constant::Bool(*value)),
            OperationKind::ConstantInteger { value, .. } => Some(Constant::I64(*value)),
            OperationKind::ConstantUnsignedInteger { value, .. } => Some(Constant::U64(*value)),
            OperationKind::Unop { kind } => cfold_unop(egraph, *kind, &self.deps),
            OperationKind::Binop { kind } => cfold_binop(egraph, *kind, &self.deps),
            _ => None,
        }
    }
}
#[derive(Debug)]
pub struct ClassAnalysis {
    constant: Option<Constant>,
}
impl ClassAnalysis {
    fn new(egraph: &GraphInner, enode: &Node) -> Self {
        Self {
            constant: enode.constant_fold(egraph),
        }
    }
    fn merge(&mut self, other: Self) -> egg::DidMerge {
        egg::merge_option(&mut self.constant, other.constant, |a, b| {
            assert_eq!(a, &b, "rewrite type violation (merge non-equal constants)");
            egg::DidMerge(false, false)
        })
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
}
impl Analysis {
    pub fn new() -> Self {
        Self {
            head: 0, // HACK: this only works as long as funclet IDs are usize
            // then again, new() only exists as part of a hack for building the graph...
            blocks: HashMap::new(),
            block_ids: BTreeSet::new(),
            thread_candidates: Vec::new(),
        }
    }
    pub(super) fn build_with_graph(
        &mut self,
        graph: &mut GraphInner,
        program: &ir::Program,
        start: ir::FuncletId,
    ) -> Result<(), FromIrError> {
        const OPAQUE: ir::FuncletId = usize::MAX;
        let mut stack = vec![(start, OPAQUE)];
        while let Some((id, pred)) = stack.pop() {
            match self.blocks.entry(id) {
                Entry::Occupied(mut block) => {
                    block.get_mut().add_pred(pred);
                }
                Entry::Vacant(spot) => {
                    let preds = vec![pred];
                    let mut cvt = from_ir::FuncletConverter::new(graph, id);
                    for (node_id, node) in program.funclets[&id].nodes.iter().enumerate() {
                        cvt.add_node(node, node_id)?;
                    }
                    let tail = match &program.funclets[&id].tail_edge {
                        ir::TailEdge::Return { return_values } => {
                            BlockTail::Return(BlockArgs::from_ir(&cvt, &return_values)?)
                        }
                        ir::TailEdge::Jump(jump) => {
                            self.thread_candidates.push(id);
                            stack.push((jump.target, id));
                            BlockTail::Jump(Jump::from_ir(&cvt, jump)?)
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
                    spot.insert(BlockSkeleton { preds, tail });
                    self.block_ids.insert(id);
                }
            }
        }
        self.blocks.get_mut(&start).unwrap().remove_pred(OPAQUE);
        Ok(())
    }
    /// Attempts to collapse branches into jumps using constant folding.
    fn collapse_branches(&mut self, graph: &GraphInner) {
        for id in self.block_ids.iter() {
            let block = self.blocks.get_mut(id).unwrap();
            if let BlockTail::Branch { cond, j0, j1 } = &block.tail {
                if let Some(Constant::Bool(cond)) = graph[*cond].data.constant {
                    let (tail, other) = if cond { (j1, j0.dest) } else { (j0, j1.dest) };
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
        fn unwrap_jump(tail: &BlockTail) -> &Jump {
            if let BlockTail::Jump(jump) = &tail {
                return jump;
            }
            unreachable!("thread_candidates invariant violation");
        }
        while let Some(candidate) = self.thread_candidates.pop() {
            // according to dirty list invariant, this should never fail
            let target = unwrap_jump(&self.blocks[&candidate].tail).dest;
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
    /// Returns an iterator over the nodes which the given funclet references. This
    /// includes all nodes *currently* referenced by a tail edge and their recursive dependencies.
    /// Funclet outputs which were inlined/deleted are *not* considered required.
    pub fn referenced(&self, id: ir::FuncletId) -> impl Iterator<Item = &'_ GraphId> {
        let jumps = self.blocks[&id].tail.jumps();
        jumps.map(|j| j.args.args()).flatten()
    }
}

impl egg::Analysis<Node> for Analysis {
    type Data = ClassAnalysis;
    fn make(egraph: &GraphInner, enode: &Node) -> Self::Data {
        ClassAnalysis::new(egraph, enode)
    }
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> egg::DidMerge {
        a.merge(b)
    }
}

pub mod inline_args {
    use super::*;
    use std::str::FromStr;

    /// If it's possible to inline paramater #`index` of funclet `id`, returns the ID of
    /// the equivalent eclass.
    fn inline_param(id: ir::FuncletId, index: usize, egraph: &GraphInner) -> Option<egg::Id> {
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
            egraph: &GraphInner,
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
