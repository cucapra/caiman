use super::*;
use crate::ir;
use std::collections::hash_map::{Entry, HashMap};

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);
impl BlockId {
    /// Represents an external/unknown reference to a block/funclet
    const OPAQUE: Self = BlockId(usize::MAX);
}

#[derive(Clone)]
struct BlockArgs {
    args: Vec<Option<GraphId>>,
}
impl BlockArgs {
    fn new(converter: &from_ir::FuncletConverter, ids: &[ir::NodeId]) -> Result<Self, FromIrError> {
        let mut args = Vec::new();
        for &ir_id in ids {
            let graph_id = converter.convert_node_id(ir_id, ir::Dependent::Tail)?;
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
    dest: BlockId,
    args: BlockArgs,
}

#[derive(Clone)]
enum BlockTail {
    Return(BlockArgs),
    Jump(Jump),
    Branch { cond: GraphId, j0: Jump, j1: Jump },
}
struct BlockSkeleton {
    ir_id: ir::FuncletId,
    // This should *really* be a set - but since most nodes only have a few predecessors,
    // I suspect a vector (with linear search) will be faster.
    // Invariant: No duplicates.
    preds: Vec<BlockId>,
    tail: BlockTail,
}
impl BlockSkeleton {
    fn remove_pred(&mut self, pred: BlockId) {
        // we use a linear search since blocks probably don't have too many preds
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                self.preds.swap_remove(i);
                break;
            }
        }
    }
    fn add_pred(&mut self, pred: BlockId) {
        // make sure it doesn't already exist
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                return;
            }
        }
        self.preds.push(pred);
    }
    /// TODO: Refactor this so that we don't have this weird interface
    fn delete_arg(&mut self, destination: BlockId, arg_index: usize) {
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
    blocks: Vec<BlockSkeleton>,
    ir_map: HashMap<ir::FuncletId, BlockId>,
    /// A list of candidates for jump threading.
    /// Each entry in this list is guaranteed to have a jump tail edge.
    thread_candidates: Vec<BlockId>,
}
impl Analysis {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            ir_map: HashMap::new(),
            thread_candidates: Vec::new(),
        }
    }
    pub(super) fn build_with_graph(
        &mut self,
        graph: &mut GraphInner,
        program: &ir::Program,
        start: ir::FuncletId,
    ) -> Result<(), FromIrError> {
        let mut stack = vec![(start, BlockId::OPAQUE)];
        while let Some((ir_id, pred)) = stack.pop() {
            match self.ir_map.entry(ir_id) {
                Entry::Occupied(block_id) => {
                    self.blocks[block_id.get().0].add_pred(pred);
                }
                Entry::Vacant(spot) => {
                    let block_id = BlockId(self.blocks.len());
                    let preds = vec![pred];
                    let mut converter = from_ir::FuncletConverter::new(graph, ir_id);
                    for (node_id, node) in program.funclets[&ir_id].nodes.iter().enumerate() {
                        converter.add_node(node, node_id)?;
                    }
                    let tail = match &program.funclets[&ir_id].tail_edge {
                        ir::TailEdge::Return { return_values } => {
                            BlockTail::Return(BlockArgs::new(&converter, &return_values)?)
                        }
                        ir::TailEdge::Jump(jump) => {
                            self.thread_candidates.push(block_id);
                            stack.push((jump.target, block_id));
                            todo!();
                        }
                        ir::TailEdge::Branch { cond, j0, j1 } => {
                            stack.push((j0.target, block_id));
                            stack.push((j1.target, block_id));
                            todo!();
                        }
                    };
                    self.blocks.push(BlockSkeleton { ir_id, preds, tail });
                    spot.insert(block_id);
                }
            }
        }
        Ok(())
    }
    /// Attempts to collapse branches into jumps using constant folding.
    fn collapse_branches(&mut self, graph: &GraphInner) {
        for i in 0..self.blocks.len() {
            let block = &mut self.blocks[i];
            if let BlockTail::Branch { cond, j0, j1 } = &mut block.tail {
                if let Some(Constant::Bool(cond)) = graph[*cond].data.constant {
                    let (tail, other) = if cond { (j1, j0.dest) } else { (j0, j1.dest) };
                    // collapse the tail, and update the other destination's branches
                    block.tail = BlockTail::Jump(tail.clone());
                    self.blocks[other.0].remove_pred(BlockId(i));
                    // update dirty list
                    self.thread_candidates.push(BlockId(i));
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
            let target = unwrap_jump(&self.blocks[candidate.0].tail).dest;
            // if the target has a single predecessor, it *must* be the candidate
            if let &[pred] = self.blocks[target.0].preds.as_slice() {
                assert!(candidate == pred);
                // fold target into candidate by replacing candidate's tail
                self.blocks[candidate.0].tail = self.blocks[target.0].tail.clone();
                // keep preds correct -- this depends on the fact that the target
                // is now truly "dead" (i.e. unreferenced)
                match &self.blocks[candidate.0].tail {
                    &BlockTail::Return(_) => (),
                    &BlockTail::Jump(Jump { dest, .. }) => {
                        self.blocks[dest.0].remove_pred(target);
                        self.blocks[dest.0].add_pred(candidate);
                        // we still end in a jump, so we'll want to check this block again
                        self.thread_candidates.push(candidate);
                    }
                    &BlockTail::Branch {
                        j0: Jump { dest: d0, .. },
                        j1: Jump { dest: d1, .. },
                        ..
                    } => {
                        self.blocks[d0.0].remove_pred(target);
                        self.blocks[d1.0].remove_pred(target);
                        self.blocks[d0.0].add_pred(candidate);
                        self.blocks[d1.0].add_pred(candidate);
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
