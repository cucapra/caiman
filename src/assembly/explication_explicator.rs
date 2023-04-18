use crate::assembly::explication_context::Context;
use crate::assembly::explication_util::*;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_context::FuncletLocation;
use crate::ir::ffi;
use crate::{assembly_ast, assembly_context, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => todo!(),
    }
}

pub fn explicate_allocate_temporary(
    place_hole: &Hole<ir::Place>,
    storage_type_hole: &Hole<assembly_ast::StorageTypeId>,
    operation_hole: &Hole<assembly_ast::RemoteNodeId>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = reject_hole(place_hole.as_ref());
    let storage_type = reject_hole(storage_type_hole.as_ref());
    let operation = reject_hole(operation_hole.as_ref());
    context.add_allocation(operation);
    Some(ir::Node::AllocTemporary {
        place: place.clone(),
        storage_type: ffi::TypeId(context.inner.loc_type_id(storage_type.clone())),
        operation: remote_conversion(operation, context),
    })
}

fn explicate_known_operation(
    operation: assembly_ast::RemoteNodeId,
    input_vec: &Box<[Hole<assembly_ast::OperationId>]>,
    output_vec: &Box<[Hole<assembly_ast::OperationId>]>,
    context: &mut Context,
) -> Option<(ir::RemoteNodeId, Box<[ir::OperationId]>, Box<[ir::OperationId]>)> {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for (index, input) in input_vec.iter().enumerate() {
        match input {
            Some(n) => inputs.push(context.inner.node_id(n.clone())),
            None => {
                let node = context.node_lookup(&operation);
                match node {
                    assembly_ast::Node::Constant { .. } => {
                        // nothing to fill
                    }
                    _ => todo!("Unsupported node {:?}", node),
                }
            }
        }
    }

    for (index, output) in output_vec.iter().enumerate() {
        match output {
            Some(n) => outputs.push(context.inner.node_id(n.clone())),
            None => {
                let node = context.node_lookup(&operation);
                match node {
                    assembly_ast::Node::Constant { .. } => {
                        match context.get_allocation(&operation) {
                            None => return None, // failed to explicated on this pass
                            Some(alloc_loc) => {
                                assert_eq!(
                                    alloc_loc.funclet_id.clone(),
                                    context.inner.current_funclet_name()
                                );
                                outputs.push(context.inner.node_id(alloc_loc.node_id.clone()))
                            }
                        }
                    }
                    _ => todo!("Unsupported node {:?}", node),
                }
            }
        }
    }

    Some((remote_conversion(&operation, context), inputs.into_boxed_slice(), outputs.into_boxed_slice()))
}

pub fn explicate_encode_do(
    place_hole: &Hole<ir::Place>,
    operation_hole: &Hole<assembly_ast::RemoteNodeId>,
    inputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    outputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = reject_hole(place_hole.clone());
    let mut input_vec = reject_hole(inputs_hole.as_ref());
    let mut output_vec = reject_hole(outputs_hole.as_ref());
    let result = match operation_hole.as_ref() {
        Some(op) => explicate_known_operation(op.clone(), input_vec, output_vec, context),
        None => todo!()
    };
    // a bit sloppy, but oh well
    let (operation, inputs, outputs) = match result {
        None => { return None },
        Some(t) => t
    };
    Some(ir::Node::EncodeDo {
        place,
        operation,
        inputs,
        outputs,
    })
}
