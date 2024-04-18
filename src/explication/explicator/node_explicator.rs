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

    match node_type {
        // we need to add this reference to our available allocations
        // TODO: check the flow to see if we can actually do this
        ir::Type::Ref {
            storage_type,
            storage_place,
            buffer_flags,
        } => {
            new_state.add_storage_node(node_id, node_type.clone(), context);
        }
        _ => {}
    }

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

fn explicate_local_do_builtin(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
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
    let operation = expir_operation.as_ref().opt().expect(&error).clone();
    let node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&error).iter() {
        let input_open = input.as_ref().opt().expect(&error).clone();
        inputs.push(input_open);
    }
    // note that offset refers to the requirement that extracts be in sequence after a do
    for (offset, output) in expir_outputs
        .as_ref()
        .opt()
        .expect(&error)
        .iter()
        .enumerate()
    {
        let output_open = output.as_ref().opt().expect(&error).clone();
        let value_location = Location::new(
            state
                .get_funclet_spec(
                    state.get_current_funclet_id(),
                    &SpecLanguage::Value,
                    context,
                )
                .funclet_id_opt
                .unwrap(),
            node_id,
        );
        new_state.set_instantiation(
            output_open,
            LocationTriple::new_value(value_location),
            context,
        );
        outputs.push(output_open);
    }
    let node = ir::Node::LocalDoBuiltin {
        operation,
        inputs: inputs.into_boxed_slice(),
        outputs: outputs.into_boxed_slice(),
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

fn explicate_local_do_external(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_funclet_id: &Hole<expir::ExternalFunctionId>,
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
    let operation = expir_operation.as_ref().opt().expect(&error).clone();
    let node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&error).iter() {
        let input_open = input.as_ref().opt().expect(&error).clone();
        inputs.push(input_open);
    }
    // note that offset refers to the requirement that extracts be in sequence after a do
    for (offset, output) in expir_outputs
        .as_ref()
        .opt()
        .expect(&error)
        .iter()
        .enumerate()
    {
        let output_open = output.as_ref().opt().expect(&error).clone();
        let value_location = Location::new(
            state
                .get_funclet_spec(
                    state.get_current_funclet_id(),
                    &SpecLanguage::Value,
                    context,
                )
                .funclet_id_opt
                .unwrap(),
            node_id + offset + 1,
        );
        new_state.set_instantiation(
            output_open,
            LocationTriple::new_value(value_location),
            context,
        );
        outputs.push(output_open);
    }
    let node = ir::Node::LocalDoExternal {
        operation,
        external_function_id: expir_external_funclet_id
            .as_ref()
            .opt()
            .expect(&error)
            .clone(),
        inputs: inputs.into_boxed_slice(),
        outputs: outputs.into_boxed_slice(),
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

fn explicate_encode_do(
    expir_encoder: &Hole<usize>,
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_funclet_id: &Hole<expir::ExternalFunctionId>,
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
    let operation = expir_operation.as_ref().opt().expect(&error).clone();
    let node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for local do builtin {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for input in expir_inputs.as_ref().opt().expect(&error).iter() {
        let input_open = input.as_ref().opt().expect(&error).clone();
        inputs.push(input_open);
    }
    for (offset, output) in expir_outputs
        .as_ref()
        .opt()
        .expect(&error)
        .iter()
        .enumerate()
    {
        let output_open = output.as_ref().opt().expect(&error).clone();
        let value_location = Location::new(
            state
                .get_funclet_spec(
                    state.get_current_funclet_id(),
                    &SpecLanguage::Value,
                    context,
                )
                .funclet_id_opt
                .unwrap(),
            node_id + offset + 1,
        );
        new_state.set_instantiation(
            output_open,
            LocationTriple::new_value(value_location),
            context,
        );
        outputs.push(output_open);
    }
    let external_function_id = expir_external_funclet_id
        .as_ref()
        .opt()
        .expect(&error)
        .clone();
    let node = ir::Node::EncodeDoExternal {
        encoder,
        operation,
        inputs: inputs.into_boxed_slice(),
        outputs: outputs.into_boxed_slice(),
        external_function_id,
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
    let mut new_state = state.clone();
    let source = expir_source.as_ref().opt().expect(&error).clone();
    let destination = expir_destination.as_ref().opt().expect(&error).clone();
    let storage_type = expir_storage_type.as_ref().opt().expect(&error).clone();
    // assume that the typechecker will find a misaligned storage type
    let info = state.get_node_information(destination, context);
    new_state.set_instantiation(destination, info.instantiation.clone(), context);
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

    let info = state.get_node_information(source, context);
    let schedule_node = state.get_current_node_id().unwrap();

    new_state.add_storage_node(schedule_node, info.typ.clone(), context);
    new_state.set_instantiation(schedule_node, info.instantiation.clone(), context);

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

    let info = state.get_node_information(source, context);
    let schedule_node = state.get_current_node_id().unwrap();

    new_state.add_storage_node(
        schedule_node,
        expir::Type::NativeValue { storage_type },
        context,
    );
    new_state.set_instantiation(schedule_node, info.instantiation.clone(), context);
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
    let info = state.get_node_information(input, context);
    new_state.set_instantiation(output, info.instantiation.clone(), context);
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
    let info = state.get_node_information(input, context);
    new_state.set_instantiation(output, info.instantiation.clone(), context);
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
    let timeline_spec = state.get_funclet_spec(
        state.get_current_funclet_id(),
        &SpecLanguage::Timeline,
        context,
    );
    let timeline_loc = LocationTriple::new_timeline(Location {
        funclet_id: timeline_spec.funclet_id_opt.unwrap(),
        quot: event.clone(),
    });

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
    dbg!(&state.get_managed_by_timeline(encoder, context));
    let event = expir_event.as_ref().opt().expect(&error).clone();
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
