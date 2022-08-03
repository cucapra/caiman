#![warn(warnings)]
use crate::ir;

mod from_ir;
pub use from_ir::FromIrError;

mod operation_kind;
use operation_kind::OperationKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TailKind {
    // One child dependency, an `IdList` of captured arguments.
    Return,

    // Same as above.
    Jump { target: ir::FuncletId },

    // One `IdList` dependency for each targets, and an additional final dependency
    // for the selector.
    Switch { targets: Box<[ir::FuncletId]> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeKind {
    /// An arbitrarily-sized list of child ids. The list ordering is meaningful.
    IdList,

    /// A funclet parameter (corresponds to a funclet phi node)
    Param {
        funclet_id: ir::FuncletId,
        index: usize,
    },

    /// A tail edge for a given funclet.
    Tail {
        funclet_id: ir::FuncletId,
        kind: TailKind,
    },

    Operation {
        kind: OperationKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node {
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

type Graph = egg::EGraph<Node, ()>;
type GraphId = egg::Id;
