use crate::arena::Arena;
use crate::ir;

use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeId(usize);
impl NodeId {
    const ENTRY: Self = NodeId(0);
}

#[derive(Clone)]
struct Node {
    // The ir funclet which this analysis node corresponds to.
    fid: ir::FuncletId,
    // The index of this funclet's immediate dominator.
    // For the entry node, this will be its own id (since it doesn't have an idom)
    idom: NodeId,
    // A list of this funclet's immediate predecessors.
    preds: Vec<NodeId>,
}

pub struct AnalysisGraph {
    /// The nodes in the analysis tree.
    nodes: Vec<Node>,
    // A map from funclets to their node index
    lookup: HashMap<ir::FuncletId, NodeId>,
}
impl AnalysisGraph {
    fn recalc_idom(nodes: &[Node], mut a: NodeId, mut b: NodeId) -> NodeId {
        loop {
            match a.cmp(&b) {
                Ordering::Less => b = nodes[b.0].idom,
                Ordering::Greater => a = nodes[a.0].idom,
                Ordering::Equal => return a,
            }
        }
    }
    pub fn new(head: ir::FuncletId, funclets: &Arena<ir::Funclet>) -> Self {
        // This is loosely based on http://www.hipersoft.rice.edu/grads/publications/dom14.pdf
        let mut lookup = HashMap::new();
        let mut nodes: Vec<Node> = Vec::new();

        let mut stack = Vec::new();
        stack.push((head, NodeId::ENTRY));

        while let Some((fid, pred)) = stack.pop() {
            let nid = match lookup.entry(fid) {
                // already visited this funclet... might need to recalc immediate dominator
                Entry::Occupied(entry) => {
                    let nid: NodeId = *entry.get();
                    nodes[nid.0].preds.push(pred);
                    let old_idom = nodes[nid.0].idom;
                    let new_idom = Self::recalc_idom(&nodes, old_idom, pred);
                    if old_idom == new_idom {
                        continue;
                    }
                    nodes[nid.0].idom = new_idom;
                    nid
                }
                // unvisited funclet, set its idom to the previous node & add its children
                Entry::Vacant(spot) => {
                    let nid = NodeId(nodes.len());
                    spot.insert(nid);
                    nodes.push(Node {
                        fid,
                        idom: pred,
                        preds: vec![pred],
                    });
                    nid
                }
            };
            match &funclets[&fid].tail_edge {
                ir::TailEdge::Return { .. } => (),
                ir::TailEdge::Jump(jump) => stack.push((jump.target, nid)),
                ir::TailEdge::Switch { cases, .. } => {
                    cases.iter().for_each(|jump| stack.push((jump.target, nid)))
                }
            }
        }
        // fixup the entry node, which shouldn't have any preds
        nodes[0].preds.clear();
        Self { nodes, lookup }
    }
    pub fn immediate_dominator(&self, fid: ir::FuncletId) -> Option<ir::FuncletId> {
        let nid = self.lookup[&fid];
        if nid == NodeId::ENTRY {
            return None;
        }
        let idom = self.nodes[nid.0].idom;
        Some(self.nodes[idom.0].fid)
    }
    pub fn dominators(&self, fid: ir::FuncletId) -> impl '_ + Iterator<Item = ir::FuncletId> {
        Dominators::new(&self, fid)
    }
}
struct Dominators<'a> {
    graph: &'a AnalysisGraph,
    cur: Option<NodeId>,
}
impl<'a> Dominators<'a> {
    fn new(graph: &'a AnalysisGraph, fid: ir::FuncletId) -> Self {
        let cur = Some(graph.lookup[&fid]);
        Self { graph, cur }
    }
}
impl<'a> Iterator for Dominators<'a> {
    type Item = ir::FuncletId;
    fn next(&mut self) -> Option<Self::Item> {
        let nid = self.cur?;
        let node = &self.graph.nodes[nid.0];
        self.cur = (nid != NodeId::ENTRY).then(|| node.idom);
        Some(node.fid)
    }
}
