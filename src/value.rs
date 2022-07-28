#![warn(warnings)]
//! [[Tate09]](https://rosstate.org/publications/eqsat/eqsat_tate_popl09.pdf)
use crate::ir;
use thiserror::Error;

mod analysis;

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

/// Primitive functions. These are only used internally by the egraph and don't show up in the
/// output IR.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Primitive {
    /// An arbitrarily-sized list of child [`egg::Id`]s. The list ordering is meaningful.
    IdList,

    /// A generalization of a `φ` node which uses a integer to "choose" a value.
    ///
    /// The first child is the selector, which evaluates to some unsigned integer `i`. The
    /// switch node as a whole evaluates to the value of its `i+1`th child.
    Switch,

    /// An abstract representation of a sequence of values. This is called `θ` in [[Tate09]].
    ///
    /// The first child is the sequence's initial value. The second child is an expression which
    /// gives the current value; usually, this expression is recursive.
    SeqExpr,

    /// Extracts a given value from a `Sequence`. This is called `eval` in [[Tate09]].
    ///
    /// The first child is the value sequence; the second child is the index of the desired value.
    SeqExtract,

    /// Represents the minimum `i ∈ ℕ` such that the `i`th element in the child sequence is true.
    /// This is called `pass` in [[Tate09]].
    SeqFirst,
}

/// Semantic functions. These are language-specific.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Function {
    Todo,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Operation {
    Prim(Primitive),
    Func(Function),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node {
    /// The operation represented by this node. This does *NOT* include the node's children.
    operation: Operation,
    /// The indices of each of the node's children. The order is significant. There is no
    /// guarantee that the entries are unique.
    /// This is a `Box<[NodeIndex]>` instead of a `Vec<NodeIndex>` in order to save space.
    /// You generally shouldn't be adding or removing children anyways.
    deps: Box<[egg::Id]>,
}
impl egg::Language for Node {
    fn matches(&self, other: &Self) -> bool {
        self.operation == other.operation
    }
    fn children(&self) -> &[egg::Id] {
        &self.deps
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        &mut self.deps
    }
}
