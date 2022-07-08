use crate::dataflow::{Graph, Node, NodeIndex, ValueDependent};
use crate::transformations::{Error, SubgraphTransform};
use std::collections::HashMap;
pub struct BasicCse {
    remaps: HashMap<NodeIndex, NodeIndex>,
    // TODO: This has pretty poor memory usage... in the worst case scenario,
    // we're essentially duplicating the entire graph, and then some because alignment.
    exprs: HashMap<Node, NodeIndex>,
}
impl BasicCse {
    fn new() -> Self {
        Self {
            remaps: HashMap::new(),
            exprs: HashMap::new(),
        }
    }
}
impl SubgraphTransform for BasicCse {
    fn attempt(&mut self, graph: &mut Graph, index: NodeIndex) -> bool {
        let remap = |id| self.remaps.get(&id).copied().unwrap_or(id);
        graph.node_mut(index).map_dependencies(remap);
        // This is a bit ugly, hopefully it can be eliminated in the future
        graph.tail_mut().map_dependencies(remap);
        if let Some(&other) = self.exprs.get(graph.node(index)) {
            self.remaps.insert(index, other);
            true
        } else {
            self.exprs.insert(graph.node(index).clone(), index);
            false
        }
    }
    fn reset(&mut self) {
        self.remaps.clear();
        self.exprs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::BasicCse;
    use crate::dataflow::validate;
    use crate::transformations::{attempt_subgraph_transforms, SubgraphTransform};
    fn validate_cse(pre_str: &str, post_str: &str) {
        let cse = Box::new(BasicCse::new()) as Box<dyn SubgraphTransform>;
        validate(
            pre_str,
            |graph| {
                attempt_subgraph_transforms(graph, &mut [cse]).unwrap();
            },
            post_str,
        )
    }
    #[test]
    fn empty() {
        let empty = "(
            kind : MixedImplicit,
            input_types : [],
            output_types : [],
            nodes : [],
            tail_edge : Return(return_values : []) 
        )";
        validate_cse(empty, empty);
    }
    #[test]
    fn unchanged() {
        let unchanged = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                ExtractResult(node_id : 1, index : 0),
            ],
            tail_edge : Return(return_values : [2]) 
        )";
        validate_cse(unchanged, unchanged);
    }
    #[test]
    fn unchanged_multi() {
        let unchanged_multi = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                ExtractResult(node_id : 1, index : 0),
            ],
            tail_edge : Return(return_values : [0, 2]) 
        )";
        validate_cse(unchanged_multi, unchanged_multi);
    }
    #[test]
    fn basic_node() {
        let pre = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0, 1])
            ],
            tail_edge : Return(return_values : [2]) 
        )";
        let post = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0, 0])
            ],
            tail_edge : Return(return_values : [1]) 
        )";
        validate_cse(pre, post);
    }
    #[test]
    fn basic_tail() {
        let pre = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                Phi(index : 0)
            ],
            tail_edge : Return(return_values : [0, 1]) 
        )";
        let post = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
            ],
            tail_edge : Return(return_values : [0, 0]) 
        )";
        validate_cse(pre, post);
    }
    #[test]
    fn different_node_indexes() {
        let pre = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                CallExternalCpu(external_function_id : 0, arguments : [1])
            ],
            tail_edge : Return(return_values : [2, 3]) 
        )";
        let post = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
            ],
            tail_edge : Return(return_values : [1, 1])
        )";
        validate_cse(pre, post);
    }
    #[test]
    fn interleave() {
        let pre = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [2])
            ],
            tail_edge : Return(return_values : [1, 3]) 
        )";
        let post = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
            ],
            tail_edge : Return(return_values : [1, 1])
        )";
        validate_cse(pre, post);
    }
    #[test]
    fn complex() {
        let pre = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [1]),
                ExtractResult(node_id : 3, index : 0),
                ExtractResult(node_id : 1, index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [1, 5]),
                CallExternalCpu(external_function_id : 0, arguments : [1, 4]),
                CallExternalCpu(external_function_id : 0, arguments : [6, 7])
            ],
            tail_edge : Return(return_values : [3, 8]) 
        )";
        let post = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                ExtractResult(node_id : 1, index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [1, 2]),
                CallExternalCpu(external_function_id : 0, arguments : [3, 3])
            ],
            tail_edge : Return(return_values : [1, 4]) 
        )";
        validate_cse(pre, post);
    }
}
