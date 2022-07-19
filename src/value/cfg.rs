use crate::arena::Arena;
use crate::ir;

use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
};

pub struct DominatorGraph {
    /// The nodes in the dominator tree. Each node is comprised of it's corresponding funclet ID
    /// and the index of its immediate dominator (or `0` if its the entry node).
    /// The node at index 0 is a fake "pre-entry" node which strictly dominates all other nodes.
    nodes: Vec<(ir::FuncletId, usize)>,
    // A map from funclets to their node index
    lookup: HashMap<ir::FuncletId, usize>,
}
impl DominatorGraph {
    fn first_shared_ancestor(
        nodes: &[(ir::FuncletId, usize)],
        mut a: usize,
        mut b: usize,
    ) -> usize {
        loop {
            match a.cmp(&b) {
                Ordering::Less => b = nodes[b].1,
                Ordering::Greater => a = nodes[a].1,
                Ordering::Equal => return a,
            }
        }
    }
    pub fn new(pipeline: &ir::Pipeline, funclets: &Arena<ir::Funclet>) -> Self {
        // This is loosely based on http://www.hipersoft.rice.edu/grads/publications/dom14.pdf
        let mut lookup = HashMap::new();
        let mut nodes = Vec::new();
        nodes.push((0, 0)); // pre-entry node
        let mut stack = Vec::new();
        stack.push((pipeline.entry_funclet, 0));
        while let Some((id, prev)) = stack.pop() {
            match lookup.entry(id) {
                // already visited this funclet... might need to recalc immediate dominator
                Entry::Occupied(mut val) => {
                    val.insert(Self::first_shared_ancestor(&nodes, *val.get(), prev));
                }
                // unvisited funclet, set its idom to the previous node & add its children
                Entry::Vacant(spot) => {
                    let node_id = nodes.len();
                    spot.insert(node_id);
                    nodes.push((id, prev));
                    match &funclets[&id].tail_edge {
                        ir::TailEdge::Return { .. } => (),
                        ir::TailEdge::Jump(jump) => stack.push((jump.target, node_id)),
                        ir::TailEdge::Switch { cases, .. } => cases
                            .iter()
                            .for_each(|jump| stack.push((jump.target, node_id))),
                    }
                }
            }
        }
        Self { nodes, lookup }
    }
    pub fn immediate_dominator(&self, funclet: ir::FuncletId) -> Option<ir::FuncletId> {
        let node_index = self.lookup[&funclet];
        let node = self.nodes[node_index];
        if node.1 != 0 {
            let idom_node = self.nodes[node.1];
            return Some(idom_node.0);
        }
        // made it here: at entry funclet, which has no immediate dominator
        assert!(node_index == 1);
        None
    }
    pub fn dominators(&self, funclet: ir::FuncletId) -> impl '_ + Iterator<Item = ir::FuncletId> {
        Dominators::new(&self, funclet)
    }
}
struct Dominators<'a> {
    graph: &'a DominatorGraph,
    index: usize,
}
impl<'a> Dominators<'a> {
    fn new(graph: &'a DominatorGraph, funclet: ir::FuncletId) -> Self {
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
        let node = self.graph.nodes[self.index];
        self.index = node.1;
        Some(node.0)
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
        let graph = DominatorGraph::new(&pipeline, &funclets);
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
    fn irreducible_1() {
        assert_cfg(&[
            (sel![1, 2], &[0]), // = 0 (dom14:2:5)
            (jmp!(3), &[0, 1]), // = 1 (dom14:2:4)
            (jmp!(4), &[0, 2]), // = 2 (dom14:2:3)
            (jmp!(4), &[0, 3]), // = 3 (dom14:2:1)
            (jmp!(3), &[0, 4]), // = 1 (dom14:2:2)
        ])
    }
    #[test]
    fn irreducible_2() {
        assert_cfg(&[
            (sel![1, 2], &[0]),    // = 0 (dom14:4:6)
            (jmp!(3), &[0, 1]),    // = 1 (dom14:4:5)
            (sel![4, 5], &[0, 2]), // = 2 (dom14:4:4)
            (jmp!(4), &[0, 3]),    // = 3 (dom14:4:1)
            (sel![3, 5], &[0, 4]), // = 4 (dom14:4:2)
            (jmp!(4), &[0, 5]),    // = 5 (dom14:4:3)
        ])
    }
}
