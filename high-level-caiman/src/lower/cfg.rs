use std::collections::HashMap;

use crate::parse::ast::{SchedExpr, SchedStmt};

/// A terminator of a basic block
#[derive(Clone, Debug)]
pub enum Terminator {
    Call(String, Vec<SchedExpr>),
    Select(SchedExpr),
    Return(Option<SchedExpr>),
}

/// A basic block in a scheduling function
pub struct BasicBlock {
    pub id: usize,
    /// Invariant: no tail edges in the middle of the block
    pub stmts: Vec<SchedStmt>,
    pub terminator: Terminator,
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

/// A control flow graph for a scheduling function
pub struct Cfg {
    pub blocks: Vec<BasicBlock>,
    graph: HashMap<usize, Edge>,
}

/// Make a basic block from the current statements, giving it the next id
/// and incrementing the id counter. The current statements are cleared.
fn make_block(cur_id: &mut usize, cur_stmt: &mut Vec<SchedStmt>, term: Terminator) -> BasicBlock {
    let res = BasicBlock {
        id: *cur_id,
        stmts: std::mem::take(cur_stmt),
        terminator: term,
    };
    *cur_id += 1;
    res
}

/// A pending child block to be processed
struct PendingChild {
    parent_id: usize,
    join_id: usize,
    true_block: Vec<SchedStmt>,
    false_block: Option<Vec<SchedStmt>>,
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
    blocks: &mut Vec<BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
    stmts: Vec<SchedStmt>,
    join_edge: Edge,
) -> usize {
    let mut cur_stmts = vec![];
    let root_id = *cur_id;
    // we do a BFS on the graph, this is the queue of children
    let mut children = vec![];
    for stmt in stmts {
        match stmt {
            SchedStmt::Return(_, sched_expr) => {
                let old_id = *cur_id;
                blocks.push(make_block(
                    cur_id,
                    &mut cur_stmts,
                    Terminator::Return(Some(sched_expr)),
                ));
                edges.insert(old_id, Edge::None);
            }
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                ..
            } => {
                let parent_id = *cur_id;
                blocks.push(make_block(
                    cur_id,
                    &mut cur_stmts,
                    Terminator::Select(guard),
                ));
                children.push(PendingChild {
                    parent_id,
                    join_id: *cur_id,
                    true_block,
                    false_block: false_block.map(|x| vec![*x]),
                });
            }
            // TODO (calls)
            other => cur_stmts.push(other),
        }
    }
    make_child_blocks(children, join_edge, cur_id, blocks, edges);
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
    join_edge: Edge,
    cur_id: &mut usize,
    blocks: &mut Vec<BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
) {
    for PendingChild {
        parent_id,
        join_id,
        true_block,
        false_block,
    } in children
    {
        let join_edge = if edges.contains_key(&join_id) {
            Edge::Next(join_id)
        } else {
            join_edge
        };
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
        let mut blocks = vec![];
        let mut edges = HashMap::new();
        let mut cur_id = 0;
        make_blocks(&mut cur_id, &mut blocks, &mut edges, stmts, Edge::None);
        Self {
            blocks,
            graph: edges,
        }
    }
}
