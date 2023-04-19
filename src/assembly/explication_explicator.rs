use crate::assembly::explication_context::Context;
use crate::assembly::explication_util::*;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_context::FuncletLocation;
use crate::ir::ffi;
use crate::{assembly_ast, assembly_context, frontend, ir};
use crate::assembly_ast::{
    ExternalCpuFunction, ExternalGpuFunction, FuncletId, NodeId, OperationId,
    StorageTypeId, TypeId, ValueFunctionId,
};
use std::any::Any;
use std::collections::HashMap;
use crate::stable_vec::StableVec;

fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => todo!(),
    }
}

fn find_filled<T>(v: Vec<Hole<T>>) -> StableVec<T> {
    let mut result = StableVec::new();
    for (index, hole) in v.into_iter().enumerate() {
        match hole {
            Some(value) => { result.add(value); }
            None => {}
        }
    };
    result
}

fn find_filled_hole<T>(h: Hole<Box<[Hole<T>]>>) -> StableVec<T> {
    match h {
        Some(v) => find_filled(v.into_vec()),
        None => StableVec::new()
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

fn infer_operation(
    known_inputs: &StableVec<OperationId>,
    known_outputs: &StableVec<OperationId>,
    context: &mut Context
) -> Option<assembly_ast::RemoteNodeId> {
    None
}

fn explicate_operation(
    operation_hole: &Hole<assembly_ast::RemoteNodeId>,
    input_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    output_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    context: &mut Context,
) -> Option<(
    ir::RemoteNodeId,
    Box<[ir::OperationId]>,
    Box<[ir::OperationId]>,
)> {
    let known_inputs = find_filled_hole(input_hole.clone());
    let known_outputs = find_filled_hole(output_hole.clone());
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // First try and infer the operation.  If this can't be done, we give up
    let operation = match operation_hole {
        Some(op) => op.clone(),
        None => match infer_operation(&known_inputs, &known_outputs, context) {
            Some(op) => op,
            None => { return None }
        }
    };

    match input_hole {
        None => unreachable!("empty inputs assumed to match with empty operation"),
        Some(input_vec) => for (index, input) in input_vec.iter().enumerate() {
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
    }

    let output_vec = match output_hole {
        Some(v) => v.into_vec(),
        None => vec![None; 5]
    };

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

    Some((
        remote_conversion(&operation, context),
        inputs.into_boxed_slice(),
        outputs.into_boxed_slice(),
    ))
}

pub fn explicate_encode_do(
    place_hole: &Hole<ir::Place>,
    operation_hole: &Hole<assembly_ast::RemoteNodeId>,
    inputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    outputs_hole: &Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = reject_hole(place_hole.clone());
    dbg!(&inputs_hole);
    dbg!(&outputs_hole);
    dbg!(&operation_hole);
    let result = explicate_operation(operation_hole, inputs_hole, outputs_hole, context);
    // a bit sloppy, but oh well
    let (operation, inputs, outputs) = match result {
        None => return None,
        Some(t) => t,
    };
    Some(ir::Node::EncodeDo {
        place,
        operation,
        inputs,
        outputs,
    })
}
