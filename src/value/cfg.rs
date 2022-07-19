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
        let mut nodes = Vec::new();
        nodes.push((0, 0)); // pre-entry node
        let mut lookup = HashMap::new();

        let mut stack = Vec::new();
        stack.push((pipeline.entry_funclet, 0));

        while let Some((id, parent)) = stack.pop() {
            match lookup.entry(id) {
                Entry::Occupied(mut val) => {
                    // we already visited this funclet... we need to fix up its dominators
                    val.insert(Self::first_shared_ancestor(&nodes, *val.get(), parent));
                }
                Entry::Vacant(spot) => {
                    // we haven't visited this funclet, add it & set its immediate dominator
                    // to whatever node we arrived from...
                    let node_id = nodes.len();
                    spot.insert(node_id);
                    nodes.push((id, parent));
                    // ... and add all its "children" to the stack for later traversal
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
    pub fn new(graph: &'a DominatorGraph, funclet: ir::FuncletId) -> Self {
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

    fn assert_cfg(input: &[(ir::TailEdge, &[usize], Option<usize>)]) {
        let pipeline = ir::Pipeline {
            name: "test pipeline".to_string(),
            entry_funclet: 0,
            kind: ir::PipelineKind::Function,
        };
        let mut funclets = Arena::new();
        for (index, item) in input.iter().enumerate() {
            let id = funclets.create(ir::Funclet {
                tail_edge: item.0.clone(),
                // everything below this point is an arbitrarily chosen,
                // probably-incorrect default
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
        for (index, item) in input.iter().enumerate() {
            let mut expected: Vec<_> = item.1.iter().map(|&id| id).collect();
            expected.sort_unstable();
            let mut actual: Vec<_> = graph.dominators(index).collect();
            actual.sort_unstable();
            assert_eq!(expected, actual);
            assert_eq!(item.2, graph.immediate_dominator(index));
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
    fn test_ret() {
        assert_cfg(&[(ret!(), &[0], None)])
    }
    #[test]
    fn test_jmp() {
        assert_cfg(&[
            (jmp!(1), &[0], None),
            (jmp!(2), &[0, 1], Some(0)),
            (ret!(), &[0, 1, 2], Some(1)),
        ])
    }
    #[test]
    fn test_sel() {
        assert_cfg(&[
            (sel![1, 2, 3], &[0], None),    // = 0
            (ret!(), &[0, 1], Some(0)),     // = 1
            (sel![4, 5], &[0, 2], Some(0)), // = 2
            (ret!(), &[0, 3], Some(0)),     // = 3
            (ret!(), &[0, 2, 4], Some(2)),  // = 4
            (ret!(), &[0, 2, 5], Some(2)),  // = 5
        ])
    }
}
