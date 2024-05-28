use ron::value;

use crate::explication::context::{staticcontext, InState, OperationOutState, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::{explication, frontend, ir};

fn explicate_phi_node(
    index: usize,
    mut state: InState,
    context: &StaticContext,
) -> Option<OperationOutState> {
    let (value_spec, timeline_spec, _) =
        state.get_funclet_spec_triple(state.get_current_funclet_id(), context);
    let value_funclet_id = value_spec.funclet_id_opt.unwrap();
    let timeline_funclet_id = timeline_spec.funclet_id_opt.unwrap();
    match get_expect_box(&value_spec.input_tags, index) {
        Hole::Empty => {}
        Hole::Filled(value) => {
            let operation = Location {
                funclet_id: value_funclet_id,
                quot: value.quot.clone(),
            };
            state.add_value_operation(operation, context);
        }
    }
    match get_expect_box(&timeline_spec.input_tags, index) {
        Hole::Empty => {}
        Hole::Filled(timeline) => {
            let operation = Location {
                funclet_id: timeline_funclet_id,
                quot: timeline.quot.clone(),
            };
            state.add_timeline_operation(operation, context);
        }
    }
    let node = expir::Node::Phi {
        index: Hole::Filled(index),
    };
    state.next_node();
    match explicate_node(state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn build_do_operation<T>(
    operation: &Hole<expir::Quotient>,
    external_function_id: Option<&Hole<expir::ExternalFunctionId>>,
    node_builder: T,
    state: InState,
    context: &StaticContext,
) -> Option<OperationOutState>
where
    T: Fn(expir::Quotient, Option<expir::ExternalFunctionId>) -> expir::Node,
{
    fn read_external_id<'a>(
        function_class_id: &Hole<usize>,
        funclet_id: &FuncletId,
        context: &'a StaticContext,
    ) -> &'a expir::FunctionClass {
        context.program.function_classes.get_expect(
            function_class_id
                .as_ref()
                .opt()
                .expect(&format!(
                    "Hole found in {}",
                    context.debug_info.funclet(funclet_id)
                ))
                .clone(),
        )
    }
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    dbg!(&external_function_id);
    dbg!("");
    let operations_to_try = match operation {
        Hole::Filled(op) => vec![op.clone()],
        Hole::Empty => state
            .find_satisfied_operations(&value_funclet_id, context)
            .drain(..)
            .filter(|val_op| {
                match context.get_node(Location {
                    funclet_id: value_funclet_id,
                    quot: val_op.clone(),
                }) {
                    expir::Node::Constant {
                        value: _,
                        type_id: _,
                    } => external_function_id.is_none(),
                    expir::Node::CallFunctionClass {
                        function_id,
                        arguments: _,
                    } => match external_function_id {
                        // builtin case,
                        None => false,
                        // external hole case
                        Some(Hole::Empty) => true,
                        // external informational case
                        Some(Hole::Filled(external_id)) => {
                            dbg!(&function_id);
                            dbg!(&read_external_id(function_id, &value_funclet_id, context));
                            // check to see if this operation implements the external id
                            read_external_id(function_id, &value_funclet_id, context)
                                .external_function_ids
                                .contains(external_id)
                        }
                    },
                    _ => false,
                }
            })
            .collect(),
    };

    dbg!(&state.get_current_node(context));
    dbg!(&operations_to_try);

    for operation_to_try in operations_to_try {
        let mut new_state = state.clone();
        let location = Location {
            funclet_id: value_funclet_id,
            quot: operation_to_try.clone(),
        };
        dbg!(&context.debug_info.node(&location.funclet_id, location.node_id(context).unwrap()));
        new_state.add_value_operation(location, context);

        let external_id = match external_function_id {
            None => None,
            Some(Hole::Filled(id)) => Some(id.clone()),
            Some(Hole::Empty) => {
                match context.get_node(Location {
                    funclet_id: value_funclet_id,
                    quot: operation_to_try.clone(),
                }) {
                    expir::Node::CallFunctionClass {
                        function_id,
                        arguments: _,
                    } => Some(
                        read_external_id(function_id, &value_funclet_id, context)
                            .external_function_ids
                            .first()
                            .expect(&format!(
                                "No external function found to implement {}",
                                context
                                    .debug_info
                                    // now safe to unwrap wrt debugging
                                    .external_function(function_id.as_ref().opt().unwrap())
                            ))
                            .clone(),
                    ),
                    _ => unreachable!(),
                }
            }
        };
        let node = node_builder(operation_to_try, external_id);
        new_state.next_node();
        match explicate_node(new_state, context) {
            None => {}
            Some(mut out) => {
                out.add_node(node);
                return Some(out);
            }
        }
    }
    None
}

fn explicate_local_do_builtin(
    operation: &Hole<expir::Quotient>,
    inputs: &Hole<Box<[Hole<NodeId>]>>,
    outputs: &Hole<Box<[Hole<NodeId>]>>,
    state: InState,
    context: &StaticContext,
) -> Option<OperationOutState> {
    build_do_operation(
        operation,
        None,
        |operation_to_try, external| {
            assert!(external.is_none());
            expir::Node::LocalDoBuiltin {
                operation: Hole::Filled(operation_to_try),
                inputs: inputs.clone(),
                outputs: outputs.clone(),
            }
        },
        state,
        context,
    )
}

fn explicate_local_do_external(
    operation: &Hole<expir::Quotient>,
    inputs: &Hole<Box<[Hole<NodeId>]>>,
    outputs: &Hole<Box<[Hole<NodeId>]>>,
    external_function_id: &Hole<expir::ExternalFunctionId>,
    state: InState,
    context: &StaticContext,
) -> Option<OperationOutState> {
    build_do_operation(
        operation,
        Some(external_function_id),
        |operation_to_try, external_to_try| expir::Node::LocalDoExternal {
            operation: Hole::Filled(operation_to_try),
            external_function_id: Hole::Filled(external_to_try.unwrap()),
            inputs: inputs.clone(),
            outputs: outputs.clone(),
        },
        state,
        context,
    )
}

fn explicate_encode_do(
    operation: &Hole<expir::Quotient>,
    inputs: &Hole<Box<[Hole<NodeId>]>>,
    outputs: &Hole<Box<[Hole<NodeId>]>>,
    external_function_id: &Hole<expir::ExternalFunctionId>,
    encoder: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<OperationOutState> {
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    let operations_to_try = match operation {
        Hole::Filled(op) => vec![op.clone()],
        Hole::Empty => state
            .find_satisfied_operations(&value_funclet_id, context)
            .drain(..)
            .filter(|val_op| {
                match context.get_node(Location {
                    funclet_id: value_funclet_id,
                    quot: val_op.clone(),
                }) {
                    expir::Node::CallFunctionClass {
                        function_id: _,
                        arguments: _,
                    } => true,
                    _ => false,
                }
            })
            .collect(),
    };

    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    for operation_to_try in operations_to_try {
        let mut new_state = state.clone();
        let location = Location {
            funclet_id: value_funclet_id,
            quot: operation_to_try.clone(),
        };
        let base_node_id = location.node_id(context).unwrap();
        let node_info = context
            .get_node_type_information(&value_funclet_id, &location.node_id(context).unwrap());
        for offset in 0..node_info.output_types.len() {
            let offset_location = Location::new(value_funclet_id, base_node_id + offset + 1);
            new_state.add_value_operation(offset_location, context);
        }
        new_state.add_value_operation(location, context);
        let node = expir::Node::EncodeDoExternal {
            operation: Hole::Filled(operation_to_try),
            inputs: inputs.clone(),
            outputs: outputs.clone(),
            external_function_id: external_function_id.clone(),
            encoder: encoder.clone(),
        };
        new_state.next_node();
        match explicate_node(new_state, context) {
            None => {}
            Some(mut out) => {
                out.add_node(node);
                return Some(out);
            }
        }
    }
    None
}

pub fn explicate_node(state: InState, context: &StaticContext) -> Option<OperationOutState> {
    let debug_funclet = context.debug_info.funclet(&state.get_current_funclet_id());
    if state.is_end_of_funclet(context) {
        explicate_tail_edge(&state, context)
    } else {
        let current_node = state.get_current_node(context);
        match current_node {
            Hole::Empty => {
                let mut new_state = state.clone();
                new_state.add_explication_hole();
                explicate_node(new_state, context);
                todo!()
            }
            Hole::Filled(expir::Node::Phi { index }) => explicate_phi_node(
                index
                    .as_ref()
                    .opt()
                    .expect(&format!(
                        "Cannot have a hole for index in Phi node {}",
                        context.debug_info.node_expir(
                            state.get_current_funclet_id(),
                            current_node.as_ref().opt().unwrap()
                        )
                    ))
                    .clone(),
                state,
                context,
            ),
            Hole::Filled(expir::Node::LocalDoBuiltin {
                operation,
                inputs,
                outputs,
            }) => explicate_local_do_builtin(operation, inputs, outputs, state, context),
            Hole::Filled(expir::Node::LocalDoExternal {
                operation,
                inputs,
                outputs,
                external_function_id,
            }) => explicate_local_do_external(
                operation,
                inputs,
                outputs,
                external_function_id,
                state,
                context,
            ),
            Hole::Filled(expir::Node::EncodeDoExternal {
                encoder,
                operation,
                inputs,
                outputs,
                external_function_id,
            }) => explicate_encode_do(
                operation,
                inputs,
                outputs,
                external_function_id,
                encoder,
                state,
                context,
            ),
            Hole::Filled(node) => {
                let mut new_state = state.clone();
                new_state.next_node();
                match explicate_node(new_state, context) {
                    None => None,
                    Some(mut out) => {
                        out.add_node(node.clone());
                        Some(out)
                    }
                }
            }
        }
    }
}

pub fn explicate_tail_edge(state: &InState, context: &StaticContext) -> Option<OperationOutState> {
    match state.get_current_tail_edge(context) {
        Hole::Filled(tail_edge) => {
            let error = format!("Unimplemented hole in tail edge {:?}", tail_edge);
            let mut result = OperationOutState::new();
            result.set_tail_edge(tail_edge.clone());
            Some(result)
        }
        Hole::Empty => {
            todo!()
        }
    }
}
