use crate::ir;
use crate::value;
use std::collections::HashMap;
use thiserror::Error;

/// This error is produced when an [`ir::Dependent`](crate::ir) depends on a node which:
///   1. occurs after the dependent, or
///   2. doesn't exist at all
#[derive(Debug, Error)]

pub enum FromIrError {
    #[error(
        "IR conversion error (funclet {funclet_id}): {needed_by} depends on node #{dependency_id}"
    )]
    InvalidDependency {
        /// The ID of the funclet where the error occurred
        funclet_id: ir::FuncletId,
        /// The node ID of the dependency
        dependency_id: ir::NodeId,
        /// The dependent which caused the failure.
        needed_by: ir::Dependent,
    },
}

type Result = std::result::Result<value::GraphId, FromIrError>;

pub struct FuncletConverter<'a> {
    funclet_id: ir::FuncletId,
    node_ids: HashMap<ir::NodeId, value::GraphId>,
    egraph: &'a mut value::Graph,
}
impl<'a> FuncletConverter<'a> {
    pub(super) fn new(egraph: &'a mut value::Graph, funclet_id: ir::FuncletId) -> Self {
        Self {
            funclet_id,
            node_ids: HashMap::new(),
            egraph,
        }
    }
    pub fn convert_node_id(&self, node_id: ir::NodeId, needed_by: ir::Dependent) -> Result {
        self.node_ids
            .get(&node_id)
            .copied()
            .ok_or(FromIrError::InvalidDependency {
                funclet_id: self.funclet_id,
                dependency_id: node_id,
                needed_by,
            })
    }
    pub fn make_id_list(&mut self, node_ids: &[ir::NodeId], needed_by: ir::Dependent) -> Result {
        let deps: Box<_> = node_ids
            .iter()
            .map(|&node_id| self.convert_node_id(node_id, needed_by))
            .collect::<std::result::Result<_, _>>()?;
        let bundled_id = self.egraph.add(value::Node {
            kind: value::NodeKind::IdList,
            deps,
        });
        Ok(bundled_id)
    }
    pub fn add_node(&mut self, node: &ir::Node, node_id: ir::NodeId) -> Result {
        let graph_node = if let &ir::Node::Phi { index } = node {
            value::Node {
                kind: value::NodeKind::Param {
                    funclet_id: self.funclet_id,
                    index,
                },
                deps: Box::new([]),
            }
        } else {
            let needed_by = ir::Dependent::Node(node_id);
            let mut deps = Vec::new();
            macro_rules! build_dep {
                ($arg:ident : [Operation]) => {
                    deps.push(self.make_id_list($arg, needed_by)?)
                };
                ($arg:ident : Operation) => {
                    deps.push(self.convert_node_id(*$arg, needed_by)?)
                };
                ($arg:ident : $arg_type:tt) => {};
            }
            macro_rules! build_deps {
                ($($_l:ident $name:ident ( $($arg:ident : $arg_type:tt,)* ) -> $_o:ident;)*) => {
                    #[allow(unused_variables)]
                    match node { $(ir::Node::$name { $($arg),* } => {$(build_dep!($arg : $arg_type);)*})* }
                };
            }
            with_operations!(build_deps);
            let kind = value::OperationKind::from_ir_node(node);
            value::Node {
                kind: value::NodeKind::Operation { kind },
                deps: deps.into_boxed_slice(),
            }
        };
        let egraph_id = self.egraph.add(graph_node);
        self.node_ids.insert(node_id, egraph_id);
        Ok(egraph_id)
    }
}
