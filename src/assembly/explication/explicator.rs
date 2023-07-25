use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId, TypeId,
};
use crate::assembly::explication::context::AllocationInfo;
use crate::assembly::explication::context::Context;
use crate::assembly::explication::util;
use crate::assembly::explication::util::{reject_hole, todo_hole};
use crate::assembly::parser;
use crate::ir::ffi;
use crate::stable_vec::StableVec;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

// try to infer the value operation associated with this schedule operation
fn infer_operation(
    known_inputs: &Vec<(usize, NodeId)>,
    known_outputs: &Vec<(usize, NodeId)>,
    context: &mut Context,
) -> Option<assembly::ast::RemoteNodeId> {
    // ignoring inputs for now due to being syntactically disallowed
    known_outputs.get(0).and_then(|output| {
        context
            .get_value_allocation(context.location_funclet(), &output.1)
            .map(|name| name.clone())
    })
}

fn get_node_arguments(node: &assembly::ast::Node, context: &Context) -> Vec<NodeId> {
    fn collect_arguments(arguments: &Hole<Vec<Hole<NodeId>>>) -> Vec<NodeId> {
        reject_hole(arguments.as_ref())
            .iter()
            .map(|x| reject_hole(x.as_ref()).clone())
            .collect()
    }
    match node {
        assembly::ast::Node::Phi { .. } => {
            panic!("Encode-do of a phi node isn't defined")
        }
        assembly::ast::Node::Constant { .. } => Vec::new(),
        assembly::ast::Node::ExtractResult { .. } => {
            panic!("Encode-do of an extraction isn't defined")
        }
        assembly::ast::Node::CallFunctionClass {
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
    operation_hole: &Hole<assembly::ast::RemoteNodeId>,
    input_hole: &Hole<Vec<Hole<assembly::ast::NodeId>>>,
    output_hole: &Hole<Vec<Hole<assembly::ast::NodeId>>>,
    context: &mut Context,
) -> (ast::Quotient, Vec<NodeId>, Vec<NodeId>) {
    let known_inputs = util::find_filled_hole(input_hole.clone());
    let known_outputs = util::find_filled_hole(output_hole.clone());
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // First try and infer the operation.  If this can't be done, we give up
    let operation = match operation_hole {
        Some(op) => reject_hole(op.node.as_ref()).clone(),
        None => match infer_operation(&known_inputs, &known_outputs, context) {
            Some(op) => op.node.unwrap(),
            None => todo!("Unfinished path"),
        },
    };
    let remote = RemoteNodeId {
        funclet: Some(context.get_current_value_funclet().unwrap().clone()),
        node: Some(operation),
    };
    let qop = ast::Quotient::Node(Some(remote.clone()));
    let op = remote.clone();

    let node = context
        .get_node(&remote.funclet.unwrap(), &remote.node.unwrap());
    let node_arguments = get_node_arguments(&node, context);

    // lookup the allocation location and add the argument to the inputs
    fn add_to_inputs(
        op: &ast::Quotient,
        inputs: &mut Vec<NodeId>,
        argument: &NodeId,
        context: &Context,
    ) {
        let alloc_name = context.get_current_schedule_allocation(argument).unwrap();
        inputs.push(alloc_name.clone())
    }

    match input_hole {
        None => {
            for argument in node_arguments.iter() {
                add_to_inputs(&qop, &mut inputs, argument, context);
            }
        }
        Some(input_vec) => {
            for (index, input) in input_vec.iter().enumerate() {
                match input {
                    Some(n) => inputs.push(n.clone()),
                    None => {
                        let argument = node_arguments.get(index).unwrap();
                        add_to_inputs(&qop, &mut inputs, argument, context);
                    }
                }
            }
        }
    };

    let output_vec = match output_hole {
        Some(v) => v.clone(),
        None => vec![None; 5],
    };

    for (index, output) in output_vec.iter().enumerate() {
        match output {
            Some(n) => outputs.push(n.clone()),
            None => {
                let node = context
                    .get_node(&op.funclet.as_ref().unwrap(), &op.node.as_ref().unwrap());
                match node {
                    assembly::ast::Node::Constant { .. } => {
                        match context.get_schedule_allocations(
                            &op.funclet.as_ref().unwrap(),
                            &op.node.as_ref().unwrap(),
                        ) {
                            None => todo!("Unfinished path"), // failed to explicate on this pass
                            Some(alloc_map) => {
                                match alloc_map.get(&AllocationInfo {
                                    schedule_funclet: context.location_funclet().clone(),
                                    place: ir::Place::Gpu, // make actually correct
                                }) {
                                    None => todo!(),
                                    Some(alloc_loc) => outputs.push(alloc_loc.clone()),
                                }
                            }
                        }
                    }
                    _ => todo!("Unsupported node for explication {:?}", node),
                }
            }
        }
    }

    (qop, inputs, outputs)
}

// pub fn explicate_allocate_temporary(
//     place_hole: &Hole<ir::Place>,
//     storage_type_hole: &Hole<assembly::ast::StorageTypeId>,
//     operation_hole: &Hole<assembly::ast::RemoteNodeId>,
//     context: &mut Context,
// ) {
//     let place = todo_hole(place_hole.as_ref());
//     let storage_type = todo_hole(storage_type_hole.as_ref());
//     let operation = todo_hole(operation_hole.as_ref());
//     context.add_allocation(
//         todo_hole(operation.node.as_ref()).clone(),
//         context.location.node_name.clone(),
//     );
// }

pub fn explicate_local_do_builtin(
    operation_hole: &Hole<assembly::ast::RemoteNodeId>,
    inputs_hole: &Hole<Vec<Hole<assembly::ast::NodeId>>>,
    outputs_hole: &Hole<Vec<Hole<assembly::ast::NodeId>>>,
    context: &mut Context,
) -> ast::Node {
    let (operation, inputs, outputs) =
        explicate_operation(operation_hole, inputs_hole, outputs_hole, context);
    ast::Node::LocalDoBuiltin {
        operation: Some(operation),
        inputs: Some(inputs.iter().map(|s| Some(s.clone())).collect()),
        outputs: Some(outputs.iter().map(|s| Some(s.clone())).collect()),
    }
}

pub fn explicate_return(return_values: &Hole<Vec<Hole<NodeId>>>, context: &mut Context) {
    // special case cause a return can be considered unique sorta?
    let mut result = Vec::new();

    let value_returns = match context
        .get_current_value_funclet()
        .and_then(|vf| context.get_tail_edge(vf))
    {
        Some(assembly::ast::TailEdge::Return { return_values }) => {
            Some(return_values.as_ref().unwrap())
        }
        _ => None,
    };

    for (index, return_value) in reject_hole(return_values.as_ref()).iter().enumerate() {
        match return_value {
            Some(value) => result.push(value.clone()),
            None => {
                let value = reject_hole(value_returns.unwrap().get(index).unwrap().as_ref());
                let alloc = context.get_current_schedule_allocation(value);
                result.push(todo_hole(alloc).clone());
            }
        }
    }
}
