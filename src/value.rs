#![warn(warnings)]
use crate::ir;
use std::collections::hash_map::{Entry, HashMap};

mod from_ir;
pub use from_ir::FromIrError;

mod into_ir;

mod analysis;
use analysis::Analysis;

mod operation_kind;
pub use operation_kind::OperationKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    /// An arbitrarily-sized list of graph ids. The list ordering is meaningful.
    IdList,

    /// A funclet parameter (corresponds to a funclet phi node)
    Param {
        funclet_id: ir::FuncletId,
        index: usize,
    },

    Operation {
        kind: OperationKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    /// The node's type, including any constant attributes.
    kind: NodeKind,
    /// The indices of each of the node's children. The order is significant. There is no
    /// guarantee that the entries are unique.
    /// This is a `Box<[NodeIndex]>` instead of a `Vec<NodeIndex>` in order to save space.
    /// You generally shouldn't be adding or removing children anyways.
    deps: Box<[GraphId]>,
}
impl egg::Language for Node {
    fn matches(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
    fn children(&self) -> &[egg::Id] {
        &self.deps
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        &mut self.deps
    }
}

type GraphInner = egg::EGraph<Node, Analysis>;
type GraphId = egg::Id;

pub struct Graph(GraphInner);
impl Graph {
    pub fn new(program: &ir::Program, start: ir::FuncletId) -> Result<Self, FromIrError> {
        // due to lifetime issues, we store the analysis separately while constructing it,
        // and then move it into the graph once we're done
        let mut graph = Self(egg::EGraph::new(Analysis::new()));
        let mut analysis = Analysis::new();
        analysis.build_with_graph(&mut graph.0, program, start)?;
        graph.0.analysis = analysis;
        Ok(graph)
    }
}
