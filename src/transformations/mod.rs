#![warn(warnings)]
use crate::dataflow::{traversals, Graph, NodeIndex};
use crate::ir;
use thiserror::Error;

mod basic_cse;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown transformation: {0}")]
    UnknownTransformation(String),
    #[error("invalid num iterations: {0}")]
    InvalidNumIterations(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ValueError(#[from] crate::dataflow::Error),
}

pub struct Transformer {
    /// The maximum number of transformation iterations to apply.
    max_iterations: usize,
    /// Whether to run basic constant subexpression elimination.
    basic_cse: bool,
    /// The list of subgraph transforms to apply.
    transforms: Vec<&'static dyn SubgraphTransform>,
}
impl Transformer {
    pub fn new(options: &[&str]) -> Result<Self, Error> {
        let mut cfg = Self {
            max_iterations: 32,
            basic_cse: false,
            transforms: Vec::new(),
        };
        for &opt in options {
            if opt == "basic-cse" {
                cfg.basic_cse = true;
            } else if let Some(maybe_iters) = opt.strip_prefix("iterations=") {
                cfg.max_iterations = maybe_iters.parse()?;
            } else {
                return Err(Error::UnknownTransformation(opt.to_owned()));
            }
        }
        Ok(cfg)
    }
    pub fn apply(&self, program: &mut ir::Program) -> Result<(), Error> {
        for (_, funclet) in program.funclets.iter_mut() {
            let mut graph = Graph::from_ir(&funclet.nodes, &funclet.tail_edge)?;
            for _ in 0..self.max_iterations {
                if self.basic_cse {
                    basic_cse::apply(&mut graph)?;
                }
                let mut mutated = false;
                let mut traversal = traversals::DependencyFirst::new(&graph);
                while let Some(index) = traversal.next(&graph)? {
                    for transform in self.transforms.iter() {
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
