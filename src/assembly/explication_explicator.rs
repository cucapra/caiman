use crate::assembly::explication_context::Context;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_context::FuncletLocation;
use crate::ir::ffi;
use crate::{assembly_ast, assembly_context, frontend, ir};
use std::any::Any;
use std::collections::HashMap;
use crate::assembly::explication_util::*;

fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => todo!(),
    }
}

pub fn explicate_allocate_temporary(
    place: &Hole<ir::Place>,
    storage_type: &Hole<assembly_ast::StorageTypeId>,
    operation: &Hole<assembly_ast::RemoteNodeId>,
    context: &mut Context,
) -> Option<ir::Node> {
    Some(ir::Node::AllocTemporary {
        place: reject_hole(place.as_ref()).clone(),
        storage_type: ffi::TypeId(
            context
            .inner()
            .loc_type_id(reject_hole(storage_type.clone())),
        ),
        operation: remote_conversion(reject_hole(operation.as_ref()), context),
    })
}

pub fn explicate_encode_do(
    place_hole: &Hole<ir::Place>,
    operation_hole: &Hole<assembly_ast::RemoteNodeId>,
    inputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    outputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = place_hole.unwrap_or(todo!());
    // Some(ir::Node::EncodeDo {
    //     place: reject_hole(place.clone()),
    //     operation: remote_conversion(reject_hole_ref(operation), context),
    //     inputs: reject_hole_ref(inputs)
    //         .iter()
    //         .map(|n| context.inner().node_id(reject_hole(n.clone())))
    //         .collect(),
    //     outputs: reject_hole_ref(outputs)
    //         .iter()
    //         .map(|n| context.inner().node_id(reject_hole(n.clone())))
    //         .collect(),
    // })
}
