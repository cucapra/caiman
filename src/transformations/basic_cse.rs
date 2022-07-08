use crate::dataflow::{traversals, Error, Graph, ValueDependent};
use std::collections::HashMap;

/// Applies common subexpression elimination to `graph`. CSE is only applied to "reachable" nodes
/// (those iterated over by [`traversals::DependencyFirst`]) — this is probably what you want.
///
/// Currently, this function *does not* utilize mathematical properties such as transitivity.
pub fn apply(graph: &mut Graph) -> Result<(), Error> {
    // Map from nodes (their actual contents!) to a canonical node index
    // *TODO:* This has absolutely abysmal memory usage. In the worst case scenario, it may
    // store a copy of the entire graph (+ extra memory due to the load factor!)
    // To make this sane, I'll probably want to use HashMap::raw_entry_mut
    let mut canonical = HashMap::new();
    // Map from duplicate node index to canonical node index
    let mut dedup = HashMap::new();

    let mut traversal = traversals::DependencyFirst::new(graph);
    while let Some(index) = traversal.next(graph).map_err(Error::from)? {
        // -------- Step 1: Canonicalize this node's dependencies
        let canonicalize = |i| dedup.get(&i).copied().unwrap_or(i);
        graph.node_mut(index).map_dependencies(canonicalize);

        // -------- Step 2: See if there's already a canonical index for this node
        // If so, ensure dependents on this node are remapped to refer to the canonical index
        // instead; if not, this becomes the canonical index
        if let Some(&other) = canonical.get(graph.node(index)) {
            dedup.insert(index, other);
        } else {
            canonical.insert(graph.node(index).clone(), index);
        }
    }

    // -------- Step 3: Canonicalize the tail edge's dependencies
    // TODO: This should not be necessary after operation-tail unification (coming soon?)
    let canonicalize = |i| dedup.get(&i).copied().unwrap_or(i);
    graph.tail_mut().map_dependencies(canonicalize);

    Ok(())
}

#[cfg(test)]
mod tests {
    fn validate_cse(pre_str: &str, post_str: &str) {
        crate::dataflow::validate(pre_str, |graph| super::apply(graph).unwrap(), post_str)
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
                CallExternalCpu(external_function_id : 0, arguments : [2]),
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
