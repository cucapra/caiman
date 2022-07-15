#![warn(warnings)]

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeData {
    Bundle,
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
