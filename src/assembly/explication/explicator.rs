use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::context::Context;
use crate::assembly::explication::util::*;
use crate::{assembly, frontend, ir};

fn explicate_allocation(
    node: &NodeId,
    place: Hole<ir::Place>,
    storage_type: Hole<ast::TypeId>,
    context: &mut Context,
) {

}
