//! Computes dominators of a CFG.
//! Mostly taken from
//! [here](https://github.com/stephenverderame/cs6120-bril/blob/main/cfg/src/analysis/dominators.rs)
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::lower::sched_hir::cfg::{BasicBlock, Cfg};

/// A node in the dominator tree.
#[derive(Clone)]
pub struct DomNode {
    /// The block id
    #[allow(dead_code)]
    block: usize,
    /// The blocks immediately dominated by `block`
    dominated: Vec<usize>,
}

impl DomNode {
    /// Returns the blocks strictly dominated by `self`
    /// # Arguments
    /// * `nodes` - A map from each block id to `DomTree` node
    /// * `doms` - The accumulated set of blocks dominated by `self`
    fn dominated(&self, nodes: &HashMap<usize, Self>, mut doms: HashSet<usize>) -> HashSet<usize> {
        for &dom in &self.dominated {
            if doms.insert(dom) {
                doms = nodes[&dom].dominated(nodes, doms);
            }
        }
        doms
    }
}

/// A dominator tree. The dominator tree is a tree where each node
/// immediately dominates its children.
#[derive(Clone)]
pub struct DomTree {
    /// A map from each block id to `DomTree` node
    nodes: HashMap<usize, DomNode>,
}

impl DomTree {
    /// Constructs a new dom tree
    /// # Arguments
    /// * `doms` - A map from each block to nodes that dominate it
    /// # Returns
    /// * The dom tree
    fn new(doms_map: &HashMap<usize, HashSet<usize>>) -> Self {
        let mut nodes = doms_map
            .keys()
            .copied()
            .map(|k| {
                (
                    k,
                    DomNode {
                        block: k,
                        dominated: Vec::new(),
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        for (block, block_dominators) in doms_map {
            for dominator in block_dominators.iter().filter(|x| **x != *block) {
                // all the nodes that `dominator` dominates
                let dominated = doms_map
                    .iter()
                    .filter_map(|(k, v)| {
                        if v.contains(dominator) && *k != *dominator {
                            Some(*k)
                        } else {
                            None
                        }
                    })
                    .collect::<HashSet<_>>();

                // > 1 because `block` should be in both sets
                if dominated.intersection(doms_map.get(block).unwrap()).count() > 1 {
                    // dominator dominates a node which is also a
                    // dominator of `block`

                    // so `dominator` is not a strict dominator
                    continue;
                }
                nodes.get_mut(dominator).unwrap().dominated.push(*block);
            }
        }
        Self { nodes }
    }

    /// Computes the dominance frontier of a block
    /// The dominance frontier of a block `b` is the set of blocks `d` such that
    /// `d` is a successor of a block `s` dominated by `b` but `d` is not dominated by `b`
    /// # Arguments
    /// * `block` - The block
    /// * `cfg` - The CFG
    /// # Returns
    /// * The dominance frontier of `block`
    #[must_use]
    pub fn dom_frontier(&self, block: usize, cfg: &Cfg) -> Vec<usize> {
        use std::iter;
        let dominated = self.nodes[&block].dominated(&self.nodes, HashSet::new());
        let mut frontier = Vec::new();
        for dom in dominated.iter().chain(iter::once(&block)) {
            let dom = *dom;
            let dom_succs = cfg.graph[&dom].targets();
            for succ in dom_succs {
                if !dominated.contains(&succ) {
                    frontier.push(succ);
                }
            }
        }
        frontier
    }

    /// Returns the blocks immediately (and strictly) dominated by `block`
    #[must_use]
    pub fn immediately_dominated(&self, block: usize) -> HashSet<usize> {
        self.nodes[&block].dominated.iter().copied().collect()
    }

    /// Returns all the blocks dominated by `block`
    #[must_use]
    pub fn dominated(&self, block: usize) -> HashSet<usize> {
        let mut s = self.nodes[&block].dominated(&self.nodes, HashSet::new());
        s.insert(block);
        s
    }
}

/// Computes the dominators of each block in the CFG
/// # Arguments
/// * `cfg` - The CFG
/// # Returns
/// * A dominator tree
/// # Panics
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn compute_dominators(cfg: &Cfg) -> DomTree {
    DomTree::new(&make_dom_map(&cfg.transpose_graph, &cfg.blocks))
}

/// Returns a map from blocks to nodes that dominate it
fn make_dom_map<T>(
    preds: &HashMap<usize, T>,
    blocks: &HashMap<usize, BasicBlock>,
) -> HashMap<usize, HashSet<usize>>
where
    for<'a> &'a T: IntoIterator<Item = &'a usize>,
{
    let mut doms: HashMap<_, HashSet<_>> = HashMap::new();
    let all_blocks = blocks.keys().copied().collect::<HashSet<_>>();
    for block in blocks.keys() {
        doms.insert(*block, all_blocks.clone());
    }
    let mut changed = true;
    while changed {
        changed = false;
        for block in blocks.keys() {
            if let Some(pred_iter) = preds.get(block) {
                let mut pred_iter = pred_iter.into_iter();
                let mut new_dom: HashSet<usize> = pred_iter
                    .next()
                    .map(|x| doms.get(x).unwrap().clone())
                    .unwrap_or_default();
                for pred in pred_iter {
                    new_dom = new_dom
                        .intersection(doms.get(pred).unwrap())
                        .copied()
                        .collect();
                }
                new_dom.insert(*block);
                if new_dom != *doms.get(block).unwrap_or(&HashSet::new()) {
                    doms.insert(*block, new_dom);
                    changed = true;
                }
            }
        }
    }
    doms
}

pub struct DomInfo {
    /// A map from each block to nodes that dominate it
    dominated_by: HashMap<usize, HashSet<usize>>,
    /// A map from each block to nodes that it dominates
    dominates: HashMap<usize, HashSet<usize>>,
    /// A map from each block to nodes that postdominate it
    postdominated_by: HashMap<usize, HashSet<usize>>,
    /// A map from each block to nodes that it postdominates
    postdominates: HashMap<usize, HashSet<usize>>,
}

fn inverse_map(map: &HashMap<usize, HashSet<usize>>) -> HashMap<usize, HashSet<usize>> {
    let mut res: HashMap<_, HashSet<_>> = HashMap::new();
    for (key, vals) in map {
        for v in vals.iter() {
            res.entry(*v).or_default().insert(*key);
        }
    }
    res
}

impl DomInfo {
    pub fn new(cfg: &Cfg) -> Self {
        let dominated_by = make_dom_map(&cfg.transpose_graph, &cfg.blocks);
        let dominates = inverse_map(&dominated_by);
        let postdominated_by = make_dom_map(&cfg.graph, &cfg.blocks);
        let postdominates = inverse_map(&postdominated_by);
        Self {
            dominated_by,
            dominates,
            postdominated_by,
            postdominates,
        }
    }

    pub fn dom(&self, a: usize, b: usize) -> bool {
        self.dominated_by[&b].contains(&a)
    }

    pub fn postdom(&self, a: usize, b: usize) -> bool {
        self.postdominated_by[&b].contains(&a)
    }

    pub fn strict_dom(&self, a: usize, b: usize) -> bool {
        a != b && self.dom(a, b)
    }
}
