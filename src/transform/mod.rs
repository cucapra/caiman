#![warn(warnings)]
use crate::dataflow::{traversals, Graph, NodeIndex};
use crate::ir;
use thiserror::Error;

mod basic_cse;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown transformation: {0}")]
    UnknownTransform(String),
    #[error(transparent)]
    ValueError(#[from] crate::dataflow::Error),
}

pub struct TransformConfig {
    /// The maximum number of transformation passes to run.
    max_passes: usize,
    /// Whether to run basic constant subexpression elimination.
    basic_cse: bool,
    /// The list of subgraph transforms to apply.
    transforms: Vec<Box<dyn SubgraphTransform>>,
}
impl TransformConfig {
    pub const DEFAULT_MAX_PASSES: usize = 16;
    pub fn new(max_passes: usize) -> Self {
        Self {
            max_passes,
            basic_cse: false,
            transforms: Vec::new(),
        }
    }
    pub fn add_transform(&mut self, transform: &str) -> Result<&mut Self, Error> {
        match transform {
            "basic-cse" => self.basic_cse = true,
            unknown => return Err(Error::UnknownTransform(unknown.to_owned())),
        }
        Ok(self)
    }
}
impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            max_passes: Self::DEFAULT_MAX_PASSES,
            basic_cse: true,
            transforms: Vec::new(),
        }
    }
}
/// Represents an transformation on a subgraph of a dataflow graph.
trait SubgraphTransform {
    /// Attempts to apply the transformation to the subgraph of `graph` induced by `index`
    /// and all of its indirect and direct dependencies. The return code indicates success.
    ///
    /// This **should not** modify any nodes outside of the aforementioned subtree.
    ///
    /// # Mutability
    /// Although [`SubgraphTransform`]s are free to mutate their subgraph, they can't mutate
    /// themselves; if self-mutability is needed, you're probably better off writing a
    /// "freestanding" transformation. This restriction exists for two reasons:
    /// - Since a [`SubgraphTransform`] can't keep track of which nodes its already visited,
    ///   it can't modify nodes outside of its subtree without using out-of-band information
    ///   to obtain their node indices. (That is, it's difficult to break the trait contract
    ///   unless you intentionally "cheat".)
    /// - If a transform *requires* self-mutability, it must maintain some internal state. That
    ///   state could be accidentally invalidated by other transformations running in parallel.
    ///   For example, [`BasicCse`](basic_cse::BasicCse) maintains an internal hashmap from node
    ///   contents to node indices. A transformation interspersed between `BasicCse` applications
    ///   could mutate node contents, thus de-syncing the hashmap from the graph.
    ///   Self-immutability helps avoid these footguns.
    fn attempt(&self, graph: &mut Graph, index: NodeIndex) -> bool;
}

pub fn apply(config: &TransformConfig, program: &mut ir::Program) -> Result<(), Error> {
    for (_, funclet) in program.funclets.iter_mut() {
        let mut graph = Graph::from_ir(&funclet.nodes, &funclet.tail_edge)?;
        for _ in 0..config.max_passes {
            if config.basic_cse {
                basic_cse::apply(&mut graph)?;
            }
            let mut mutated = false;
            let mut traversal = traversals::DependencyFirst::new(&graph);
            while let Some(index) = traversal.next(&graph)? {
                for transform in config.transforms.iter() {
                    mutated |= transform.attempt(&mut graph, index);
                }
            }
            if !mutated {
                break;
            }
        }
        let (ir_nodes, ir_tail) = graph.into_ir()?;
        funclet.nodes = ir_nodes.into_boxed_slice();
        funclet.tail_edge = ir_tail;
    }
    Ok(())
}
