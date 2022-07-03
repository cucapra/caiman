//! Dataflow graph traversals.

use crate::dataflow::{Graph, NodeIndex, Tail};
use std::collections::HashMap;
use thiserror::Error;

/// A dependency cycle was encountered during graph traversal.
#[derive(Debug, Error)]
#[error("dependency cycle (includes node #{})", includes.0)]
pub struct DependencyCycle {
    /// A representative node of the cycle.
    pub includes: NodeIndex,
}

/// An abstract "command" for a traversal to follow. These are meant to be traversal-agnostic
/// and used inside traversal data structures (i.e. their stack).
enum Command {
    /// The traversal should visit this node.
    Visit(NodeIndex),

    /// The traversal should "leave" this node. The exact meaning of "leaving" a node depends
    /// on the traversal. For example, in a dependency-first traversal, a node is left after all
    /// of its dependencies have been left.
    Leave(NodeIndex),
}

/// Represents the "status" of a visited node.
enum VisitStatus {
    /// The node — but not all of its dependencies — have been visited.
    Working,

    /// The node — and all its dependencies — have been visited.
    Done,
}

/// A **dependency-first** traversal of the reachable nodes in a graph.
///
/// (TODO: Properly define "reachable". Currently it means "referenced by a tail edge" but
/// this may change with the addition of stateful value operations like `print`.)
///
/// More specifically, this implements a postorder depth-first search, which is equivalent
/// to a reversed topological sort in an acyclic graph. (In a traditional topological sort,
/// dependencies have edges *to* their dependents, but in a dataflow graph dependencies have
/// edges *from* their dependents. So, in a dataflow graph, a *reversed* topological sort captures
/// the intuition that dependencies should come before their dependents.)
///
/// This traversal validates that the graph (or at least the subgraph induced by its reachable
/// nodes) is acyclic.
///
/// An instance does not retain a reference to its graph. This has a few consequences:
/// - You can mutate the graph mid-traversal.
/// - You *must* call [`next`](Self::next) with the same graph used to create the instance.
/// - If you change a node's dependencies after the traversal has left that node, the new
///   dependencies won't be added to the search.
pub struct DependencyFirst {
    stack: Vec<Command>,
    visited: HashMap<NodeIndex, VisitStatus>,
}
impl DependencyFirst {
    /// Creates a new dependency-first traversal of `graph`.
    pub fn new(graph: &Graph) -> Self {
        let visited = HashMap::new();
        let mut stack = Vec::new();
        graph.tail.for_each_dependency(|&i| {
            stack.push(Command::Visit(i)) //
        });
        Self { stack, visited }
    }
    /// Retrieves the next node in the traversal. `graph` must be the same graph which was used to
    /// construct this instance. Returns an error upon encountering a dependency cycle.
    ///
    /// The exact order of traversal is unspecified, but the following invariants hold:
    /// - Each reachable node will be returned (assuming the graph is acyclic)
    /// - A node will only be returned from `next` if all its dependencies have already been
    ///   returned from `next`.
    pub fn next(&mut self, graph: &Graph) -> Result<Option<NodeIndex>, DependencyCycle> {
        loop {
            match self.stack.pop() {
                None => return Ok(None),
                Some(Command::Leave(index)) => {
                    self.visited.insert(index, VisitStatus::Done);
                    return Ok(Some(index));
                }
                Some(Command::Visit(index)) => {
                    let status = self.visited.insert(index, VisitStatus::Working);
                    if let Some(VisitStatus::Working) = status {
                        return Err(DependencyCycle { includes: index });
                    }
                    self.stack.push(Command::Leave(index));
                    graph.operation(index).for_each_dependency(|&i| {
                        self.stack.push(Command::Visit(i)) //
                    });
                }
            }
        }
    }
}
