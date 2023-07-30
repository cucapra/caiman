use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::ir::Place;
use crate::assembly::explication::context::Context;
use crate::assembly::explication::util::*;
use crate::{assembly, frontend, ir};

// find an available allocation
// fills in an allocation if needed
// fn explicate_allocation (
//     spec_funclet: &FuncletId,
//     spec_node: &NodeId,
//     place: &Place,
//     storage_type: &TypeId,
//     context: &mut Context,
// ) -> RemoteNodeId {
//
// }

pub fn resolve_allocation (
    place: Hole<Place>,
    ffi_type: Hole<FFIType>,
    context: &mut Context,
) {
    context.add_available_allocation(context.location_node().clone(), ffi_type, place);
}

// external_id is None if it's a builtin
pub fn resolve_local_do (
    operation: Hole<ast::Quotient>,
    external_id: Option<Hole<ExternalFunctionId>>,
    inputs: Hole<Vec<Hole<NodeId>>>,
    outputs: Hole<Vec<Hole<NodeId>>>,
    context: &mut Context
) {
    // context.add_available_write(context.location_node().clone())
}

pub fn resolve_read_ref (
    source: &NodeId,
    storage_type: Hole<FFIType>,
    context: &mut Context
) {

}