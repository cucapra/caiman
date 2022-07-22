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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ComponentId(usize);

struct Component {
    nids: Vec<ir::FuncletId>,
    preds: Vec<ComponentId>,
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
    fn components(&self) -> Vec<Component> {
        // create initial components: one for each node
        let mut comps = Vec::with_capacity(self.nodes.len() - 1);
        for node in self.nodes.iter() {
            comps.push(Component {
                nids: vec![node.fid],
                preds: node.preds.iter().map(|x| ComponentId(x.0 - 1)).collect(),
            })
        }
        loop {
            let old_comps_len = comps.len();
            let mut new_comps: Vec<Component> = Vec::new();
            let mut remap = HashMap::new();
            for (i, comp) in comps.iter_mut().enumerate() {
                // sort & deduplicate predecessors and remove self-loops (T1)
                comp.preds.sort_unstable();
                comp.preds.dedup();
                comp.preds.retain(|&id| id == ComponentId(i));
            }
            for (i, comp) in comps.into_iter().enumerate() {
                // merge
                if comp.preds.len() == 1 {
                    let pred = comp.preds[0];
                    remap.insert(ComponentId(i), pred);
                    new_comps[remap[&pred].0].nids.extend(comp.nids);
                } else {
                    let new_id = new_comps.len();
                    new_comps.push(comp);
                    remap.insert(ComponentId(i), ComponentId(new_id));
                }
            }
            comps = new_comps;
            for comp in comps.iter_mut() {
                comp.preds.iter_mut().for_each(|id| *id = remap[id]);
            }
            if old_comps_len == comps.len() {
                break;
            }
        }
        comps
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

#[cfg(test)]
mod tests {
    use super::*;
    struct NodeDesc {
        tail: ir::TailEdge,
        doms: &'static [ir::FuncletId],
    }
    fn make_arena(desc: &[NodeDesc]) -> Arena<ir::Funclet> {
        // NOTE: correctness here depends on empty arenas adding nodes with sequential ids
        let mut arena = Arena::new();
        for nd in desc.iter() {
            arena.create(ir::Funclet {
                tail_edge: nd.tail.clone(),
                // everything below this point is probably incorrect
                kind: ir::FuncletKind::MixedExplicit,
                input_types: Box::new([]),
                output_types: Box::new([]),
                nodes: Box::new([]),
                input_resource_states: Default::default(),
                output_resource_states: Default::default(),
                local_meta_variables: Default::default(),
            });
        }
        arena
    }
    fn validate_dominators(desc: &[NodeDesc], analysis: &AnalysisGraph) {
        for (index, nd) in desc.iter().enumerate() {
            let mut expected: Vec<_> = nd.doms.iter().map(|&id| id).collect();
            expected.sort_unstable();
            let mut actual: Vec<_> = analysis.dominators(index).collect();
            actual.sort_unstable();
            assert_eq!(expected, actual);
            // the last element of the dominator slice is always the node itself,
            // the *second* to last element is treated as the immediate dominator.
            let idom = if nd.doms.len() >= 2 {
                Some(nd.doms[nd.doms.len() - 2])
            } else {
                None
            };
            assert_eq!(idom, analysis.immediate_dominator(index));
        }
    }
    fn test_reducible(desc: &[NodeDesc]) {
        let funclets = make_arena(desc);
        let analysis = AnalysisGraph::new(0, &funclets);
        validate_dominators(desc, &analysis);
        //let comps = analysis.components();
        //assert!(comps.len() == 1);
    }
    macro_rules! node_tail {
        ((ret)) => {
            ir::TailEdge::Return { return_values: Box::new([])}
        };
        ((jmp $i:expr)) => {
            ir::TailEdge::Jump(ir::Jump {target: $i, args: Box::new([])})
        };
        ((sel $($i:expr),+)) => {
            ir::TailEdge::Switch {
                key: 0,
                cases: Box::new([
                    $(ir::Jump {target: $i, args: Box::new([])}),+
                ])
            }
        };
    }
    macro_rules! node {
        ($tail:tt [$($dom:expr),+]) => {
            NodeDesc {
                tail: node_tail!($tail),
                doms: &[ $($dom),+ ]
            }
        }
    }
    macro_rules! nodes {
        ($($tail:tt $doms:tt),*) => {
            &[ $(node!($tail $doms)), *]
        }
    }

    #[test]
    fn ret() {
        test_reducible(nodes![(ret)[0]])
    }
    #[test]
    fn jmp() {
        test_reducible(nodes![
            (jmp 1)     [0],                // = 0
            (jmp 2)     [0, 1],             // = 1
            (ret)       [0, 1, 2]           // = 2
        ])
    }
    #[test]
    fn sel() {
        test_reducible(nodes![
            (sel 1, 2, 3)   [0],            // = 0
            (ret)           [0, 1],         // = 1
            (sel 4, 5)      [0, 2],         // = 2
            (ret)           [0, 3],         // = 3
            (ret)           [0, 2, 4],      // = 4
            (ret)           [0, 2, 5]       // = 5
        ])
    }
    #[test]
    fn entry_loop_inf() {
        test_reducible(nodes![(jmp 0) [0]])
    }
    #[test]
    fn entry_loop() {
        test_reducible(nodes![(sel 0, 1) [0], (ret) [0, 1]])
    }
    #[test]
    fn nested_loop_1() {
        test_reducible(nodes![
            (jmp 1)     [0],            // = 0
            (sel 1, 2)  [0, 1],         // = 1
            (sel 0, 3)  [0, 1, 2],      // = 2
            (ret)       [0, 1, 2, 3]    // = 3
        ])
    }
    #[test]
    fn nested_loop_2() {
        test_reducible(nodes![
            (sel 0, 1)          [0],            // = 0
            (sel 0, 1, 2)       [0, 1],         // = 1
            (sel 0, 1, 2, 3)    [0, 1, 2],      // = 2
            (ret)               [0, 1, 2, 3]    // = 3
        ])
    }
    #[test]
    fn diamond() {
        test_reducible(nodes![
            (sel 1, 2)  [0],    // = 0
            (jmp 3)     [0, 1], // = 1
            (jmp 3)     [0, 2], // = 2
            (ret)       [0, 3]  // = 3
        ])
    }
    #[test]
    fn ece5775_lec6_pg22() {
        test_reducible(nodes![
            (jmp 1)     [0],                    // just to make nodes align
            (jmp 2)     [0, 1],                 // = 1
            (sel 3, 10) [0, 1, 2],              // = 2
            (jmp 4)     [0, 1, 2, 3],           // = 3
            (sel 5, 9)  [0, 1, 2, 3, 4],        // = 4
            (sel 6, 7)  [0, 1, 2, 3, 4, 5],     // = 5
            (jmp 8)     [0, 1, 2, 3, 4, 5, 6],  // = 6
            (jmp 8)     [0, 1, 2, 3, 4, 5, 7],  // = 7
            (jmp 4)     [0, 1, 2, 3, 4, 5, 8],  // = 8
            (jmp 2)     [0, 1, 2, 3, 4, 9],     // = 9
            (ret)       [0, 1, 2, 10]           // = 10
        ])
    }
    #[test]
    fn switch_fallthrough() {
        test_reducible(nodes![
            (sel 1, 2, 3, 4)    [0],    // = 0
            (jmp 2)             [0, 1], // = 1
            (jmp 3)             [0, 2], // = 2
            (jmp 4)             [0, 3], // = 3
            (ret)               [0, 4]  // = 4
        ])
    }
    #[test]
    fn switch_fallthrough_rev() {
        test_reducible(nodes![
            (sel 1, 2, 3, 4)    [0],    // = 0
            (ret)               [0, 1], // = 1
            (jmp 1)             [0, 2], // = 2
            (jmp 2)             [0, 3], // = 3
            (jmp 3)             [0, 4]  // = 4
        ])
    }

    /*
    mod irreducible {
        use super::*;
        #[test]
        fn dom14_fig2() {
            assert_cfg(&[
                (sel![1, 2], &[0]), // = 0 (dom14:2:5)
                (jmp!(3), &[0, 1]), // = 1 (dom14:2:4)
                (jmp!(4), &[0, 2]), // = 2 (dom14:2:3)
                (jmp!(4), &[0, 3]), // = 3 (dom14:2:1)
                (jmp!(3), &[0, 4]), // = 1 (dom14:2:2)
            ])
        }
        #[test]
        fn dom14_fig4() {
            assert_cfg(&[
                (sel![1, 2], &[0]),    // = 0 (dom14:4:6)
                (jmp!(3), &[0, 1]),    // = 1 (dom14:4:5)
                (sel![4, 5], &[0, 2]), // = 2 (dom14:4:4)
                (jmp!(4), &[0, 3]),    // = 3 (dom14:4:1)
                (sel![3, 5], &[0, 4]), // = 4 (dom14:4:2)
                (jmp!(4), &[0, 5]),    // = 5 (dom14:4:3)
            ])
        }
        #[test]
        fn triangle() {
            assert_cfg(&[
                (sel![1, 2], &[0]), // = 0
                (jmp!(2), &[0, 1]), // = 1
                (jmp!(1), &[0, 2]), // = 2
            ])
        }
        #[test]
        fn pogo() {
            assert_cfg(&[
                (jmp!(1), &[0]),       // = 0
                (jmp!(2), &[0, 1]),    // = 1
                (jmp!(1), &[0, 1, 2]), // = 2
            ])
        }
        #[test]
        fn circle_like() {
            assert_cfg(&[
                (sel![1, 5], &[0]),       // = 0
                (jmp!(2), &[0, 1]),       // = 1
                (sel![1, 3], &[0, 2]),    // = 2
                (sel![2, 4, 6], &[0, 3]), // = 3
                (sel![3, 5], &[0, 4]),    // = 4
                (jmp!(4), &[0, 5]),       // = 5
                (ret!(), &[0, 3, 6]),     // = 6
            ])
        }
    }
    */
}