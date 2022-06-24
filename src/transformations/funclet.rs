///! Transformations which operate on individual funclets in isolation.
use crate::ir;
use crate::transformations::Result;
use std::collections::HashMap;

/// Prunes all unused nodes from `funclet`.
pub fn prune_unused_nodes(funclet: &mut ir::Funclet) -> Result<()> {
    /// Mark the node given by `id`, and all its value dependencies, as used by appending
    /// their node ids to `used_nodes`.
    fn mark_used(funclet: &ir::Funclet, id: ir::NodeId, used_ids: &mut Vec<ir::NodeId>) {
        used_ids.push(id);
        (&funclet.nodes[id])
            .for_each_referenced_node(|referenced| mark_used(funclet, referenced, used_ids))
    }
    // IDs of all used nodes.
    let mut used_ids: Vec<ir::NodeId> = Vec::with_capacity(funclet.nodes.len());
    funclet
        .tail_edge
        .for_each_referenced_node(|referenced| mark_used(funclet, referenced, &mut used_ids));
    used_ids.sort();
    used_ids.dedup();

    // Map from old node ID to new node ID. Has no entry for unused nodes.
    let inverse: HashMap<_, _> = used_ids.iter().enumerate().map(|(x, &y)| (y, x)).collect();

    let new_nodes: Vec<ir::Node> = used_ids
        .into_iter()
        .map(|id| funclet.nodes[id].map_referenced_nodes(|referenced| inverse[&referenced]))
        .collect();

    funclet.nodes = new_nodes.into_boxed_slice();
    funclet.tail_edge = funclet
        .tail_edge
        .map_referenced_nodes(|referenced| inverse[&referenced]);

    Ok(())
}

#[cfg(test)]
mod prune_unused_nodes {
    use super::super::*;
    fn validate_transform(pre_str: &str, post_str: &str) {
        let mut pre: ir::Funclet = ron::from_str(pre_str).unwrap();
        funclet::prune_unused_nodes(&mut pre).unwrap();
        let post: ir::Funclet = ron::from_str(post_str).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn empty() {
        let empty_str = "(
            kind : MixedImplicit,
            input_types : [],
            output_types : [],
            nodes : [],
            tail_edge : Return(return_values : []) 
        )";
        validate_transform(empty_str, empty_str);
    }
    #[test]
    fn all_used() {
        let all_used_str = "(
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
        validate_transform(all_used_str, all_used_str);
    }
    #[test]
    fn none_used() {
        let pre_str = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : [0]),
                ExtractResult(node_id : 1, index : 0),
            ],
            tail_edge : Return(return_values : []) 
        )";
        let post_str = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [],
            nodes : [],
            tail_edge : Return(return_values : []) 
        )";
        validate_transform(pre_str, post_str);
    }
    #[test]
    fn some_used() {
        let pre_str = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0],
            nodes : [
                Phi(index : 0),
                CallExternalCpu(external_function_id : 0, arguments : []),
                ExtractResult(node_id : 1, index : 0),
                CallExternalCpu(external_function_id : 1, arguments : [2]),
                ExtractResult(node_id : 3, index : 0),
                CallExternalCpu(external_function_id : 2, arguments: [0]),
                ExtractResult(node_id : 5, index : 0)
            ],
            tail_edge : Return(return_values : [4]) 
        )";
        let post_str = "(
            kind : MixedImplicit,
            input_types : [0],
            output_types : [0],
            nodes : [
                CallExternalCpu(external_function_id : 0, arguments : []),
                ExtractResult(node_id : 0, index : 0),
                CallExternalCpu(external_function_id : 1, arguments : [1]),
                ExtractResult(node_id : 2, index : 0),
            ],
            tail_edge : Return(return_values : [3]) 
        )";
        validate_transform(pre_str, post_str);
    }
    #[test]
    fn complex_use() {
        let pre_str = "(
            kind : MixedImplicit,
            input_types : [0, 0, 0, 0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                Phi(index : 1),
                Phi(index : 2),
                Phi(index : 3),

                CallExternalCpu(external_function_id : 0, arguments : [0, 2]),
                ExtractResult(node_id : 4, index : 0),

                CallExternalCpu(external_function_id : 0, arguments : [1, 3]),
                ExtractResult(node_id : 6, index : 0),

                ConstantInteger(value : 42, type_id : 0),
                CallExternalCpu(external_function_id : 0, arguments : [3, 8]),
                ExtractResult(node_id : 9, index : 0),

                CallExternalCpu(external_function_id : 0, arguments : [5, 8]),
                ExtractResult(node_id : 11, index : 0)
            ],
            tail_edge : Return(return_values : [1, 12]) 
        )";
        let post_str = "(
            kind : MixedImplicit,
            input_types : [0, 0, 0, 0],
            output_types : [0, 0],
            nodes : [
                Phi(index : 0),
                Phi(index : 1),
                Phi(index : 2),

                CallExternalCpu(external_function_id : 0, arguments : [0, 2]),
                ExtractResult(node_id : 3, index : 0),

                ConstantInteger(value : 42, type_id : 0),

                CallExternalCpu(external_function_id : 0, arguments : [4, 5]),
                ExtractResult(node_id : 6, index : 0)
            ],
            tail_edge : Return(return_values : [1, 7]) 
        )";
        validate_transform(pre_str, post_str);
    }
}
