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
    /// Retrieves the next node in the traversal, or `None` if all reachable nodes have been
    /// traversed. `graph` must be the same graph which was used to construct this instance.
    ///
    /// The exact order of traversal is unspecified, but the following invariants hold:
    /// - Each reachable node will be returned (assuming the graph is acyclic)
    /// - A node will only be returned from `next` if all its dependencies have already been
    ///   returned from `next`.
    ///
    /// # Errors
    /// An error will be returned if a dependency cycle is detected. Calling [`next`](Self::next)
    /// again may behave incorrectly.
    pub fn next(&mut self, graph: &Graph) -> Result<Option<NodeIndex>, DependencyCycle> {
        loop {
            let index = match self.stack.pop() {
                None => return Ok(None),
                Some(Command::Leave(resolved)) => {
                    self.visited.insert(resolved, VisitStatus::Done);
                    return Ok(Some(resolved));
                }
                Some(Command::Visit(index)) => index,
            };
            let resolved = graph.resolve_index(index);
            match self.visited.insert(resolved, VisitStatus::Working) {
                Some(VisitStatus::Working) => return Err(DependencyCycle { includes: resolved }),
                Some(VisitStatus::Done) => (),
                None => {
                    self.stack.push(Command::Leave(resolved));
                    graph.operation(resolved).for_each_dependency(|&i| {
                        self.stack.push(Command::Visit(i)) //
                    });
                }
            }
        }
    }
}
