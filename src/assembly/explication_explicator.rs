use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalCpuFunctionId, ExternalGpuFunctionId, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::assembly::context::Context;
use crate::assembly::explication_util;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::stable_vec::StableVec;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

fn todo_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => todo!(),
    }
}

fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => panic!("Invalid hole location"),
    }
}

fn find_filled<T>(v: Vec<Hole<T>>) -> Vec<(usize, T)> {
    let mut result = Vec::new();
    for (index, hole) in v.into_iter().enumerate() {
        match hole {
            Some(value) => {
                result.push((index, value));
            }
            None => {}
        }
    }
    result
}

fn find_filled_hole<T>(h: Hole<Box<[Hole<T>]>>) -> Vec<(usize, T)>
where
    T: Clone,
{
    match h {
        Some(v) => find_filled(v.into_vec()),
        None => Vec::new(),
    }
}

pub fn explicate_allocate_temporary(
    place_hole: &Hole<ir::Place>,
    storage_type_hole: &Hole<assembly::ast::StorageTypeId>,
    operation_hole: &Hole<assembly::ast::RemoteNodeName>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = todo_hole(place_hole.as_ref());
    let storage_type = todo_hole(storage_type_hole.as_ref());
    let operation = todo_hole(operation_hole.as_ref());
    context.add_allocation(
        operation.node_name.clone(),
        context.location.node_name.clone(),
    );
    Some(ir::Node::AllocTemporary {
        place: place.clone(),
        storage_type: ffi::TypeId(context.loc_type_id(storage_type)),
        operation: explication_util::remote_conversion(operation, context),
    })
}

// try to infer the value operation associated with this schedule operation
fn infer_operation(
    known_inputs: &Vec<(usize, OperationId)>,
    known_outputs: &Vec<(usize, OperationId)>,
    context: &mut Context,
) -> Option<assembly::ast::OperationId> {
    // ignoring inputs for now due to being syntactically disallowed
    known_outputs.get(0).and_then(|output| {
        context
            .get_value_allocation(&context.location.funclet_name, &output.1)
            .map(|name| name.clone())
    })
}

fn get_node_arguments(node: &assembly::ast::Node, context: &Context) -> Vec<String> {
    fn collect_arguments(arguments: &Hole<Box<[Hole<OperationId>]>>) -> Vec<String> {
        reject_hole(arguments.as_ref())
            .to_vec()
            .into_iter()
            .map(|x| reject_hole(x))
            .collect()
    }
    match node {
        assembly::ast::Node::Constant { .. } => Vec::new(),
        assembly::ast::Node::ExtractResult { .. } => {
            panic!("Encode-do of an extract doesn't seem defined?")
        }
        assembly::ast::Node::CallExternalCpu {
            external_function_id,
            arguments,
        } => collect_arguments(arguments),
        assembly::ast::Node::CallExternalGpuCompute {
            external_function_id,
            dimensions,
            arguments,
        } => collect_arguments(arguments),
        assembly::ast::Node::CallValueFunction {
            function_id,
            arguments,
        } => collect_arguments(arguments),
        assembly::ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => vec![
            reject_hole(condition.as_ref()).clone(),
            reject_hole(true_case.as_ref()).clone(),
            reject_hole(false_case.as_ref()).clone(),
        ],
        _ => unreachable!("Value funclets shouldn't have {:?}", node),
    }
}

fn explicate_operation(
    operation_hole: &Hole<assembly::ast::RemoteNodeName>,
    input_hole: &Hole<Box<[Hole<assembly::ast::OperationId>]>>,
    output_hole: &Hole<Box<[Hole<assembly::ast::OperationId>]>>,
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
        Some(op) => op.node_name.clone(),
        None => match infer_operation(&known_inputs, &known_outputs, context) {
            Some(op) => op,
            None => todo!("Unfinished path"),
        },
    };
    let op = assembly::ast::RemoteNodeName {
        funclet_name: context.get_current_value_funclet().unwrap().clone(),
        node_name: operation,
    };

    let node = context
        .node_lookup(&op.funclet_name, &op.node_name)
        .unwrap();
    let node_arguments = get_node_arguments(&node.node, context);

    // lookup the allocation location and add the argument to the inputs
    fn add_to_inputs(
        op: &assembly::ast::RemoteNodeName,
        inputs: &mut Vec<usize>,
        argument: &String,
        context: &Context,
    ) {
        let alloc_name = context.get_current_schedule_allocation(argument).unwrap();
        inputs.push(context.node_id(alloc_name))
    }

    match input_hole {
        None => {
            for argument in node_arguments.iter() {
                add_to_inputs(&op, &mut inputs, argument, context);
            }
        }
        Some(input_vec) => {
            for (index, input) in input_vec.iter().enumerate() {
                match input {
                    Some(n) => inputs.push(context.node_id(&n)),
                    None => {
                        let argument = node_arguments.get(index).unwrap();
                        add_to_inputs(&op, &mut inputs, argument, context);
                    }
                }
            }
        }
    };

    let output_vec = match output_hole {
        Some(v) => v.clone().into_vec(),
        None => vec![None; 5],
    };

    for (index, output) in output_vec.iter().enumerate() {
        match output {
            Some(n) => outputs.push(context.node_id(&n)),
            None => {
                let node = context
                    .node_lookup(&op.funclet_name, &op.node_name)
                    .unwrap();
                match node.node {
                    assembly::ast::Node::Constant { .. } => {
                        match context.get_schedule_allocations(&op.funclet_name, &op.node_name) {
                            None => todo!("Unfinished path"), // failed to explicate on this pass
                            Some(alloc_map) => {
                                match alloc_map.get(&context.location.funclet_name) {
                                    None => todo!(),
                                    Some(alloc_loc) => outputs.push(context.node_id(&alloc_loc)),
                                }
                            }
                        }
                    }
                    _ => todo!("Unsupported node for explication {:?}", node),
                }
            }
        }
    }

    Some((
        explication_util::remote_conversion(&op, context),
        inputs.into_boxed_slice(),
        outputs.into_boxed_slice(),
    ))
}

pub fn explicate_encode_do(
    place_hole: &Hole<ir::Place>,
    operation_hole: &Hole<assembly::ast::RemoteNodeName>,
    inputs_hole: &Hole<Box<[Hole<assembly::ast::OperationId>]>>,
    outputs_hole: &Hole<Box<[Hole<assembly::ast::OperationId>]>>,
    context: &mut Context,
) -> Option<ir::Node> {
    let place = todo_hole(place_hole.clone());
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

pub fn explicate_return(
    return_values: &Hole<Vec<Hole<NodeId>>>,
    context: &mut Context,
) -> Option<ir::TailEdge> {
    // special case cause a return can be considered unique sorta?
    let mut result = Vec::new();

    let value_returns = match context
        .get_current_value_funclet()
        .as_ref()
        .and_then(|vf| context.get_tail_edge(vf).as_ref())
    {
        Some(assembly::ast::TailEdge::Return { return_values }) => {
            Some(return_values.as_ref().unwrap())
        }
        _ => None,
    };

    for (index, return_value) in reject_hole(return_values.as_ref()).iter().enumerate() {
        match return_value {
            Some(value) => result.push(context.node_id(value)),
            None => {
                let value = reject_hole(value_returns.unwrap().get(index).unwrap().as_ref());
                let alloc = context.get_current_schedule_allocation(value);
                result.push(context.node_id(todo_hole(alloc)));
            }
        }
    }

    Some(ir::TailEdge::Return {
        return_values: result.into_boxed_slice(),
    })
}
