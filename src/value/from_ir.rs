use crate::ir;
use crate::value;
use std::collections::HashMap;
use thiserror::Error;

/// This error is produced when an [`ir::Dependent`](crate::ir) depends on a node which:
///   1. occurs after the dependent, or
///   2. doesn't exist at all
#[derive(Debug, Error)]
#[error("IR conversion error (funclet {funclet_id}): {needed_by} depends on node #{dependency_id}")]
pub struct FromIrError {
    /// The ID of the funclet where the error occurred
    pub funclet_id: ir::FuncletId,
    /// The node ID of the dependency
    pub dependency_id: ir::NodeId,
    /// The dependent which caused the failure.
    pub needed_by: ir::Dependent,
}

struct FuncletCvtCtx<'a> {
    funclet_id: ir::FuncletId,
    node_ids: HashMap<ir::NodeId, value::GraphId>,
    graph: &'a mut value::GraphInner,
}

fn convert_node_id(
    context: &FuncletCvtCtx,
    node_id: ir::NodeId,
    needed_by: ir::Dependent,
) -> Result<value::GraphId, FromIrError> {
    context.node_ids.get(&node_id).copied().ok_or(FromIrError {
        funclet_id: context.funclet_id,
        dependency_id: node_id,
        needed_by,
    })
}
fn bundle_node_ids(
    context: &mut FuncletCvtCtx,
    node_ids: &[ir::NodeId],
    needed_by: ir::Dependent,
) -> Result<value::GraphId, FromIrError> {
    let deps: Box<_> = node_ids
        .iter()
        .map(|&node_id| convert_node_id(context, node_id, needed_by))
        .collect::<Result<_, _>>()?;
    let bundled_id = context.graph.add(value::Node {
        kind: value::NodeKind::IdList,
        deps,
    });
    Ok(bundled_id)
}
fn convert_tail(
    context: &mut FuncletCvtCtx,
    tail: &ir::TailEdge,
) -> Result<value::GraphId, FromIrError> {
    let needed_by = ir::Dependent::Tail;
    let mut deps = Vec::new();
    let kind = match tail {
        ir::TailEdge::Return { return_values } => {
            deps.push(bundle_node_ids(context, &return_values, needed_by)?);
            value::TailKind::Return
        }
        ir::TailEdge::Jump(jump) => {
            deps.push(bundle_node_ids(context, &jump.args, needed_by)?);
            value::TailKind::Jump {
                target: jump.target,
            }
        }
        ir::TailEdge::Switch { key, cases } => {
            for jump in cases.iter() {
                deps.push(bundle_node_ids(context, &jump.args, needed_by)?);
            }
            deps.push(convert_node_id(context, *key, needed_by)?);
            value::TailKind::Switch {
                targets: cases.iter().map(|jump| jump.target).collect(),
            }
        }
    };
    let graph_id = context.graph.add(value::Node {
        kind: value::NodeKind::Tail {
            funclet_id: context.funclet_id,
            kind,
        },
        deps: deps.into_boxed_slice(),
    });
    Ok(graph_id)
}

fn convert_node(
    context: &mut FuncletCvtCtx,
    node_id: ir::NodeId,
    node: &ir::Node,
) -> Result<value::GraphId, FromIrError> {
    let graph_node = if let &ir::Node::Phi { index } = node {
        value::Node {
            kind: value::NodeKind::Param {
                funclet_id: context.funclet_id,
                index,
            },
            deps: Box::new([]),
        }
    } else {
        let needed_by = ir::Dependent::Node(node_id);
        let mut deps = Vec::new();
        macro_rules! build_dep {
            ($arg:ident : [Operation]) => {
                deps.push(bundle_node_ids(context, $arg, needed_by)?)
            };
            ($arg:ident : Operation) => {
                deps.push(convert_node_id(context, *$arg, needed_by)?)
            };
            ($arg:ident : $arg_type:tt) => {};
        }
        macro_rules! build_deps {
            ($($_l:ident $name:ident ( $($arg:ident : $arg_type:tt,)* ) -> $_o:ident;)*) => {
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
    let graph_id = context.graph.add(graph_node);
    context.node_ids.insert(node_id, graph_id);
    Ok(graph_id)
}

pub(super) fn convert_funclet(
    graph: &mut value::GraphInner,
    program: &ir::Program,
    funclet_id: ir::FuncletId,
) -> Result<value::GraphId, FromIrError> {
    let mut context = FuncletCvtCtx {
        funclet_id,
        node_ids: HashMap::new(),
        graph,
    };
    for (node_id, node) in program.funclets[&funclet_id].nodes.iter().enumerate() {
        convert_node(&mut context, node_id, node)?;
    }
    convert_tail(&mut context, &program.funclets[&funclet_id].tail_edge)
}
