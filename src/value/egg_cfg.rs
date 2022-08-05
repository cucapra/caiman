use crate::ir;
use crate::value::{Constant, GraphId, GraphInner};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FunId(usize);
impl FunId {
    const EXTERNAL: Self = Self(0);
}
#[derive(Clone)]
struct FunArgs {
    args: Vec<Option<GraphId>>,
}
impl FunArgs {
    fn new(ids: &[GraphId]) -> Self {
        Self {
            args: ids.iter().map(|&id| Some(id)).collect(),
        }
    }
    fn delete(&mut self, index: usize) {
        self.args[index] = None;
    }
}

#[derive(Clone)]
struct Jump {
    dest: FunId,
    args: FunArgs,
}

#[derive(Clone)]
enum FunTail {
    Return(FunArgs),
    Jump(Jump),
    Branch { cond: GraphId, j0: Jump, j1: Jump },
}
struct BlockSkeleton {
    ir_id: ir::FuncletId,
    // This should *really* be a set - but since most nodes only have a few predecessors,
    // I suspect a vector (with linear search) will be faster.
    // Invariant: No duplicates.
    preds: Vec<FunId>,
    tail: FunTail,
}
impl BlockSkeleton {
    fn remove_pred(&mut self, pred: FunId) {
        // we use a linear search since blocks probably don't have too many preds
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                self.preds.swap_remove(i);
                break;
            }
        }
    }
    fn add_pred(&mut self, pred: FunId) {
        // make sure it doesn't already exist
        for i in 0..self.preds.len() {
            if self.preds[i] == pred {
                return;
            }
        }
        self.preds.push(pred);
    }
    /// TODO: Refactor this so that we don't have this weird interface
    fn delete_arg(&mut self, destination: FunId, arg_index: usize) {
        match &mut self.tail {
            FunTail::Return(_) => (),
            FunTail::Jump(jump) => {
                if jump.dest == destination {
                    jump.args.delete(arg_index);
                }
            }
            FunTail::Branch { j0, j1, .. } => {
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
pub struct EggCfg {
    blocks: Vec<BlockSkeleton>,
}
impl EggCfg {
    /// Attempts to collapse branches into jumps using constant folding.
    /// Returns true if the CFG skeleton was changed.
    fn collapse_branches(&mut self, graph: &GraphInner) -> bool {
        let mut mutated = false;
        for i in 0..self.blocks.len() {
            let block = &mut self.blocks[i];
            if let FunTail::Branch { cond, j0, j1 } = &mut block.tail {
                if let Some(Constant::Bool(cond)) = graph[*cond].data.constant {
                    // collapse the tail, and update the other destination's branches
                    let unselected_id;
                    if cond {
                        unselected_id = j0.dest;
                        block.tail = FunTail::Jump(j1.clone());
                    } else {
                        unselected_id = j1.dest;
                        block.tail = FunTail::Jump(j0.clone());
                    }
                    self.blocks[unselected_id.0].remove_pred(FunId(i));
                    mutated |= true;
                }
            }
        }
        mutated
    }
    /// Attempts to apply "generalized jump threading"
    fn thread_jumps(&mut self) {
        let mut stack: Vec<_> = (0..self.blocks.len()).map(|i| FunId(i)).collect();
        while let Some(candidate) = stack.pop() {
            let block = &self.blocks[candidate.0];
            // if the candidate has a single predecessor...
            if let &[pred] = block.preds.as_slice() {
                // ...and that predecessor is a direct jump to the candidate...
                if let FunTail::Jump(jump) = &self.blocks[pred.0].tail {
                    assert!(jump.dest == candidate);
                    // ...then we can "merge" the predecessor and the candidate.
                    // The predecessor recieves the candidate's tail.
                    self.blocks[pred.0].tail = block.tail.clone();
                    // However, the candidate may have had successors. Each successor should
                    // be fixed up so it references the predecessor.
                    match &self.blocks[pred.0].tail {
                        &FunTail::Return(_) => (),
                        &FunTail::Jump(Jump { dest, .. }) => {
                            self.blocks[dest.0].remove_pred(candidate);
                            self.blocks[dest.0].add_pred(pred);
                            // now the predecessor has a jump tail, so it's destination
                            // is a potential candidate
                            stack.push(dest);
                        }
                        &FunTail::Branch {
                            j0: Jump { dest: d0, .. },
                            j1: Jump { dest: d1, .. },
                            ..
                        } => {
                            self.blocks[d0.0].remove_pred(candidate);
                            self.blocks[d0.0].add_pred(pred);
                            self.blocks[d1.0].remove_pred(candidate);
                            self.blocks[d1.0].add_pred(pred);
                        }
                    }
                }
            }
        }
    }
    /// Attempts to optimize the CFG skeleton via branch elimination and generalized jump threading
    pub(super) fn optimize(&mut self, graph: &GraphInner) {
        let changed = self.collapse_branches(graph);
        if changed {
            self.thread_jumps();
        }
    }
}
