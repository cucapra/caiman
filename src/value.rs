#![warn(warnings)]
use crate::ir;

mod from_ir;
pub use from_ir::FromIrError;

mod into_ir;
pub use into_ir::IntoIrError;

mod analysis;
use analysis::Analysis;

mod operation_kind;
pub use operation_kind::OperationKind;

mod constant;
mod from_op;
mod rewrite;

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
impl Node {
    // convenience function, almost like "unwrap"
    fn operation(&self) -> Option<&'_ OperationKind> {
        match &self.kind {
            NodeKind::Operation { kind } => Some(kind),
            _ => None,
        }
    }
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

type Graph = egg::EGraph<Node, Analysis>;
type GraphRunner = egg::Runner<Node, Analysis, ()>;
type GraphHook = Box<dyn FnMut(&mut GraphRunner) -> Result<(), String> + 'static>;
type GraphRewrite = egg::Rewrite<Node, Analysis>;
type GraphId = egg::Id;
type GraphClass = egg::EClass<Node, analysis::ClassAnalysis>;

pub struct Optimizer {
    /// Invariant: This is always Some(_). Why is it an option? See `optimize`
    runner: Option<GraphRunner>,
    rules: Vec<GraphRewrite>,
}
impl Optimizer {
    pub fn new(program: &ir::Program, head: ir::FuncletId) -> Result<Self, FromIrError> {
        let runner = analysis::create(program, head)?;
        analysis::validate(&runner);
        Ok(Self {
            runner: Some(runner),
            rules: Vec::new(),
        })
    }
    pub fn optimize(&mut self) {
        // `run` takes self by value and returns Self, instead of taking &mut self.
        // This makes me incredibly sad, especially since there's literally no technical reason why.
        let runner = self.runner.take().expect("invariant violated");
        let runner = runner.run(&self.rules);
        self.runner = Some(runner);
    }
    pub fn write_into(&self, program: &mut ir::Program) -> Result<ir::FuncletId, IntoIrError> {
        todo!()
    }
}
