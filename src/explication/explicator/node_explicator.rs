use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::{explication, frontend, ir};

use crate::rust_wgpu_backend::ffi;

use super::force_lower_node;
use super::tail_edge_explicator;

fn explicate_phi_node(
    index: usize,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let funclet_id = state.get_current_funclet_id();
    let node_id = state.get_current_node_id().unwrap();
    let node_type = context
        .get_type(
            state
                .get_current_funclet(context)
                .input_types
                .get(index)
                .unwrap(),
        )
        .clone();

    new_state.add_storage_node(node_id, node_type.clone(), context);
    let location =
        LocationTriple::new_triple_mapped(spec_input, funclet_id, node_id, &state, context);

    new_state.set_instantiation(node_id, location, context);
    let node = ir::Node::Phi { index };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_allocate_temporary(
    expir_place: &Hole<expir::Place>,
    expir_storage_type: &Hole<ffi::TypeId>,
    expir_buffer_flags: &Hole<ir::BufferFlags>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let storage_place = expir_place.as_ref().opt().expect(&error).clone();
    let storage_type = expir_storage_type.as_ref().opt().expect(&error).clone();
    let buffer_flags = expir_buffer_flags.as_ref().opt().expect(&error).clone();
    let mut new_state = state.clone();
    new_state.add_storage_node(
        state.get_current_node_id().unwrap(),
        expir::Type::Ref {
            storage_type,
            storage_place,
            buffer_flags,
        },
        context,
    );
    new_state.next_node();
    let node = ir::Node::AllocTemporary {
        place: storage_place,
        storage_type,
        buffer_flags,
    };
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn enumerate_output_type_attempts(
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    value_funclet_id: &FuncletId,
    value_node_id: &NodeId,
    state: &InState,
    context: &StaticContext,
) -> Vec<Vec<usize>> {
    let value_error_text = format!(
        " with value node {} {}",
        context
            .debug_info
            .node(value_funclet_id, value_node_id.clone()),
        state.get_node_error(context)
    );
    let value_error = |error: &str| format!("{} : {}", error, value_error_text);
    let value_spec_node_types =
        context.get_node_type_information(&value_funclet_id, &value_node_id);

    let unpacked_outputs = match expir_outputs.as_ref().opt() {
        Some(outputs) => outputs.iter().map(|o| o.clone().opt()).collect(),
        None => {
            let mut expanded_outputs = Vec::new();
            for _ in 0..value_spec_node_types.output_types.len() {
                expanded_outputs.push(None);
            };
            expanded_outputs
        }
    };

    let mut output_attempts = Vec::new();
    output_attempts.push(Vec::new());
    for (offset, output) in unpacked_outputs
        .iter()
        .enumerate()
    {
        match output {
            Some(open_output) => {
                for output_attempt in output_attempts.iter_mut() {
                    output_attempt.push(open_output.clone());
                }
            }
            None => {
                let storage_type = match context.get_type(
                    value_spec_node_types
                        .output_types
                        .get(offset)
                        .expect(&value_error(&format!("Missing argument index {}", offset))),
                ) {
                    ir::Type::NativeValue { storage_type } => storage_type.clone(),
                    typ => panic!(
                        "{}",
                        value_error(&format!("Cannot have type {:?} in a value funclet", typ))
                    ),
                };
                let target_type = expir::Type::Ref {
                    storage_type,
                    storage_place: expir::Place::Local,
                    buffer_flags: expir::BufferFlags::new(),
                };
                let mut new_output_attempts = Vec::new();
                for output_to_try in state.find_all_storage_nodes(&target_type, context).iter() {
                    if output_to_try.funclet_id == state.get_current_funclet_id() {
                        for current_output in output_attempts.iter() {
                            let mut new_output = current_output.clone();
                            new_output.push(output_to_try.node_id().unwrap());
                            new_output_attempts.push(new_output)
                        }
                    }
                }
                output_attempts = new_output_attempts;
            }
        }
    }
    output_attempts
}

fn explicate_local_do_builtin(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    let operation = expir_operation.as_ref().opt().expect(&state.hole_error(context)).clone();
    let value_node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };

    let node_location = Location::new(value_funclet_id, value_node_id);
    let mut inputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&state.hole_error(context)).iter() {
        let input_open = input.as_ref().opt().expect(&state.hole_error(context)).clone();
        inputs.push(input_open);
    }

    let mut output_attempts = enumerate_output_type_attempts(
        expir_outputs,
        &value_funclet_id,
        &value_node_id,
        &state,
        context,
    );

    for outputs in output_attempts.drain(..) {
        let mut new_state = state.clone();
        assert!(outputs.len() == 1, "Local do builtin only supported on non-tuple output {}", state.get_node_error(context));
        let value_location = Location::new(value_funclet_id, value_node_id);
        new_state.set_instantiation(
            outputs.first().unwrap().clone(),
            LocationTriple::new_value(value_location),
            context,
        );
        let node = ir::Node::LocalDoBuiltin {
            operation,
            inputs: inputs.clone().into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
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

fn explicate_local_do_external(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_function_id: &Hole<expir::ExternalFunctionId>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    let operation = expir_operation.as_ref().opt().expect(&state.hole_error(context)).clone();
    let value_node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };

    let node_location = Location::new(value_funclet_id, value_node_id);
    let mut inputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&state.hole_error(context)).iter() {
        let input_open = input.as_ref().opt().expect(&state.hole_error(context)).clone();
        inputs.push(input_open);
    }

    let mut output_attempts = enumerate_output_type_attempts(
        expir_outputs,
        &value_funclet_id,
        &value_node_id,
        &state,
        context,
    );

    let external_function_id = expir_external_function_id.as_ref().opt().expect(&state.hole_error(context)).clone();

    for outputs in output_attempts.drain(..) {
        let mut new_state = state.clone();
        for (offset, output) in outputs.iter().enumerate() {
            let value_location = Location::new(value_funclet_id, value_node_id + offset + 1);
            new_state.set_instantiation(
                output.clone(),
                LocationTriple::new_value(value_location),
                context,
            );
        }
        let node = ir::Node::LocalDoExternal {
            operation,
            inputs: inputs.clone().into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            external_function_id,
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

fn explicate_encode_do(
    expir_encoder: &Hole<usize>,
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_function_id: &Hole<expir::ExternalFunctionId>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    let operation = expir_operation.as_ref().opt().expect(&state.hole_error(context)).clone();
    let value_node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };

    let node_location = Location::new(value_funclet_id, value_node_id);
    let mut inputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&state.hole_error(context)).iter() {
        let input_open = input.as_ref().opt().expect(&state.hole_error(context)).clone();
        inputs.push(input_open);
    }

    let mut output_attempts = enumerate_output_type_attempts(
        expir_outputs,
        &value_funclet_id,
        &value_node_id,
        &state,
        context,
    );

    let external_function_id = expir_external_function_id.as_ref().opt().expect(&state.hole_error(context)).clone();
    let encoder = expir_encoder.as_ref().opt().expect(&state.hole_error(context)).clone();

    for outputs in output_attempts.drain(..) {
        let mut new_state = state.clone();
        for (offset, output) in outputs.iter().enumerate() {
            let value_location = Location::new(value_funclet_id, value_node_id + offset + 1);
            new_state.set_instantiation(
                output.clone(),
                LocationTriple::new_value(value_location),
                context,
            );
        }
        let node = ir::Node::EncodeDoExternal {
            encoder,
            operation,
            inputs: inputs.clone().into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            external_function_id,
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

fn explicate_write_ref(
    expir_source: &Hole<usize>,
    expir_destination: &Hole<usize>,
    expir_storage_type: &Hole<ffi::TypeId>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );

    let source = expir_source.as_ref().opt().expect(&error).clone();
    let storage_type = expir_storage_type.as_ref().opt().expect(&error).clone();

    fn try_destinations(
        source: usize,
        destination: usize,
        storage_type: ffi::TypeId,
        state: InState,
        context: &StaticContext,
    ) -> Option<FuncletOutState> {
        let mut new_state = state.clone();
        // assume that the typechecker will find a misaligned storage type
        let info = state.get_node_information(&source, context);
        new_state.set_instantiation(
            destination,
            info.instantiation.clone().expect(&format!(
                "Missing instantiation for node {}",
                context
                    .debug_info
                    .node(&state.get_current_funclet_id(), source)
            )),
            context,
        );
        let node = ir::Node::WriteRef {
            storage_type,
            source,
            destination,
        };
        new_state.next_node();
        match explicate_node(new_state, context) {
            None => None,
            Some(mut out) => {
                out.add_node(node);
                Some(out)
            }
        }
    }

    match expir_destination.as_ref().opt() {
        Some(destination) => {
            try_destinations(source, destination.clone(), storage_type, state, context)
        }
        None => {
            todo!()
        }
    }
}

fn explicate_borrow_ref(
    expir_source: &Hole<usize>,
    expir_storage_type: &Hole<ffi::TypeId>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let source = expir_source.as_ref().opt().expect(&error).clone();
    let storage_type = expir_storage_type.as_ref().opt().expect(&error).clone();

    let info = state.get_node_information(&source, context);
    let schedule_node = state.get_current_node_id().unwrap();

    new_state.add_storage_node(schedule_node, info.typ.clone(), context);
    new_state.set_instantiation(
        schedule_node,
        info.instantiation.clone().expect(&format!(
            "Missing instantiation for node {}",
            context
                .debug_info
                .node(&state.get_current_funclet_id(), source)
        )),
        context,
    );

    let node = ir::Node::BorrowRef {
        storage_type,
        source,
    };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_read_ref(
    expir_source: &Hole<usize>,
    expir_storage_type: &Hole<ffi::TypeId>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let source = expir_source.as_ref().opt().expect(&error).clone();
    let storage_type = expir_storage_type.as_ref().opt().expect(&error).clone();

    let info = state.get_node_information(&source, context);
    let schedule_node = state.get_current_node_id().unwrap();

    new_state.add_storage_node(
        schedule_node,
        expir::Type::NativeValue { storage_type },
        context,
    );
    new_state.set_instantiation(
        schedule_node,
        info.instantiation.clone().expect(&format!(
            "Missing instantiation for node {}",
            context
                .debug_info
                .node(&state.get_current_funclet_id(), source)
        )),
        context,
    );
    let node = ir::Node::ReadRef {
        storage_type,
        source,
    };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_local_copy(
    expir_input: &Hole<usize>,
    expir_output: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let input = expir_input.as_ref().opt().expect(&error).clone();
    let output = expir_output.as_ref().opt().expect(&error).clone();
    let info = state.get_node_information(&input, context);
    new_state.set_instantiation(
        output,
        info.instantiation.clone().expect(&format!(
            "Missing instantiation for node {}",
            context
                .debug_info
                .node(&state.get_current_funclet_id(), input)
        )),
        context,
    );
    let node = ir::Node::LocalCopy { input, output };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_encode_copy(
    expir_input: &Hole<usize>,
    expir_output: &Hole<usize>,
    expir_encoder: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let input = expir_input.as_ref().opt().expect(&error).clone();
    let output = expir_output.as_ref().opt().expect(&error).clone();
    let encoder = expir_encoder.as_ref().opt().expect(&error).clone();
    let info = state.get_node_information(&input, context);
    new_state.set_instantiation(
        output,
        info.instantiation.clone().expect(&format!(
            "Missing instantiation for node {}",
            context
                .debug_info
                .node(&state.get_current_funclet_id(), input)
        )),
        context,
    );
    let node = ir::Node::EncodeCopy {
        encoder,
        input,
        output,
    };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_begin_encoding(
    expir_place: &Hole<expir::Place>,
    expir_event: &Hole<expir::Quotient>,
    expir_encoded: &Hole<Box<[Hole<NodeId>]>>,
    expir_fences: &Hole<Box<[Hole<NodeId>]>>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let place = expir_place.as_ref().opt().expect(&error).clone();
    let event = expir_event.as_ref().opt().expect(&error).clone();
    let timeline_loc = state.get_triple_for_spec(
        state.get_current_funclet_id(),
        &SpecLanguage::Timeline,
        event.clone(),
        context,
    );

    let mut encoded = Vec::new();
    for node in expir_encoded.as_ref().opt().expect(&error).iter() {
        let schedule_node = node.as_ref().opt().expect(&error);
        new_state.set_instantiation(schedule_node.clone(), timeline_loc.clone(), context);
        new_state.set_timeline_manager(
            schedule_node,
            state.get_current_node_id().unwrap(),
            context,
        );
        encoded.push(schedule_node.clone());
    }
    let fences = expir_fences
        .as_ref()
        .opt()
        .expect(&error)
        .iter()
        .map(|e| e.as_ref().opt().expect(&error).clone())
        .collect();

    let node = ir::Node::BeginEncoding {
        place,
        event,
        encoded: encoded.into_boxed_slice(),
        fences,
    };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_submit(
    expir_encoder: &Hole<NodeId>,
    expir_event: &Hole<expir::Quotient>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let encoder = expir_encoder.as_ref().opt().expect(&error).clone();
    let event = expir_event.as_ref().opt().expect(&error).clone();

    let timeline_loc = state.get_triple_for_spec(
        state.get_current_funclet_id(),
        &SpecLanguage::Timeline,
        event.clone(),
        context,
    );
    for schedule_node in state.get_managed_by_timeline(encoder, context).iter() {
        new_state.set_instantiation(schedule_node.clone(), timeline_loc.clone(), context);
        new_state.set_timeline_manager(schedule_node, state.get_current_node_id().unwrap(), context)
    }

    let node = ir::Node::Submit { encoder, event };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

fn explicate_sync_fence(
    expir_fence: &Hole<NodeId>,
    expir_event: &Hole<expir::Quotient>,
    state: InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    let error = format!(
        "TODO Hole in node {}",
        context.debug_info.node_expir(
            state.get_current_funclet_id(),
            state
                .get_current_node(context)
                .as_ref()
                .opt()
                .expect("Unreachable")
        )
    );
    let mut new_state = state.clone();
    let fence = expir_fence.as_ref().opt().expect(&error).clone();
    let event = expir_event.as_ref().opt().expect(&error).clone();

    let timeline_loc = state.get_triple_for_spec(
        state.get_current_funclet_id(),
        &SpecLanguage::Timeline,
        event.clone(),
        context,
    );
    for schedule_node in state.get_managed_by_timeline(fence, context).iter() {
        new_state.set_instantiation(schedule_node.clone(), timeline_loc.clone(), context);
        new_state.clear_timeline_manager(schedule_node, context)
    }

    let node = ir::Node::SyncFence { fence, event };
    new_state.next_node();
    match explicate_node(new_state, context) {
        None => None,
        Some(mut out) => {
            out.add_node(node);
            Some(out)
        }
    }
}

// initially setup a node that hasn't yet been read
// distinct from explication in that we have no request to fulfill
// panics if no node can be found during any step of the recursion
pub fn explicate_node(state: InState, context: &StaticContext) -> Option<FuncletOutState> {
    let debug_funclet = context.debug_info.funclet(&state.get_current_funclet_id());
    if state.is_end_of_funclet(context) {
        tail_edge_explicator::explicate_tail_edge(&state, context)
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
            Hole::Filled(expir::Node::AllocTemporary {
                place,
                storage_type,
                buffer_flags,
            }) => explicate_allocate_temporary(place, storage_type, buffer_flags, state, context),
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
            Hole::Filled(expir::Node::WriteRef {
                storage_type,
                destination,
                source,
            }) => explicate_write_ref(source, destination, storage_type, state, context),
            Hole::Filled(expir::Node::BorrowRef {
                source,
                storage_type,
            }) => explicate_borrow_ref(source, storage_type, state, context),
            Hole::Filled(expir::Node::ReadRef {
                source,
                storage_type,
            }) => explicate_read_ref(source, storage_type, state, context),
            Hole::Filled(expir::Node::LocalCopy { input, output }) => {
                explicate_local_copy(input, output, state, context)
            }
            Hole::Filled(expir::Node::EncodeCopy {
                input,
                output,
                encoder,
            }) => explicate_encode_copy(input, output, encoder, state, context),
            Hole::Filled(expir::Node::EncodeDoExternal {
                encoder,
                operation,
                inputs,
                outputs,
                external_function_id,
            }) => explicate_encode_do(
                encoder,
                operation,
                inputs,
                outputs,
                external_function_id,
                state,
                context,
            ),
            Hole::Filled(expir::Node::BeginEncoding {
                place,
                event,
                encoded,
                fences,
            }) => explicate_begin_encoding(place, event, encoded, fences, state, context),
            Hole::Filled(expir::Node::Submit { encoder, event }) => {
                explicate_submit(encoder, event, state, context)
            }
            Hole::Filled(expir::Node::SyncFence { fence, event }) => {
                explicate_sync_fence(fence, event, state, context)
            }
            Hole::Filled(node) => {
                let mut new_state = state.clone();
                new_state.next_node();
                match explicate_node(new_state, context) {
                    None => None,
                    Some(mut out) => {
                        out.add_node(force_lower_node(node, &debug_funclet));
                        Some(out)
                    }
                }
            }
        }
    }
}
