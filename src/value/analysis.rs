use super::*;
use crate::ir;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::BTreeSet;

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

#[derive(Debug)]
pub struct ClassAnalysis {
    constant: Option<Constant>,
}
impl ClassAnalysis {
    fn new(_egraph: &GraphInner, enode: &Node) -> Self {
        let mut constant = None;
        if let NodeKind::Operation { kind } = &enode.kind {
            if let OperationKind::ConstantInteger { value, .. } = kind {
                constant = Some(Constant::I64(*value));
            } else if let OperationKind::ConstantUnsignedInteger { value, .. } = kind {
                constant = Some(Constant::U64(*value));
            }
        }
        Self { constant }
    }
    fn merge(&mut self, other: Self) -> egg::DidMerge {
        let constant_merge = match (self.constant, other.constant) {
            (None, None) => egg::DidMerge(false, false),
            (Some(_), None) => egg::DidMerge(false, true),
            (None, mut b @ Some(_)) => {
                self.constant = b.take();
                egg::DidMerge(true, false)
            }
            (Some(a), Some(b)) => {
                assert!(a == b, "graph rewrite violated the type system");
                egg::DidMerge(false, false)
            }
        };
        constant_merge
    }
}

#[derive(Clone)]
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
    fn delete(&mut self, index: usize) {
        self.args[index] = None;
    }
}

#[derive(Clone)]
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
    /// TODO: Refactor this so that we don't have this weird interface
    fn delete_arg(&mut self, destination: ir::FuncletId, arg_index: usize) {
        match &mut self.tail {
            BlockTail::Return(_) => (),
            BlockTail::Jump(jump) => {
                if jump.dest == destination {
                    jump.args.delete(arg_index);
                }
            }
            BlockTail::Branch { j0, j1, .. } => {
                if j0.dest == destination {
                    j0.args.delete(arg_index);
                }
                if j1.dest == destination {
                    j1.args.delete(arg_index);
                }
            }
        }
    }
}
pub struct Analysis {
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
                    // TODO:
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
                    // update dirty list
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
