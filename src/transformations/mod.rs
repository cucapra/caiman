#![warn(warnings)]
use crate::dataflow;
use crate::ir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Dataflow(#[from] dataflow::Error),
}

/// Represents a transformation on a subgraph of a dataflow graph.
trait SubgraphTransform {
    /// Attempts to apply the transformation to the subgraph of `graph` induced by `index`
    /// and all of its indirect and direct dependencies. The return code indicates success.
    fn attempt(&mut self, graph: &mut dataflow::Graph, index: dataflow::NodeIndex) -> bool;
}

fn attempt_subgraph_transforms(
    graph: &mut dataflow::Graph,
    transforms: &mut [Box<dyn SubgraphTransform>],
) -> Result<bool, Error> {
    let mut mutated = false;
    let mut traversal = dataflow::traversals::DependencyFirst::new(graph);
    while let Some(index) = traversal.next(graph).map_err(dataflow::Error::from)? {
        for transform in transforms.iter_mut() {
            mutated |= transform.attempt(graph, index);
        }
    }
    Ok(mutated)
}
pub fn optimize(program: &mut ir::Program) -> Result<(), Error> {
    for (_, funclet) in program.funclets.iter_mut() {
        let mut graph = dataflow::Graph::from_ir(&funclet.nodes, &funclet.tail_edge)?;
        const MAX_ITERATIONS: usize = 32;
        for _ in 0..MAX_ITERATIONS {
            let mutated = attempt_subgraph_transforms(&mut graph, /* todo */ &mut [])?;
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
/*
pub fn apply(
    funclet: ir::Funclet,
    transforms: &mut [&mut dyn TreeTransformer],
) -> Result<bool, Error> {

}
*/
