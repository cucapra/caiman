#![warn(warnings)]
//! [[Tate09]](https://rosstate.org/publications/eqsat/eqsat_tate_popl09.pdf)
use crate::ir;
use thiserror::Error;

/// This error is produced when an [`ir::Dependent`](crate::ir) depends on a node which:
///   1. occurs after the dependent, or
///   2. doesn't exist at all
#[derive(Debug, Error)]
#[error("IR conversion error: {needed_by} incorrectly depends on node #{dependency}")]
pub struct FromIrError {
    /// The ID of the dependency
    pub dependency: ir::NodeId,
    /// The dependent which caused the failure.
    pub needed_by: ir::Dependent,
}

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

mod operation_kind;
use operation_kind::OperationKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeKind {
    /// An arbitrarily-sized list of child ids. The list ordering is meaningful.
    IdList,

    /// A funclet parameter (corresponds to a funclet phi node)
    Param {
        funclet: ir::FuncletId,
        index: usize,
    },

    /// A tail edge for a given funclet.
    Tail {
        funclet: ir::FuncletId,
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
    deps: Box<[egg::Id]>,
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
