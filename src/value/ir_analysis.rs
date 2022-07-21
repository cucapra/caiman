use crate::arena::Arena;
use crate::ir;

use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(usize);
impl NodeId {
    const PREENTRY: Self = Self(0);
}

#[derive(Clone)]
struct Node {
    // The ir funclet which this analysis node corresponds to.
    fid: ir::FuncletId,
    // The index of this funclet's immediate dominator. If `0`, this node is an entry node.
    idom: NodeId,
    // A list of this funclet's immediate predecessors.
    preds: Vec<NodeId>,
}
pub struct AnalysisGraph {
    /// The nodes in the analysis tree.
    /// The node at index 0 is a fake "pre-entry" node which strictly dominates all other nodes.
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

        // push pre-entry node
        nodes.push(Node {
            fid: usize::MAX,
            idom: NodeId::PREENTRY,
            preds: Vec::new(),
        });

        let mut stack = Vec::new();
        stack.push((head, NodeId::PREENTRY));

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
        Self { nodes, lookup }
    }
    pub fn immediate_dominator(&self, fid: ir::FuncletId) -> Option<ir::FuncletId> {
        let node = &self.nodes[self.lookup[&fid].0];
        if node.idom != NodeId::PREENTRY {
            return Some(self.nodes[node.idom.0].fid);
        }
        None
    }
    pub fn dominators(&self, fid: ir::FuncletId) -> impl '_ + Iterator<Item = ir::FuncletId> {
        Dominators::new(&self, fid)
    }
}
struct Dominators<'a> {
    graph: &'a AnalysisGraph,
    cur: NodeId,
}
impl<'a> Dominators<'a> {
    fn new(graph: &'a AnalysisGraph, fid: ir::FuncletId) -> Self {
        let cur = graph.lookup[&fid];
        Self { graph, cur }
    }
}
impl<'a> Iterator for Dominators<'a> {
    type Item = ir::FuncletId;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == NodeId::PREENTRY {
            return None;
        }
        let node = &self.graph.nodes[self.cur.0];
        self.cur = node.idom;
        Some(node.fid)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cfg(input: &[(ir::TailEdge, &[usize])]) {
        // NOTE: correctness here depends on empty arenas adding nodes with sequential ids
        let mut funclets = Arena::new();
        for (tail_edge, _) in input.iter() {
            funclets.create(ir::Funclet {
                tail_edge: tail_edge.clone(),
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
        let graph = AnalysisGraph::new(0, &funclets);
        for (index, (_, dominators)) in input.iter().enumerate() {
            let mut expected: Vec<_> = dominators.iter().map(|&id| id).collect();
            expected.sort_unstable();
            let mut actual: Vec<_> = graph.dominators(index).collect();
            actual.sort_unstable();
            assert_eq!(expected, actual);
            // the last element of the dominator slice is always the node itself,
            // the *second* to last element is treated as the immediate dominator.
            let idom = if dominators.len() >= 2 {
                Some(dominators[dominators.len() - 2])
            } else {
                None
            };
            assert_eq!(idom, graph.immediate_dominator(index));
        }
    }
    macro_rules! ret {
        () => {
            ir::TailEdge::Return {
                return_values: Box::new([]),
            }
        };
    }
    macro_rules! jmp {
        ($i:expr) => {
            ir::TailEdge::Jump(ir::Jump {
                target: $i,
                args: Box::new([]),
            })
        };
    }
    macro_rules! sel {
        ($($i:expr),* $(,)?) => {
            ir::TailEdge::Switch {
                key: 0,
                cases: Box::new([
                    $(ir::Jump {target: $i, args: Box::new([])}),*
                ])
            }
        }
    }
    #[test]
    fn ret() {
        assert_cfg(&[(ret!(), &[0])])
    }
    #[test]
    fn jmp() {
        assert_cfg(&[
            (jmp!(1), &[0]),      // = 0
            (jmp!(2), &[0, 1]),   // = 1
            (ret!(), &[0, 1, 2]), // = 2
        ])
    }
    #[test]
    fn sel() {
        assert_cfg(&[
            (sel![1, 2, 3], &[0]), // = 0
            (ret!(), &[0, 1]),     // = 1
            (sel![4, 5], &[0, 2]), // = 2
            (ret!(), &[0, 3]),     // = 3
            (ret!(), &[0, 2, 4]),  // = 4
            (ret!(), &[0, 2, 5]),  // = 5
        ])
    }
    #[test]
    fn entry_loop_inf() {
        assert_cfg(&[(jmp!(0), &[0])])
    }
    #[test]
    fn entry_loop() {
        assert_cfg(&[(sel![0, 1], &[0]), (ret!(), &[0, 1])])
    }
    #[test]
    fn nested_loop_1() {
        assert_cfg(&[
            (jmp!(1), &[0]),          // = 0
            (sel![1, 2], &[0, 1]),    // = 1
            (sel![0, 3], &[0, 1, 2]), // = 2
            (ret!(), &[0, 1, 2, 3]),  // = 3
        ])
    }
    #[test]
    fn nested_loop_2() {
        assert_cfg(&[
            (sel![0, 1], &[0]),             // = 0
            (sel![0, 1, 2], &[0, 1]),       // = 1
            (sel![0, 1, 2, 3], &[0, 1, 2]), // = 2
            (ret!(), &[0, 1, 2, 3]),        // = 3
        ])
    }
    #[test]
    fn diamond() {
        assert_cfg(&[
            (sel![1, 2], &[0]), // = 0
            (jmp!(3), &[0, 1]), // = 1
            (jmp!(3), &[0, 2]), // = 2
            (ret!(), &[0, 3]),  // = 3
        ])
    }
    #[test]
    fn ece5775_lec6_pg22() {
        assert_cfg(&[
            (jmp!(1), &[0]),                   // just to make nodes align
            (jmp!(2), &[0, 1]),                // = 1
            (sel![3, 10], &[0, 1, 2]),         // = 2
            (jmp!(4), &[0, 1, 2, 3]),          // = 3
            (sel![5, 9], &[0, 1, 2, 3, 4]),    // = 4
            (sel![6, 7], &[0, 1, 2, 3, 4, 5]), // = 5
            (jmp!(8), &[0, 1, 2, 3, 4, 5, 6]), // = 6
            (jmp!(8), &[0, 1, 2, 3, 4, 5, 7]), // = 7
            (jmp!(4), &[0, 1, 2, 3, 4, 5, 8]), // = 8
            (jmp!(2), &[0, 1, 2, 3, 4, 9]),    // = 9
            (ret!(), &[0, 1, 2, 10]),          // = 10
        ])
    }
    #[test]
    fn switch_fallthrough() {
        assert_cfg(&[
            (sel![1, 2, 3, 4], &[0]), // = 0
            (jmp!(2), &[0, 1]),       // = 1
            (jmp!(3), &[0, 2]),       // = 2
            (jmp!(4), &[0, 3]),       // = 3
            (ret!(), &[0, 4]),        // = 4
        ])
    }
    #[test]
    fn switch_fallthrough_rev() {
        assert_cfg(&[
            (sel![1, 2, 3, 4], &[0]), // = 0
            (ret!(), &[0, 1]),        // = 1
            (jmp!(1), &[0, 2]),       // = 2
            (jmp!(2), &[0, 3]),       // = 3
            (jmp!(3), &[0, 4]),       // = 4
        ])
    }
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
}
