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

mod from_op;

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

enum SealedRewrite {
    RunnerHook(GraphRunnerHook),
    RewriteRule(GraphRewriteRule),
}
/// An opaque value graph rewrite.
/// This may be an "abstract" rewrite which isn't actually implemented as a rewrite rule,
/// such as redundant branch elimination.
pub struct Rewrite(SealedRewrite);

type GraphInner = egg::EGraph<Node, Analysis>;
type GraphRunner = egg::Runner<Node, Analysis, ()>;
type GraphRunnerHook = Box<dyn FnMut(&mut GraphRunner) -> Result<(), String> + 'static>;
type GraphRewriteRule = egg::Rewrite<Node, Analysis>;
type GraphId = egg::Id;

pub struct Graph {
    runner: GraphRunner,
    rules: Vec<GraphRewriteRule>,
}
impl Graph {
    pub fn new(program: &ir::Program, head: ir::FuncletId) -> Result<Self, FromIrError> {
        // due to lifetime issues, we store the analysis separately while constructing it,
        // and then move it into the graph once we're done
        // TODO: does this hack cause issues with constant folding (lost class analyses?)
        let mut egraph = egg::EGraph::new(Analysis::new());
        let mut analysis = Analysis::new();
        analysis.build_with_graph(&mut egraph, program, head)?;
        egraph.analysis = analysis;
        // this looks weird because egg::Runner has a weird API, don't blame me!
        // I don't understand why there's not a constructor that takes an egraph...
        Ok(Self {
            runner: GraphRunner::new(Analysis::new()).with_egraph(egraph),
            rules: Vec::new(),
        })
    }
    pub fn add_rewrite(&mut self, rewrite: Rewrite) {
        match rewrite.0 {
            SealedRewrite::RunnerHook(hook) => {
                self.runner.hooks.push(hook);
            }
            SealedRewrite::RewriteRule(rule) => {
                self.rules.push(rule);
            }
        }
    }
    pub fn analyze(&mut self) {
        // `run` takes self by value and returns Self, instead of taking &mut self.
        // This
        let mut runner = GraphRunner::new(Analysis::new());
        // makes
        std::mem::swap(&mut runner, &mut self.runner);
        // me
        let runner = runner.run(&self.rules);
        // sad.
        self.runner = runner;
    }
    pub fn elaborate_into(&self, program: &mut ir::Program) -> Result<ir::FuncletId, IntoIrError> {
        todo!()
    }
}
