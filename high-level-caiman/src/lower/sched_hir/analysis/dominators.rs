//! Computes dominators of a CFG.
//! Mostly taken from
//! [here](https://github.com/stephenverderame/cs6120-bril/blob/main/cfg/src/analysis/dominators.rs)
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::lower::sched_hir::cfg::Cfg;

/// A node in the dominator tree
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

/// A dominator tree
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
}

/// Computes the dominators of each block in the CFG
/// # Arguments
/// * `cfg` - The CFG
/// # Returns
/// * A map from each block to nodes that dominate it
/// # Panics
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn compute_dominators(cfg: &Cfg) -> DomTree {
    let preds = &cfg.transpose_graph;
    let mut doms: HashMap<_, HashSet<_>> = HashMap::new();
    let all_blocks = cfg.blocks.keys().copied().collect::<HashSet<_>>();
    for block in cfg.blocks.keys() {
        doms.insert(*block, all_blocks.clone());
    }
    let mut changed = true;
    let default_preds = BTreeSet::new();
    while changed {
        changed = false;
        for block in cfg.blocks.keys() {
            let mut pred_iter = preds.get(block).unwrap_or(&default_preds).iter();
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
    DomTree::new(&doms)
}
