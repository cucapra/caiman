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

pub struct Graph {
    inner: GraphInner,
    tail_map: HashMap<ir::FuncletId, GraphId>,
}
impl Graph {
    pub fn new(
        program: &ir::Program,
        entry_funclet_id: ir::FuncletId,
    ) -> Result<Self, FromIrError> {
        todo!();
        /*let mut stack = vec![entry_funclet_id];
        let mut graph = Self {
            inner: egg::EGraph::new(()),
            tail_map: HashMap::new(),
        };
        while let Some(funclet_id) = stack.pop() {
            if let Entry::Vacant(spot) = graph.tail_map.entry(funclet_id) {
                let mut converter = from_ir::FuncletConverter::new(&mut graph.inner, funclet_id);
                for (node_id, node) in program.funclets[&funclet_id].nodes.iter().enumerate() {
                    converter.add_node(node, node_id)?;
                }
                let tail_id = converter.add_tail(&program.funclets[&funclet_id].tail_edge)?;
                spot.insert(tail_id);
                program.funclets[&funclet_id]
                    .tail_edge
                    .for_each_funclet(|id| stack.push(id));
            }
        }
        Ok(graph)*/
    }
}
