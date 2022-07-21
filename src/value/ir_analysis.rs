use crate::arena::Arena;
use crate::ir;

use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
};

#[derive(Clone)]
struct AnalysisNode {
    funclet_id: ir::FuncletId,
    // The index of this funclet's immediate dominator. If `0`, this node is an entry node.
    idom_index: usize,
    // A list of this funclet's immediate predecessors.
    predecessors: Vec<usize>,
}
pub struct AnalysisGraph {
    /// The nodes in the analysis tree.
    /// The node at index 0 is a fake "pre-entry" node which strictly dominates all other nodes.
    nodes: Vec<AnalysisNode>,
    // A map from funclets to their node index
    lookup: HashMap<ir::FuncletId, usize>,
}
impl AnalysisGraph {
    fn recalc_idom(nodes: &[AnalysisNode], mut idom_a: usize, mut idom_b: usize) -> usize {
        loop {
            match idom_a.cmp(&idom_b) {
                Ordering::Less => idom_b = nodes[idom_b].idom_index,
                Ordering::Greater => idom_a = nodes[idom_a].idom_index,
                Ordering::Equal => return idom_a,
            }
        }
    }
    pub fn new(pipeline: &ir::Pipeline, funclets: &Arena<ir::Funclet>) -> Self {
        // This is loosely based on http://www.hipersoft.rice.edu/grads/publications/dom14.pdf
        let mut lookup = HashMap::new();
        let mut nodes: Vec<AnalysisNode> = Vec::new();
        // push pre-entry node
        nodes.push(AnalysisNode {
            funclet_id: usize::MAX,
            idom_index: 0,
            predecessors: Vec::new(),
        });

        let mut stack = Vec::new();
        stack.push((pipeline.entry_funclet, 0));

        while let Some((funclet_id, parent_index)) = stack.pop() {
            let funclet_node_index = match lookup.entry(funclet_id) {
                // already visited this funclet... might need to recalc immediate dominator
                Entry::Occupied(entry) => {
                    let funclet_node_index: usize = *entry.get();
                    nodes[funclet_node_index].predecessors.push(parent_index);
                    let old_idom_index = nodes[funclet_node_index].idom_index;
                    let new_idom_index = Self::recalc_idom(&nodes, old_idom_index, parent_index);
                    // TODO: This might be wrong
                    if old_idom_index == new_idom_index {
                        continue;
                    }
                    nodes[funclet_node_index].idom_index = new_idom_index;
                    funclet_node_index
                }
                // unvisited funclet, set its idom to the previous node & add its children
                Entry::Vacant(spot) => {
                    let funclet_node_index = nodes.len();
                    spot.insert(funclet_node_index);
                    nodes.push(AnalysisNode {
                        funclet_id,
                        idom_index: parent_index,
                        predecessors: vec![parent_index],
                    });
                    funclet_node_index
                }
            };
            match &funclets[&funclet_id].tail_edge {
                ir::TailEdge::Return { .. } => (),
                ir::TailEdge::Jump(jump) => stack.push((jump.target, funclet_node_index)),
                ir::TailEdge::Switch { cases, .. } => cases
                    .iter()
                    .for_each(|jump| stack.push((jump.target, funclet_node_index))),
            }
        }
        Self { nodes, lookup }
    }
    pub fn immediate_dominator(&self, funclet: ir::FuncletId) -> Option<ir::FuncletId> {
        let funclet_node_index = self.lookup[&funclet];
        let funclet_node = &self.nodes[funclet_node_index];
        if funclet_node.idom_index != 0 {
            let idom_node = &self.nodes[funclet_node.idom_index];
            return Some(idom_node.funclet_id);
        }
        None
    }
    pub fn dominators(&self, funclet: ir::FuncletId) -> impl '_ + Iterator<Item = ir::FuncletId> {
        Dominators::new(&self, funclet)
    }
}
struct Dominators<'a> {
    graph: &'a AnalysisGraph,
    index: usize,
}
impl<'a> Dominators<'a> {
    fn new(graph: &'a AnalysisGraph, funclet: ir::FuncletId) -> Self {
        let index = graph.lookup[&funclet];
        Self { graph, index }
    }
}
impl<'a> Iterator for Dominators<'a> {
    type Item = ir::FuncletId;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None; // at pre-entry funclet, which isn't real
        }
        let node = &self.graph.nodes[self.index];
        self.index = node.idom_index;
        Some(node.funclet_id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cfg(input: &[(ir::TailEdge, &[usize])]) {
        let pipeline = ir::Pipeline {
            name: "test pipeline".to_string(),
            entry_funclet: 0,
            kind: ir::PipelineKind::Function,
        };
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
        let graph = AnalysisGraph::new(&pipeline, &funclets);
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
