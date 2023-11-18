use std::collections::{HashMap, HashSet};

use crate::{
    enum_cast,
    parse::ast::{SchedExpr, SchedStmt, SchedTerm},
};

use super::{stmts_to_hir, Hir, Terminator};

/// The id of the final block of the canonicalized CFG.
/// A canonical CFG has one entry and exit node.
pub const FINAL_BLOCK_ID: usize = 0;
/// The id of the entry block of the canonicalized CFG.
/// A canonical CFG has one entry and exit node.
pub const START_BLOCK_ID: usize = FINAL_BLOCK_ID + 1;

/// A basic block in a scheduling function
pub struct BasicBlock {
    pub id: usize,
    /// Invariant: no tail edges in the middle of the block
    pub stmts: Vec<Hir>,
    pub terminator: Terminator,
    /// The next block on the same depth as this one, which
    /// is the join point for any continuations in this block
    pub join_block: Option<usize>,
}

/// An edge in the CFG
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Next(usize),
    Select {
        true_branch: usize,
        false_branch: usize,
    },
    None,
}

/// A control flow graph for a scheduling function.
/// Each basic block in the CFG contains HIR body statements and a terminator.
pub struct Cfg {
    pub blocks: HashMap<usize, BasicBlock>,
    pub(super) graph: HashMap<usize, Edge>,
}

/// Make a basic block from the current statements, giving it the next id
/// and incrementing the id counter. The current statements are cleared.
fn make_block(
    cur_id: &mut usize,
    cur_stmt: &mut Vec<SchedStmt>,
    term: Terminator,
    join_edge: &Edge,
) -> BasicBlock {
    let join_block = match join_edge {
        Edge::Next(id) => Some(*id),
        Edge::None => None,
        Edge::Select { .. } => panic!("Join edge should be unconditional"),
    };
    let res = BasicBlock {
        id: *cur_id,
        stmts: stmts_to_hir(std::mem::take(cur_stmt)),
        terminator: term,
        join_block,
    };
    *cur_id += 1;
    res
}

/// A pending child block to be processed and converted into a sequence of basic
/// blocks.
struct PendingChild {
    parent_id: usize,
    join_id: usize,
    true_block: Vec<SchedStmt>,
    false_block: Option<Vec<SchedStmt>>,
}

/// Removes all Blocks from a list of scheduling statements and flattens them into
/// a list of scheduling statements without nested blocks
fn flatten_stmts(stmts: Vec<SchedStmt>) -> Vec<SchedStmt> {
    let mut res = vec![];
    for stmt in stmts {
        match stmt {
            SchedStmt::Block(_, stmts) => res.extend(flatten_stmts(stmts)),
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                info,
            } => res.push(SchedStmt::If {
                guard,
                true_block: flatten_stmts(true_block),
                false_block: flatten_stmts(false_block),
                info,
            }),
            other => res.push(other),
        }
    }
    res
}

/// Makes one or more basic blocks from a list of scheduling statements. Adds the
/// blocks to the list of blocks and adds edges to the edge map. Also updates the
/// id counter so that the counter stores the value for the next available id.
///
/// # Arguments
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///     and thus always is the id of the next block.
/// * `blocks` - The list of blocks to add to. New blocks are appended to the end.
/// * `edges` - The list of edges to add to. Edges for new blocks are added to this map.
/// * `stmts` - The list of scheduling statements to convert to blocks.
/// * `join_edge` - The edge to use for a basic block to join back to the parent.
fn make_blocks(
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
    stmts: Vec<SchedStmt>,
    join_edge: Edge,
) -> usize {
    let mut cur_stmts = vec![];
    let root_id = *cur_id;
    // we do a BFS on the graph, this is the queue of children
    let mut children = vec![];
    let ends_with_tail = stmts.last().map_or(false, |x| {
        matches!(x, SchedStmt::Return(..) | SchedStmt::Call(..))
    });
    for stmt in stmts {
        match stmt {
            SchedStmt::Return(_, sched_expr) => {
                let old_id = *cur_id;
                blocks.insert(
                    *cur_id,
                    make_block(
                        cur_id,
                        &mut cur_stmts,
                        Terminator::Return(Some(expr_to_node_id(sched_expr))),
                        &join_edge,
                    ),
                );
                edges.insert(old_id, Edge::Next(FINAL_BLOCK_ID));
            }
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                ..
            } => {
                let parent_id = *cur_id;
                blocks.insert(
                    *cur_id,
                    make_block(
                        cur_id,
                        &mut cur_stmts,
                        Terminator::Select(expr_to_node_id(guard)),
                        &join_edge,
                    ),
                );
                children.push(PendingChild {
                    parent_id,
                    join_id: *cur_id,
                    true_block,
                    false_block: if false_block.is_empty() {
                        None
                    } else {
                        Some(false_block)
                    },
                });
            }
            // TODO (function and procedure calls)
            other => cur_stmts.push(other),
        }
    }
    if !cur_stmts.is_empty() || !edges.contains_key(cur_id) && !ends_with_tail {
        // complete CFG for void-returning functions / the end of a section of a basic block
        // this will create empty blocks when a statement is the last statement in a scope.

        // we do need empty blocks in some cases. Consider
        /*
           if x {
               // empty
           } else {
               // something
           }
        */
        // the continuation can't also be one of the target funclets in the select
        let old_id = *cur_id;
        blocks.insert(
            *cur_id,
            make_block(cur_id, &mut cur_stmts, Terminator::None, &join_edge),
        );
        edges.insert(old_id, join_edge);
    }
    make_child_blocks(children, cur_id, blocks, edges);
    root_id
}

/// Make child blocks from a list of pending children. Adds the blocks to the list of blocks
/// and adds edges to the edge map. Also updates the id counter so that the counter stores the
/// value for the next available id.
///
/// This is a recursive helper for `make_blocks` to process the next level of nodes
/// in the BFS
fn make_child_blocks(
    children: Vec<PendingChild>,
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
) {
    for PendingChild {
        parent_id,
        join_id,
        true_block,
        false_block,
    } in children
    {
        let join_edge = Edge::Next(join_id);
        let true_branch = make_blocks(cur_id, blocks, edges, true_block, join_edge);
        let false_branch = false_block.map(|f| make_blocks(cur_id, blocks, edges, f, join_edge));
        if let Some(false_branch) = false_branch {
            edges.insert(
                parent_id,
                Edge::Select {
                    true_branch,
                    false_branch,
                },
            );
        } else {
            edges.insert(parent_id, Edge::Next(true_branch));
        }
    }
}

impl Cfg {
    /// Create a new CFG from a list of scheduling statements
    pub fn new(stmts: Vec<SchedStmt>) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            FINAL_BLOCK_ID,
            BasicBlock {
                id: FINAL_BLOCK_ID,
                stmts: vec![],
                terminator: Terminator::FinalReturn,
                join_block: None,
            },
        );
        let mut edges = HashMap::new();
        let mut cur_id = START_BLOCK_ID;
        edges.insert(FINAL_BLOCK_ID, Edge::None);
        make_blocks(
            &mut cur_id,
            &mut blocks,
            &mut edges,
            flatten_stmts(stmts),
            Edge::Next(FINAL_BLOCK_ID),
        );
        Self {
            blocks,
            graph: edges,
        }
        .remove_unreachable()
    }

    /// Gets the successors of a given block
    pub fn successors(&self, block_id: usize) -> Vec<usize> {
        match self.graph[&block_id] {
            Edge::Next(id) => vec![id],
            Edge::Select {
                true_branch,
                false_branch,
            } => vec![true_branch, false_branch],
            Edge::None => vec![],
        }
    }

    /// Removes unreachable blocks from the CFG
    fn remove_unreachable(mut self) -> Self {
        // we do a BFS on a graph, that means, when we generate a sibling node of
        // a parent, we don't know whether niece or nephew nodes might join back
        // or not. Hence, we may create unreachable nodes. We remove them here.
        // Consider:
        /*
            if x {
                ret z
            } else {
                ret y
            }

        */
        // After the first if, we create a sibling block so the children can join
        // in before transitioning to the end block. However, in this case
        // the children have direct returns, so the sibling block is unreachable.
        let mut unreachable: HashSet<_> = self.graph.keys().copied().collect();
        unreachable.remove(&START_BLOCK_ID);
        for edge in self.graph.values() {
            match edge {
                Edge::Next(id) => {
                    unreachable.remove(id);
                }
                Edge::Select {
                    true_branch,
                    false_branch,
                } => {
                    unreachable.remove(true_branch);
                    unreachable.remove(false_branch);
                }
                Edge::None => (),
            }
        }
        // this doesn't take unreachable cycles into account, which I think
        // can't occur right now
        self.graph.retain(|k, _| !unreachable.contains(k));
        self.blocks.retain(|_, b| !unreachable.contains(&b.id));
        self
    }
}

/// Converts an expression to a node Id, assuming the expression is just a variable
/// # Panics
/// Panics if the expression is not a variable
fn expr_to_node_id(e: SchedExpr) -> String {
    let t = enum_cast!(SchedExpr::Term, e);
    enum_cast!(SchedTerm::Var { name, .. }, name, t)
}
