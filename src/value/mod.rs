#![warn(warnings)]
//! [[Tate09]](https://rosstate.org/publications/eqsat/eqsat_tate_popl09.pdf)

/// Primitive functions. These are essential to the functionality of the E-PEG.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Primitive {
    /// A list of [`egg::Id`]s.
    ///
    /// Unlike all other nodes, `IdList` can have an arbitrary number of children.
    /// There are no semantics attached to these children; instead, their meaning is given
    /// by the parents of the `IdList` in question. The ordering of this list is meaningful.
    IdList,

    /// A `φ` node which uses a boolean to "choose" a value.
    ///
    /// The third child is a boolean "selector". If it's `false`, this node assumes the value
    /// of its first child. If it's `true`, this node assumes the value of its second child.
    Choice,

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
enum NodeData {
    Prim(Primitive),
    Func(Function),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node {
    /// The data contained by this node. This includes what kind of node it is, as well as any
    /// per-node attributes.
    data: NodeData,
    /// The indices of each of the node's dependencies. The order is significant. There is no
    /// guarantee that the entries are unique.
    /// This is a `Box<[NodeIndex]>` instead of a `Vec<NodeIndex>` in order to save space.
    /// You generally shouldn't be adding or removing dependencies anyways.
    deps: Box<[egg::Id]>,
}
impl egg::Language for Node {
    fn matches(&self, other: &Self) -> bool {
        self.data == other.data
    }
    fn children(&self) -> &[egg::Id] {
        &self.deps
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        &mut self.deps
    }
}
