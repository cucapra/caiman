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
}

mod generated;
pub use generated::{Node, Tail};
pub mod traversals;

#[derive(Debug, Clone, Copy, Error)]
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

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {
    /// This error is produced when an [`ir::Node`](crate::ir) or [`ir::TailEdge`](crate::ir)
    /// depends on a node which:
    /// 1. occurs after the dependency, or
    /// 2. doesn't exist at all
    #[error("{required_by} incorrectly depends on IR node #{dependency}")]
    IrDependency {
        /// The ID of the dependency
        dependency: ir::NodeId,
        /// The dependent which caused the failure.
        required_by: IrDependent,
    },
    #[error("dependency cycle (includes node #{})", includes.0)]
    DependencyCycle {
        /// A representative node of the cycle.
        includes: NodeIndex,
    },
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

    /// Retrieves a reference to the graph's tail edge.
    pub fn tail(&self) -> &Tail {
        &self.tail
    }

    /// Retrieves a mutable reference to the graph's tail edge.
    pub fn tail_mut(&mut self) -> &mut Tail {
        &mut self.tail
    }

    /// Returns the number of nodes in the graph. (The tail does not count as a node.)
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
