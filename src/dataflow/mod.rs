#![warn(warnings)]
use crate::ir;
use std::collections::HashMap;
use thiserror::Error;

/// Represents a type which can depend on nodes in a value graph.
pub trait ValueDependent: PartialEq {
    /// Invoke `closure` on each dependency of the given operation.
    fn for_each_dependency(&self, closure: impl FnMut(&NodeIndex))
    where
        Self: Sized;

    /// Apply `closure` to each dependency of the given operation.
    fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex)
    where
        Self: Sized;

    /// Returns whether `self` and `other` are deep-equal.
    ///
    /// [`ValueDependent`]s are deep-equal if:
    /// - They have the same concrete type
    /// - Their *dependencies* are deep-equal
    /// - Their non-dependency fields are equal
    fn eq_deep(&self, self_graph: &Graph, other: &Self, other_graph: &Graph) -> bool
    where
        Self: Sized;
}

mod generated;
pub use generated::{Node, Tail};
pub mod traversals;

#[derive(Debug, Error)]
pub enum IrDependent {
    /// Represents the [`ir::Node`](crate::ir) specified by it's node ID in a given funclet.
    Node(ir::NodeId),
    /// Represents the [`ir::TailEdge`](crate::ir) of a given funclet.
    Tail,
}
impl std::fmt::Display for IrDependent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(id) => write!(f, "IR node (id: {id})"),
            Self::Tail => write!(f, "IR tail edge"),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// This error is produced when an [`ir::Node`](crate::ir) or [`ir::TailEdge`](crate::ir)
    /// depends on a node which:
    /// 1. occurs after the dependency, or
    /// 2. doesn't exist at all
    #[error("{required_by} incorrectly depends on IR node #{dependency}")]
    IrDependency {
        /// The ID of the dependency
        dependency: ir::NodeId,
        /// The ID of the node which caused the failure.
        required_by: IrDependent,
    },
    #[error(transparent)]
    DependencyCycle(#[from] traversals::DependencyCycle),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeIndex(usize);
impl NodeIndex {
    fn from_ir_dependency(
        dependency: ir::NodeId,
        required_by: IrDependent,
        sentinel: ir::NodeId,
    ) -> Result<Self, Error> {
        if dependency >= sentinel {
            Err(Error::IrDependency {
                dependency,
                required_by,
            })
        } else {
            Ok(NodeIndex(dependency))
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    tail: Tail,
}
impl Graph {
    pub fn from_ir(ir_nodes: &[ir::Node], ir_tail: &ir::TailEdge) -> Result<Self, Error> {
        let mut nodes = Vec::with_capacity(ir_nodes.len());
        for (i, ir_node) in ir_nodes.iter().enumerate() {
            nodes.push(Node::from_ir(ir_node, i)?);
        }
        let tail = Tail::from_ir(ir_tail, ir_nodes.len())?;
        Ok(Self { nodes, tail })
    }

    /// Retrieves a reference to the node at `index`.
    ///
    /// # Panics
    ///
    /// Panics if no node exists at that index. Unlike a slice, there is no guarantee that
    /// if index `a` is valid, then all indexes less than `a` are valid as well.
    pub fn node(&self, index: NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    /// Retrieves a mutable reference to the node at `index`.
    ///
    /// # Panics
    ///
    /// Panics if no node exists at that index. Unlike a slice, there is no guarantee that
    /// if index `a` is valid, then all indexes less than `a` are valid as well.
    pub fn node_mut(&mut self, index: NodeIndex) -> &mut Node {
        &mut self.nodes[index.0]
    }

    pub fn tail(&self) -> &Tail {
        &self.tail
    }
    pub fn tail_mut(&mut self) -> &mut Tail {
        &mut self.tail
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn into_ir(&self) -> Result<(Vec<ir::Node>, ir::TailEdge), Error> {
        let mut ir_nodes = Vec::new();
        let mut node_map = HashMap::new();
        let mut traversal = traversals::DependencyFirst::new(&self);
        while let Some(index) = traversal.next(self)? {
            ir_nodes.push(self.node(index).to_ir(&node_map));
            node_map.insert(index, node_map.len());
        }
        let ir_tail = self.tail.to_ir(&node_map);
        Ok((ir_nodes, ir_tail))
    }
}

#[cfg(test)]
pub fn validate(pre_str: &str, operation: impl FnOnce(&mut Graph), post_str: &str) {
    let mut graph = {
        let funclet: ir::Funclet = ron::from_str(pre_str).unwrap();
        Graph::from_ir(&funclet.nodes, &funclet.tail_edge).unwrap()
    };
    operation(&mut graph);
    let post = {
        let funclet: ir::Funclet = ron::from_str(post_str).unwrap();
        Graph::from_ir(&funclet.nodes, &funclet.tail_edge).unwrap()
    };
    assert!(
        graph.tail.eq_deep(&graph, &post.tail, &post),
        "assertion failed: 
     left: {:#?}
    right: {:#?}",
        graph,
        post
    )
}

#[cfg(test)]
mod unused_code_elimination {
    use super::*;
    fn validate_uce(pre_str: &str, post_str: &str) {
        validate(pre_str, |_| (), post_str)
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
        validate_uce(empty_str, empty_str);
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
        validate_uce(all_used_str, all_used_str);
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
        validate_uce(pre_str, post_str);
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
        validate_uce(pre_str, post_str);
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
        validate_uce(pre_str, post_str);
    }
}
