#![warn(warnings)]
use crate::ir;
use std::collections::hash_map::{Entry, HashMap};

mod from_ir;
pub use from_ir::FromIrError;

mod into_ir;

mod egg_cfg;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constant {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

#[derive(Debug)]
struct ClassAnalysis {
    constant: Option<Constant>,
}
impl ClassAnalysis {
    fn new(_egraph: &egg::EGraph<Node, GlobalAnalysis>, enode: &Node) -> Self {
        let mut constant = None;
        if let NodeKind::Operation { kind } = &enode.kind {
            if let OperationKind::ConstantInteger { value, .. } = kind {
                constant = Some(Constant::I64(*value));
            } else if let OperationKind::ConstantUnsignedInteger { value, .. } = kind {
                constant = Some(Constant::U64(*value));
            }
        }
        Self { constant }
    }
    fn merge(&mut self, other: Self) -> egg::DidMerge {
        let constant_merge = match (self.constant, other.constant) {
            (None, None) => egg::DidMerge(false, false),
            (Some(_), None) => egg::DidMerge(false, true),
            (None, mut b @ Some(_)) => {
                self.constant = b.take();
                egg::DidMerge(true, false)
            }
            (Some(a), Some(b)) => {
                assert!(a == b, "graph rewrite violated the type system");
                egg::DidMerge(false, false)
            }
        };
        constant_merge
    }
}
struct GlobalAnalysis {}
impl egg::Analysis<Node> for GlobalAnalysis {
    type Data = ClassAnalysis;
    fn make(egraph: &egg::EGraph<Node, Self>, enode: &Node) -> Self::Data {
        ClassAnalysis::new(egraph, enode)
    }
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> egg::DidMerge {
        a.merge(b)
    }
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

type GraphInner = egg::EGraph<Node, GlobalAnalysis>;
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
