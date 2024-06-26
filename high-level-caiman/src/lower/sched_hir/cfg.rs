//! Builds a cfg from the AST. The CFG will have a unique start and end block,
//! and be constructed in breadth-first order so that every block knows
//! its continuation when its constructed.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    enum_cast,
    error::Info,
    parse::ast::{FullType, SchedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm, Tags},
    typing::Context,
};

use super::{
    analysis::{compute_continuations, Succs},
    stmts_to_hir, Hir, HirBody, HirFuncCall, Terminator, TripleTag,
};

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

impl BasicBlock {
    /// Gets the source location of the first statement in the block
    pub fn get_starting_info(&self) -> Info {
        self.stmts.first().map_or(self.src_loc, Hir::get_info)
    }

    pub fn get_final_info(&self) -> Info {
        self.terminator.get_info()
    }
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

    /// Gets the single target of the edge, if it exists
    pub const fn next(&self) -> Option<usize> {
        match self {
            Self::Next(id) => Some(*id),
            _ => None,
        }
    }
}

/// Something which can be converted to a collection of block ids.
pub trait NextSet {
    /// Gets the collection of block ids as a vector.
    fn next_set(&self) -> Vec<usize>;
}

impl NextSet for Edge {
    fn next_set(&self) -> Vec<usize> {
        self.targets()
    }
}

impl NextSet for BTreeSet<usize> {
    fn next_set(&self) -> Vec<usize> {
        self.iter().copied().collect()
    }
}

/// A control flow graph for a scheduling function.
/// Each basic block in the CFG contains HIR body statements and a terminator.
pub struct Cfg {
    pub blocks: HashMap<usize, BasicBlock>,
    pub(super) graph: HashMap<usize, Edge>,
    pub(super) transpose_graph: HashMap<usize, BTreeSet<usize>>,
    pub(super) succs: Succs,
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
    /// The names of the return variables that the children blocks will return.
    ret_names: Vec<(String, Option<Tags>)>,
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
            SchedStmt::Seq {
                info,
                dests,
                block,
                is_const,
            } => match *block {
                SchedStmt::Block(_, mut stmts) => {
                    if !stmts.is_empty()
                        && matches!(stmts.last().as_ref().unwrap(), SchedStmt::Return(..))
                    {
                        if let SchedStmt::Return(info, ret_expr) = stmts.pop().unwrap() {
                            res.extend(flatten_stmts(stmts));
                            res.push(SchedStmt::Decl {
                                info,
                                lhs: dests,
                                is_const,
                                expr: Some(ret_expr),
                            });
                        } else {
                            unreachable!()
                        }
                    } else {
                        panic!("{info}: Empty block assigned to variables {dests:?}");
                    }
                }
                SchedStmt::If {
                    guard,
                    tag,
                    true_block,
                    false_block,
                    info: if_info,
                } => res.push(SchedStmt::Seq {
                    info,
                    dests,
                    block: Box::new(SchedStmt::If {
                        guard,
                        tag,
                        true_block: flatten_stmts(true_block),
                        false_block: flatten_stmts(false_block),
                        info: if_info,
                    }),
                    is_const,
                }),
                _ => panic!("{info}: Sequence block is neither a Block or If"),
            },
            other => res.push(other),
        }
    }
    res
}

fn ast_to_hir_named_tags(tags: Vec<(String, Option<Tags>)>) -> Vec<(String, TripleTag)> {
    tags.into_iter()
        .map(|(n, t)| (n, TripleTag::from_owned_opt(t)))
        .collect()
}

fn ast_to_hir_fulltype(tags: Vec<(String, Option<FullType>)>) -> Vec<(String, TripleTag)> {
    tags.into_iter()
        .map(|(n, t)| (n, TripleTag::from_owned_opt(t.map(|t| t.tags))))
        .collect()
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
/// * `ret_names` - The names of the return variables to write to
#[allow(clippy::too_many_arguments)]
fn handle_return(
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
    cur_stmts: &mut Vec<SchedStmt>,
    sched_expr: SchedExpr,
    join_edge: Edge,
    end: Info,
    ret_names: Vec<(String, Option<Tags>)>,
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
            Terminator::Return {
                info: end,
                dests: ast_to_hir_named_tags(ret_names),
                rets: expr_to_multi_node_id(sched_expr),
                passthrough: vec![],
            },
            None,
            info,
        ),
    );
    edges.insert(old_id, join_edge);
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
    dests: Vec<(String, Option<Tags>)>,
) {
    let parent_id = *cur_id;
    let info = Info::new_range(
        cur_stmts.first().map_or(&end_info, |x| x.get_info()),
        &end_info,
    );
    let ret_names = dests.clone();
    blocks.insert(
        *cur_id,
        make_block(
            cur_id,
            cur_stmts,
            Terminator::Select {
                info: end_info,
                dests: ast_to_hir_named_tags(dests),
                guard: expr_to_node_id(guard),
                tag: TripleTag::from_owned_opt(tag),
            },
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
        ret_names,
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
    end_info: Info,
) {
    let info = Info::new_range(
        cur_stmts
            .first()
            .map_or(&end_info, |x: &SchedStmt| x.get_info()),
        &end_info,
    );
    edges.insert(*cur_id, Edge::Next(*cur_id + 1));
    if call.yield_call {
        blocks.insert(
            *cur_id,
            make_block(
                cur_id,
                cur_stmts,
                Terminator::Yield(end_info, vec![]),
                Some(*cur_id + 1),
                info,
            ),
        );
        edges.insert(*cur_id, Edge::Next(*cur_id + 1));
        blocks.insert(
            *cur_id,
            make_block(
                cur_id,
                &mut vec![],
                Terminator::Call(ast_to_hir_fulltype(lhs), HirFuncCall::new(call)),
                Some(*cur_id + 1),
                info,
            ),
        );
    } else {
        blocks.insert(
            *cur_id,
            make_block(
                cur_id,
                cur_stmts,
                Terminator::Call(ast_to_hir_fulltype(lhs), HirFuncCall::new(call)),
                Some(*cur_id + 1),
                info,
            ),
        );
    }
}

/// Handles a sequence statement by constructing a new block with the sequence as the terminator
/// and incrementing the id counter. This also adds an in edge annotation to the continuation
/// block (the resulting `cur_stmts`).
/// # Arguments
/// * `block` - The sequence statement to handle.
/// * `cur_stmts` - The list of scheduling statements to convert to blocks. Will be
/// cleared for the continuation block.
/// * `info` - The source location of the sequence statement.
/// * `dests` - The list of variables to assign the return values to.
/// * `cur_id` - The id of the next block. For every new block created, this is incremented
///   and thus always is the id of the next block.
/// * `blocks` - The list of blocks to add to. New blocks are appended to the end.
/// * `children` - The list of pending children to add to that will queue up the
/// children of this block to be processed in the next BFS level.
/// * `last_info` - Will become the source location of the last statement in the sequence.
#[allow(clippy::too_many_arguments)]
fn handle_seq(
    block: Box<SchedStmt>,
    cur_stmts: &mut Vec<SchedStmt>,
    info: Info,
    dests: Vec<(String, Option<FullType>)>,
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    children: &mut Vec<PendingChild>,
    last_info: &mut Info,
) {
    if let SchedStmt::If {
        guard,
        tag,
        true_block,
        false_block,
        info: if_info,
    } = *block
    {
        *last_info = *false_block.last().map_or_else(
            || true_block.last().map_or(&if_info, |s| s.get_info()),
            |s| s.get_info(),
        );
        let dests: Vec<_> = dests
            .into_iter()
            .map(|(s, t)| (s, t.map(|t| t.tags)))
            .collect();
        handle_select(
            cur_id,
            blocks,
            cur_stmts,
            guard,
            tag,
            true_block,
            false_block,
            if_info,
            children,
            dests.clone(),
        );
        // add an in edge annotation to the continuation block
        // the destination of the select is a meet point, so we need to add
        // an in edge annotation to the continuation block
        assert!(cur_stmts.is_empty());
        cur_stmts.push(SchedStmt::InEdgeAnnotation {
            info,
            tags: dests
                .into_iter()
                .filter_map(|(s, t)| t.map(|t| (s, t)))
                .collect(),
        });
    } else {
        panic!("{info}: Flattened sequence is not an If");
    }
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
/// * `ret_names` - The names of the return variables to write to in order to return
///  from the current scope.
#[allow(clippy::too_many_lines)]
fn make_blocks(
    cur_id: &mut usize,
    blocks: &mut HashMap<usize, BasicBlock>,
    edges: &mut HashMap<usize, Edge>,
    stmts: Vec<SchedStmt>,
    join_edge: Edge,
    ret_names: &[(String, Option<Tags>)],
    ctx: &Context,
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
                handle_return(
                    cur_id,
                    blocks,
                    edges,
                    &mut cur_stmts,
                    sched_expr,
                    join_edge,
                    end,
                    ret_names.to_vec(),
                );
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
                    vec![],
                );
            }
            SchedStmt::Seq {
                info,
                dests,
                block,
                is_const: _,
            } => {
                // TODO: handle mutable destinations
                handle_seq(
                    block,
                    &mut cur_stmts,
                    info,
                    dests,
                    cur_id,
                    blocks,
                    &mut children,
                    &mut last_info,
                );
            }
            SchedStmt::Decl {
                info,
                lhs,
                is_const: _,
                expr: Some(SchedExpr::Term(SchedTerm::Call(_, call))),
            } if !ctx.externs.contains(
                if let SchedExpr::Term(SchedTerm::Var { name, .. }) = &*call.target {
                    name
                } else {
                    unreachable!()
                },
            ) =>
            {
                last_info = info;
                handle_call(edges, blocks, cur_id, &mut cur_stmts, lhs, call, info);
            }
            SchedStmt::Call(end, call_info)
                if !ctx.externs.contains(
                    if let SchedExpr::Term(SchedTerm::Var { name, .. }) = &*call_info.target {
                        name
                    } else {
                        unreachable!()
                    },
                ) =>
            {
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
            make_block(cur_id, &mut cur_stmts, Terminator::None(info), None, info),
        );
        edges.insert(old_id, join_edge);
    }
    make_child_blocks(children, cur_id, blocks, edges, ctx);
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
    ctx: &Context,
) {
    for PendingChild {
        parent_id,
        join_id,
        true_block,
        false_block,
        ret_names,
    } in children
    {
        let join_edge = Edge::Next(join_id);
        let true_branch = make_blocks(
            cur_id, blocks, edges, true_block, join_edge, &ret_names, ctx,
        );
        let false_branch =
            false_block.map(|f| make_blocks(cur_id, blocks, edges, f, join_edge, &ret_names, ctx));
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
    pub fn new(stmts: Vec<SchedStmt>, outputs: &[FullType], ctx: &Context) -> Self {
        use crate::lower::sched_hir::RET_VAR;
        let mut blocks = HashMap::new();
        blocks.insert(
            FINAL_BLOCK_ID,
            BasicBlock {
                id: FINAL_BLOCK_ID,
                stmts: vec![],
                terminator: Terminator::FinalReturn(
                    stmts.last().map_or_else(Info::default, |x| *x.get_info()),
                    (0..outputs.len())
                        .map(|id| format!("{RET_VAR}{id}"))
                        .collect(),
                ),
                ret_block: None,
                src_loc: stmts.last().map_or_else(Info::default, |x| *x.get_info()),
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
            &outputs
                .iter()
                .enumerate()
                .map(|(id, typ)| (format!("{RET_VAR}{id}"), Some(typ.tags.clone())))
                .collect::<Vec<_>>(),
            ctx,
        );
        compute_continuations(
            Self {
                blocks,
                transpose_graph: Self::transpose(&edges),
                graph: edges,
                succs: Succs::default(),
            }
            .remove_unreachable(),
        )
    }

    /// Transposes a CFG
    fn transpose(graph: &HashMap<usize, Edge>) -> HashMap<usize, BTreeSet<usize>> {
        let mut res = HashMap::new();
        for (id, edge) in graph {
            res.entry(*id).or_insert_with(BTreeSet::new);
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

    /// Return true if `block_id` is the sole predecessor of `FINAL_BLOCK_ID`.
    pub fn is_final_return(&self, block_id: usize) -> bool {
        let preds = self.predecessors(FINAL_BLOCK_ID);
        preds.len() == 1 && preds.first() == Some(&block_id)
    }

    /// Gets the block id of the block whose outputs must match the given block.
    /// The returned block is the continuation of the given block, unless the given
    /// block returns control flow back to a parent such as the children of a
    /// select. In this, case, the returned block is the given block.
    ///
    /// See also logic in `Funclets::output_vars`
    pub fn get_continuation_output_block(&self, block_id: usize) -> usize {
        if block_id == FINAL_BLOCK_ID {
            return FINAL_BLOCK_ID;
        }
        match &self.blocks[&block_id].terminator {
            Terminator::Call(..)
            | Terminator::CaptureCall { .. }
            | Terminator::Select { .. }
            | Terminator::Yield(..) => {
                self.get_continuation_output_block(self.blocks[&block_id].ret_block.unwrap())
            }
            Terminator::Return { .. } if self.is_final_return(block_id) => {
                // final return is a jump to final basic block
                self.get_continuation_output_block(self.blocks[&block_id].ret_block.unwrap())
            }
            Terminator::FinalReturn(..)
            | Terminator::None(..)
            | Terminator::Next(..)
            | Terminator::Return { .. } => block_id,
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
