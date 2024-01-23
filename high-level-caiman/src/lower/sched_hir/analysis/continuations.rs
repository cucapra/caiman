//! Contains functions to compute continuations and *pretinuations* of each block
//! in the CFG. Given a block `B`, the continuation of `B` is the block that
//! is a common successor to all successors of `B` such that the path from
//! `B` to the continuation is the shortest out of all such common successors.
//!
//! The *pretinuation* of a block is basically the continuation in the inverse
//! CFG. Given a block `B`, the pretinuation of `B` is a block, `C`, such that
//! `B` is the continuation of `C`.

#![allow(clippy::module_name_repetitions)]
use std::collections::{HashMap, HashSet, VecDeque};

use crate::lower::sched_hir::cfg::{Cfg, NextSet};

/// Successor information. We consider a block to be a successor of itself
/// in this data structure.
#[derive(Debug, Clone, Default)]
pub struct Succs {
    /// A map from each block to the transitive closure of nodes that are
    /// successors of the key. We consider a block to be a successor of itself
    /// in this map.
    pub succs: HashMap<usize, HashSet<usize>>,
    /// A map from each block to the transitive closure of nodes that are
    /// predecessors of the key. We consider a block to be a predecessor of
    /// itself in this map.
    pub preds: HashMap<usize, HashSet<usize>>,
}

impl Succs {
    pub fn new(preds: HashMap<usize, HashSet<usize>>) -> Self {
        let mut res: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (dominated, dominators) in &preds {
            for dominator in dominators {
                res.entry(*dominator)
                    .or_insert_with(HashSet::new)
                    .insert(*dominated);
            }
        }
        Self { succs: res, preds }
    }
}

/// Computes ALL successors and predecessors of each block in the CFG
/// # Arguments
/// * `cfg` - The forward graph
/// * `preds` - A map from each block to its predecessors
/// # Panics
#[must_use]
#[allow(clippy::module_name_repetitions)]
fn compute_sucessors<T, U>(cfg: &Cfg, graph: &HashMap<usize, U>, preds: &HashMap<usize, T>) -> Succs
where
    T: NextSet,
    U: NextSet,
{
    // Same way as computing dominators except we use union instead of intersection
    // and we start with the empty set instead of the set of all nodes.

    // Map from each block to nodes that can reach it
    let mut pred_map: HashMap<_, HashSet<_>> = HashMap::new();
    for block in graph.keys() {
        let mut h = HashSet::new();
        h.insert(*block);
        pred_map.insert(*block, h);
    }
    let mut changed = true;
    while changed {
        changed = false;
        for block in cfg.blocks.keys() {
            if let Some(preds) = preds.get(block) {
                let preds = preds.next_set();
                let mut pred_iter = preds.into_iter();
                let mut new_preds: HashSet<usize> = pred_iter
                    .next()
                    .map(|x| pred_map.get(&x).unwrap().clone())
                    .unwrap_or_default();
                for pred in pred_iter {
                    new_preds = new_preds
                        .union(pred_map.get(&pred).unwrap())
                        .copied()
                        .collect();
                }
                new_preds.insert(*block);
                if new_preds != *pred_map.get(block).unwrap_or(&HashSet::new()) {
                    pred_map.insert(*block, new_preds);
                    changed = true;
                }
            }
        }
    }
    Succs::new(pred_map)
}

/// Finds the shortest path of every node, starting from the start nodes
///
/// # Returns
/// * A map from each node to the length of the shortest path to it.
fn shortest_path<T: NextSet>(graph: &HashMap<usize, T>, start_id: usize) -> HashMap<usize, usize> {
    // BFS
    let mut res = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_id, 0));
    while let Some((block, dist)) = queue.pop_front() {
        if res.contains_key(&block) {
            // Already visited, so we've already found the shortest path
            // since this is BFS on a DAG
            continue;
        }
        res.insert(block, dist);
        for next in graph.get(&block).as_ref().unwrap().next_set() {
            queue.push_back((next, dist + 1));
        }
    }
    res
}

/// Computes the continuation of each block in the CFG and stores this
/// in the `ret_block` field of every basic block. A block with no
/// continuation has no successors.
///
/// We compute continuations as a second pass to support any
/// structure changes of the CFG. If the CFG structure
/// never changes, the the continuations determined by frontend CFG gen
/// will work fine.
#[must_use]
pub fn compute_continuations(mut cfg: Cfg) -> Cfg {
    let (merge_points, succs) = compute_merge_points(&cfg, &cfg.graph, &cfg.transpose_graph);
    for (block_id, block) in &mut cfg.blocks {
        block.ret_block = merge_points.get(block_id).copied();
    }
    cfg.succs = succs;
    cfg
}

/// Returns a map from block id, `A`, to the id of the block, `B` such that
/// `A` is the continuation of `B`.
#[must_use]
pub fn compute_pretinuations(cfg: &Cfg) -> HashMap<usize, usize> {
    compute_merge_points(cfg, &cfg.transpose_graph, &cfg.graph).0
}

/// Computes "merge points" of every block in the CFG. Given a block `B`, a
/// merge point is the block that is the successor of all of the `B`'s successors
/// such that the path from `B` to that successor is the shortest.
///
/// In this definition, by successor we consider a block to be its own successor.
///
/// # Arguments
/// * `cfg` - The CFG
/// * `graph` - The forward graph (map from each block to its immediate successors)
/// * `preds` - A map from each block to its immediate predecessors
fn compute_merge_points<T: NextSet, U: NextSet>(
    cfg: &Cfg,
    graph: &HashMap<usize, T>,
    preds: &HashMap<usize, U>,
) -> (HashMap<usize, usize>, Succs) {
    let succs = compute_sucessors(cfg, graph, preds);
    let mut res = HashMap::new();

    // The continuation is a the successor that is a successor of all of the
    // block's successors such that the path to that successor is the shortest
    for (block_id, edge) in graph {
        let mut successors = HashSet::new();
        for succ in edge.next_set() {
            if successors.is_empty() {
                successors = succs.succs.get(&succ).unwrap().clone();
            } else {
                successors = successors
                    .intersection(succs.succs.get(&succ).unwrap())
                    .copied()
                    .collect();
            }
        }
        let paths = shortest_path(graph, *block_id);
        let mut ordered_succs: Vec<usize> = successors.iter().copied().collect();
        ordered_succs.sort_by_key(|x| paths.get(x).unwrap());
        if let Some(cont_id) = ordered_succs.first() {
            res.insert(*block_id, *cont_id);
        }
    }
    (res, succs)
}
