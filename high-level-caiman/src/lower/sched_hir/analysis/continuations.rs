use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::lower::sched_hir::cfg::{Cfg, Edge, START_BLOCK_ID};

struct Succs {
    /// A map from each block to the transitive closure of nodes that are
    /// successors of the key. We consider a block to be a successor of itself.
    pub succs: HashMap<usize, HashSet<usize>>,
    /// A map from each block to the transitive closure of nodes that are
    /// predecessors of the key. We consider a block to be a predecessor of
    /// itself.
    #[allow(dead_code)]
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
/// * `cfg` - The CFG
/// # Panics
#[must_use]
#[allow(clippy::module_name_repetitions)]
fn compute_sucessors(cfg: &Cfg, preds: &HashMap<usize, BTreeSet<usize>>) -> Succs {
    // Same way as computing dominators except we use union instead of intersection
    // and we start with the empty set instead of the set of all nodes.

    // Map from each block to nodes that can reach it
    let mut pred_map: HashMap<_, HashSet<_>> = HashMap::new();
    for block in cfg.graph.keys() {
        let mut h = HashSet::new();
        h.insert(*block);
        pred_map.insert(*block, h);
    }
    let mut changed = true;
    let default_preds = BTreeSet::new();
    while changed {
        changed = false;
        for block in cfg.blocks.keys() {
            let mut pred_iter = preds.get(block).unwrap_or(&default_preds).iter();
            let mut new_preds: HashSet<usize> = pred_iter
                .next()
                .map(|x| pred_map.get(x).unwrap().clone())
                .unwrap_or_default();
            for pred in pred_iter {
                new_preds = new_preds
                    .union(pred_map.get(pred).unwrap())
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
    Succs::new(pred_map)
}

/// Finds the shortest path of every node, starting from the start nodes
///
/// # Returns
/// * A map from each node to the length of the shortest path to it.
fn shortest_path(cfg: &Cfg) -> HashMap<usize, usize> {
    // BFS
    let mut res = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back((START_BLOCK_ID, 0));
    while let Some((block, dist)) = queue.pop_front() {
        if res.contains_key(&block) {
            // Already visited, so we've already found the shortest path
            // since this is BFS on a DAG
            continue;
        }
        res.insert(block, dist);
        match cfg.graph.get(&block).unwrap() {
            Edge::Next(next) => queue.push_back((*next, dist + 1)),
            Edge::Select {
                true_branch,
                false_branch,
            } => {
                queue.push_back((*true_branch, dist + 1));
                queue.push_back((*false_branch, dist + 1));
            }
            Edge::None => {}
        }
    }
    res
}

/// Computes the continuation of each block in the CFG. A block with no
/// continuation has no successors.
pub fn compute_coninuations(mut cfg: Cfg) -> Cfg {
    let succs = compute_sucessors(&cfg, &cfg.transpose_graph);
    let paths = shortest_path(&cfg);

    // The continuation is a the successor that is a successor of all of the
    // block's successors such that the path to that successor is the shortest
    for (block_id, edge) in &cfg.graph {
        let mut successors = HashSet::new();
        for succ in edge.targets() {
            if successors.is_empty() {
                successors = succs.succs.get(&succ).unwrap().clone();
            } else {
                successors = successors
                    .intersection(succs.succs.get(&succ).unwrap())
                    .copied()
                    .collect();
            }
        }
        let mut ordered_doms: Vec<usize> = successors.iter().copied().collect();
        ordered_doms.sort_by_key(|x| paths.get(x).unwrap());
        cfg.blocks.get_mut(block_id).unwrap().ret_block = ordered_doms.first().copied();
    }

    cfg
}
