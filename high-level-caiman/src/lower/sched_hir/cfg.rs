use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    enum_cast,
    error::Info,
    parse::ast::{FullType, SchedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm},
};

use super::{analysis::compute_coninuations, stmts_to_hir, HirBody, Terminator};

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
    pub stmts: Vec<HirBody>,
    pub terminator: Terminator,
    /// The next block at the same stack level as this block. This is the block's
    /// continuation. This is `None` if the block is the last block at the
    /// current level of depth.
    pub ret_block: Option<usize>,
    /// Starting line and index for the block
    pub src_loc: Info,
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

impl Edge {
    /// Gets a vector of node ids that this edge connects to
    pub fn targets(&self) -> Vec<usize> {
        match self {
            Self::Next(id) => vec![*id],
            Self::Select {
                true_branch,
                false_branch,
            } => vec![*true_branch, *false_branch],
            Self::None => vec![],
        }
    }

    /// Gets the number of nodes that this edge connects to
    #[allow(dead_code)]
    pub const fn num_targets(&self) -> usize {
        match self {
            Self::Next(_) => 1,
            Self::Select { .. } => 2,
            Self::None => 0,
        }
    }
}

/// A control flow graph for a scheduling function.
/// Each basic block in the CFG contains HIR body statements and a terminator.
pub struct Cfg {
    pub blocks: HashMap<usize, BasicBlock>,
    pub(super) graph: HashMap<usize, Edge>,
    pub(super) transpose_graph: HashMap<usize, BTreeSet<usize>>,
}

/// Make a basic block from the current statements, giving it the next id
/// and incrementing the id counter. The current statements are cleared.
/// # Arguments
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///  and thus always is the id of the next block.
/// * `cur_stmt` - The list of scheduling statements that are part of this block.
/// * `term` - The terminator for the block.
/// * `join_edge` - The edge to use for a basic block to join back to the parent.
/// This is the next block in the parent stack frame.
/// * `cont_block` - The block whose live-out set is the return arguments of this block. This is
/// either the block itself (`None`), or the block's continuation.
/// * `src_loc` - The source location of the block.
/// # Returns
/// The newly created basic block.
fn make_block(
    cur_id: &mut usize,
    cur_stmt: &mut Vec<SchedStmt>,
    term: Terminator,
    cont_block: Option<usize>,
    src_loc: Info,
) -> BasicBlock {
    let res = BasicBlock {
        id: *cur_id,
        stmts: stmts_to_hir(std::mem::take(cur_stmt)),
        terminator: term,
        ret_block: cont_block,
        src_loc,
    };
    *cur_id += 1;
    res
}

/// A pending child block to be processed and converted into a sequence of basic
/// blocks. This is used to process blocks in a BFS manner.
struct PendingChild {
    parent_id: usize,
    /// The id of the block that will run when the children are done.
    join_id: usize,
    /// Left child (true branch)
    true_block: Vec<SchedStmt>,
    /// Right child (false branch). May be `None` if the parent is not an `if`
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
                tag,
                true_block,
                false_block,
                info,
            } => res.push(SchedStmt::If {
                guard,
                tag,
                true_block: flatten_stmts(true_block),
                false_block: flatten_stmts(false_block),
                info,
            }),
            other => res.push(other),
        }
    }
    res
}

/// Handles a return statement by constructing a new block with the return as the terminator
/// and incrementing the id counter.
/// # Arguments
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///   and thus always is the id of the next block.
/// * `blocks` - The list of blocks to add to. New blocks are appended to the end.
/// * `edges` - The list of edges to add to. Edges for new blocks are added to this map.
/// * `cur_stmts` - The list of scheduling statements to convert to blocks.
/// * `sched_expr` - The expression to return.
/// * `join_edge` - The edge to use for a basic block to join back to the parent.
/// * `end` - The source location of the return statement.
fn handle_return(
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
    cur_stmts: &mut Vec<SchedStmt>,
    sched_expr: SchedExpr,
    end: Info,
) {
    let old_id = *cur_id;
    let info = Info::new_range(
        cur_stmts.first().map_or(&end, |x: &SchedStmt| x.get_info()),
        &end,
    );
    blocks.insert(
        *cur_id,
        make_block(
            cur_id,
            cur_stmts,
            Terminator::Return(expr_to_multi_node_id(sched_expr)),
            None,
            info,
        ),
    );
    edges.insert(old_id, Edge::Next(FINAL_BLOCK_ID));
}

/// Handles a select statement by constructing a new block with the select as the terminator
/// and incrementing the id counter.
/// # Arguments
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///   and thus always is the id of the next block.
/// * `blocks` - The list of blocks to add to. New blocks are appended to the end.
/// * `cur_stmts` - The list of scheduling statements to convert to blocks.
/// * `guard` - The guard expression for the select statement.
/// * `tag` - The tag for the select statement.
/// * `true_block` - The list of scheduling statements for the true branch of the select.
/// * `false_block` - The list of scheduling statements for the false branch of the select.
/// * `end_info` - The source location of the select statement.
/// * `children` - The list of pending children to add to that will queue up the
///  children of this block to be processed in the next BFS level.
// TODO: cleanup
#[allow(clippy::too_many_arguments)]
fn handle_select(
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    cur_stmts: &mut Vec<SchedStmt>,
    guard: SchedExpr,
    tag: Option<Vec<crate::parse::ast::Tag>>,
    true_block: Vec<SchedStmt>,
    false_block: Vec<SchedStmt>,
    end_info: Info,
    children: &mut Vec<PendingChild>,
) {
    let parent_id = *cur_id;
    let info = Info::new_range(
        cur_stmts.first().map_or(&end_info, |x| x.get_info()),
        &end_info,
    );
    blocks.insert(
        *cur_id,
        make_block(
            cur_id,
            cur_stmts,
            Terminator::Select(expr_to_node_id(guard), tag),
            Some(*cur_id + 1),
            info,
        ),
    );
    //  `cur_id` incremented in `make_block`, so it currently points to the
    // next block (continuation)
    children.push(PendingChild {
        parent_id,
        join_id: *cur_id,
        true_block,
        false_block: Some(false_block),
    });
}

/// Handles a call statement by constructing a new block with the call as the terminator
/// and incrementing the id counter.
/// # Arguments
/// * `edges` - The list of edges to add to. Edges for new blocks are added to this map.
/// * `blocks` - The list of blocks to add to. New blocks are appended to the end.
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///    and thus always is the id of the next block.
/// * `cur_stmts` - The list of scheduling statements to convert to blocks.
/// * `lhs` - The left hand side of the call statement. This is the list of variables to
///   assign the return values to.
/// * `call` - The call statement to add to a block.
/// * `join_edge` - The edge to use for a basic block to join back to the parent.
/// * `info` - The source location of the call statement.
#[allow(clippy::too_many_arguments)]
fn handle_call(
    edges: &mut HashMap<usize, Edge>,
    blocks: &mut HashMap<usize, BasicBlock>,
    cur_id: &mut usize,
    cur_stmts: &mut Vec<SchedStmt>,
    lhs: Vec<(String, Option<FullType>)>,
    call: SchedFuncCall,
    info: Info,
) {
    let info = Info::new_range(
        cur_stmts
            .first()
            .map_or(&info, |x: &SchedStmt| x.get_info()),
        &info,
    );
    edges.insert(*cur_id, Edge::Next(*cur_id + 1));
    blocks.insert(
        *cur_id,
        make_block(
            cur_id,
            cur_stmts,
            Terminator::Call(lhs, call.try_into().unwrap()),
            Some(*cur_id + 1),
            info,
        ),
    );
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

    // we do BFS so that we know the continuation of a block before we
    // process the children of the block
    let mut children = vec![];
    // if the last statement is a return or call then we don't need to make another
    // empty block for the continuation
    let ends_with_tail = stmts.last().map_or(false, |x| {
        matches!(x, SchedStmt::Return(..) | SchedStmt::Call(..))
    });
    let mut last_info = Info::default();
    for stmt in stmts {
        match stmt {
            SchedStmt::Return(end, sched_expr) => {
                last_info = end;
                handle_return(cur_id, blocks, edges, &mut cur_stmts, sched_expr, end);
            }
            SchedStmt::If {
                guard,
                tag,
                true_block,
                false_block,
                info: end_info,
            } => {
                last_info = *false_block.last().map_or_else(
                    || true_block.last().map_or(&end_info, |s| s.get_info()),
                    |s| s.get_info(),
                );
                handle_select(
                    cur_id,
                    blocks,
                    &mut cur_stmts,
                    guard,
                    tag,
                    true_block,
                    false_block,
                    end_info,
                    &mut children,
                );
            }
            SchedStmt::Decl {
                info,
                lhs,
                is_const: _,
                expr: Some(SchedExpr::Term(SchedTerm::Call(_, call))),
            } => {
                last_info = info;
                handle_call(edges, blocks, cur_id, &mut cur_stmts, lhs, call, info);
            }
            SchedStmt::Call(end, call_info) => {
                last_info = end;
                handle_call(
                    edges,
                    blocks,
                    cur_id,
                    &mut cur_stmts,
                    vec![],
                    call_info,
                    end,
                );
            }
            // not a tail edge
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
        let info = cur_stmts.last().map_or(last_info, |x| *x.get_info());
        blocks.insert(
            *cur_id,
            make_block(cur_id, &mut cur_stmts, Terminator::None, None, info),
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
            // if false branch is none, then this isn't an `if`
            edges.insert(parent_id, Edge::Next(true_branch));
        }
    }
}

impl Cfg {
    /// Create a new CFG from a list of scheduling statements
    /// # Arguments
    /// * `stmts` - The list of scheduling statements to convert to blocks.
    /// * `output_len` - The number of outputs of the scheduling function.
    pub fn new(stmts: Vec<SchedStmt>, output_len: usize) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            FINAL_BLOCK_ID,
            BasicBlock {
                id: FINAL_BLOCK_ID,
                stmts: vec![],
                terminator: Terminator::FinalReturn(output_len),
                ret_block: None,
                src_loc: Info::default(),
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
        compute_coninuations(
            Self {
                blocks,
                transpose_graph: Self::transpose(&edges),
                graph: edges,
            }
            .remove_unreachable(),
        )
    }

    /// Transposes a CFG
    fn transpose(graph: &HashMap<usize, Edge>) -> HashMap<usize, BTreeSet<usize>> {
        let mut res = HashMap::new();
        for (id, edge) in graph {
            match edge {
                Edge::Next(next) => {
                    res.entry(*next).or_insert_with(BTreeSet::new).insert(*id);
                }
                Edge::Select {
                    true_branch,
                    false_branch,
                } => {
                    res.entry(*true_branch)
                        .or_insert_with(BTreeSet::new)
                        .insert(*id);
                    res.entry(*false_branch)
                        .or_insert_with(BTreeSet::new)
                        .insert(*id);
                }
                Edge::None => (),
            }
        }
        res
    }

    /// Gets the successors of a given block.
    /// The returned block ids contains no duplicates.
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

    /// Gets the predecessors of a given block.
    /// The returned block ids contains no duplicates and is sorted.
    pub fn predecessors(&self, block_id: usize) -> Vec<usize> {
        self.transpose_graph
            .get(&block_id)
            .map_or(vec![], |x| x.iter().copied().collect())
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
        self.transpose_graph = Self::transpose(&self.graph);
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

/// Converts an expression to a list of node Ids, assuming the expression is just a variable
/// or a tuple of variables
fn expr_to_multi_node_id(e: SchedExpr) -> Vec<String> {
    if let SchedExpr::Term(SchedTerm::Lit {
        lit: SchedLiteral::Tuple(lits),
        ..
    }) = e
    {
        lits.into_iter().map(expr_to_node_id).collect()
    } else {
        vec![expr_to_node_id(e)]
    }
}
