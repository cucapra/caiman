use std::any::Any;

use itertools::Itertools;

use crate::explication::context::{InState, OperationOutState, StaticContext, StorageOutState};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::{BufferFlags, Place};
use crate::{explication, frontend, ir};

use crate::rust_wgpu_backend::ffi;

use super::force_lower_node;
use super::operation_explicator;

fn explicate_phi_node(
    index: usize,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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

    new_state.add_storage_node(node_id, Hole::Filled(node_type.clone()), context);
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
    mut state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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
    let buffer_flags = expir_buffer_flags.as_ref().opt().expect(&error).clone();
    match (expir_place, expir_storage_type) {
        (Hole::Filled(storage_place), Hole::Filled(storage_type)) => {
            state.add_storage_node(
                state.get_current_node_id().unwrap(),
                Hole::Filled(expir::Type::Ref {
                    storage_place: storage_place.clone(),
                    storage_type: storage_type.clone(),
                    buffer_flags,
                }),
                context,
            );

            let node = ir::Node::AllocTemporary {
                place: storage_place.clone(),
                storage_type: storage_type.clone(),
                buffer_flags,
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
        _ => {
            state.add_storage_node(state.get_current_node_id().unwrap(), Hole::Empty, context);

            let node_id = state.get_current_node_id().unwrap();
            let funclet_id = state.get_current_funclet_id();

            state.next_node();
            match explicate_node(state, context) {
                None => None,
                Some(mut out) => {
                    match out.take_to_fill(&node_id) {
                        None => {
                            // TODO: is this the correct default behavior?
                            Some(out)
                        }
                        Some(node) => {
                            out.add_node(node);
                            Some(out)
                        }
                    }
                }
            }
        }
    }
}

// Enumerates all the ways we can attempt to fill the "current" arguments
// This is intended to work for both inputs and outputs
// Note that the argument "instantiation_bounds" defines inputs vs outputs
// this is expected to be "none" for outputs

fn enumerate_fill_attempts(
    expir_current: &Hole<Box<[Hole<NodeId>]>>,
    expected_types: &Vec<expir::Type>,
    instantiation_bounds: Option<Vec<Location>>,
    error_context: &str,
    state: &InState,
    context: &StaticContext,
) -> Vec<Vec<usize>> {
    let error = |error: &str| format!("{} : {}", error, error_context);

    let unpacked = match expir_current.as_ref().opt() {
        Some(givens) => givens.iter().map(|o| o.clone().opt()).collect(),
        None => {
            let mut expanded = Vec::new();
            for _ in 0..expected_types.len() {
                expanded.push(None);
            }
            expanded
        }
    };

    let mut attempts = Vec::new();
    attempts.push(Vec::new());
    for (offset, attempt) in unpacked.iter().enumerate() {
        match attempt {
            Some(open_attempt) => {
                for attempt in attempts.iter_mut() {
                    attempt.push(open_attempt.clone());
                }
            }
            None => {
                let target_type = expected_types
                    .get(offset)
                    .expect(&error(&format!("Missing argument index {}", offset)));
                let attempts_to_try = match &instantiation_bounds {
                    Some(bounds) => state.find_matching_instantiations(
                        &LocationTriple::new_value(bounds.get(offset).unwrap().clone()),
                        &target_type,
                        context,
                    ),
                    None => state.find_all_storage_nodes(&target_type, context),
                };

                // adds each new attempt to _each_ vector we've built so far
                // equivalent to building the matrix of attempts to make
                // will combinatorically explode, in other words
                let mut new_attempts = Vec::new();
                for attempt_to_try in attempts_to_try.iter() {
                    if attempt_to_try.funclet_id == state.get_current_funclet_id() {
                        for current_output in attempts.iter() {
                            let mut new_attempt = current_output.clone();
                            new_attempt.push(attempt_to_try.node_id(context).unwrap());
                            new_attempts.push(new_attempt)
                        }
                    }
                }
                // note that if we didn't add anything to new_attempts
                // then this must be an invalid "path"
                attempts = new_attempts;
            }
        }
    }
    attempts
}

/*
 * Returns the type filled in a given (unexplicated) storage node
 * If we don't have have a matching place/storage type already, does and returns nothing
 * Modifies state to add the now-explicated node otherwise
 */
fn attempt_empty_allocation_fill(
    output_id: NodeId,
    target_place: expir::Place,
    target_storage_type: ffi::TypeId,
    state: &mut InState,
    context: &StaticContext,
) -> Option<ir::Node> {
    // Returns exactly when the target matches the unpacked current value
    fn unpack_if_equal<T>(current: &Hole<T>, target: T) -> Option<T>
    where
        T: Eq,
    {
        match current {
            Hole::Empty => Some(target),
            Hole::Filled(value) => {
                if target == *value {
                    Some(target)
                } else {
                    None
                }
            }
        }
    };

    let funclet_id = state.get_current_funclet_id();
    match context.get_node(Location::new(funclet_id.clone(), output_id)) {
        expir::Node::AllocTemporary {
            place,
            storage_type,
            buffer_flags,
        } => {
            let found_place = unpack_if_equal(place, target_place);
            let found_storage_type = unpack_if_equal(storage_type, target_storage_type);
            match (found_place, found_storage_type) {
                (Some(p), Some(t)) => {
                    let b = buffer_flags
                        .as_ref()
                        .opt()
                        .expect(&state.hole_error(context))
                        .clone();
                    state.set_storage_type(
                        output_id,
                        expir::Type::Ref {
                            storage_place: p.clone(),
                            storage_type: t.clone(),
                            buffer_flags: b.clone(),
                        },
                        context,
                    );
                    Some(ir::Node::AllocTemporary {
                        place: p,
                        storage_type: t,
                        buffer_flags: b,
                    })
                }
                _ => None,
            }
        }
        expir::Node::StaticSubAlloc {
            node,
            place,
            storage_type,
        } => {
            let found_place = unpack_if_equal(place, target_place);
            let found_storage_type = unpack_if_equal(storage_type, target_storage_type);
            match (found_place, found_storage_type) {
                (Some(p), Some(t)) => {
                    let n = node
                        .as_ref()
                        .opt()
                        .expect(&state.hole_error(context))
                        .clone();
                    state.set_storage_type(
                        output_id,
                        expir::Type::Ref {
                            storage_place: p.clone(),
                            storage_type: t.clone(),
                            buffer_flags: BufferFlags::new(),
                        },
                        context,
                    );
                    Some(ir::Node::StaticSubAlloc {
                        place: p,
                        storage_type: t,
                        node: n,
                    })
                }
                _ => None,
            }
        }
        _ => {
            unreachable!(
                "Node {} should be an allocation",
                context.debug_info.node(&funclet_id, output_id)
            )
        }
    }
}

fn build_do_operation<T>(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    // _super_ dumb parameter, but we need a way to handle constants
    // specifically this is zero exactly for local_do_builtin
    base_offset: usize,
    node_builder: T,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState>
where
    T: Fn(ir::Quotient, Box<[ir::NodeId]>, Box<[ir::NodeId]>) -> ir::Node,
{
    let value_funclet_id = state
        .get_funclet_spec(
            state.get_current_funclet_id(),
            &SpecLanguage::Value,
            context,
        )
        .funclet_id_opt
        .unwrap();

    let operation = expir_operation
        .as_ref()
        .opt()
        .expect(&state.hole_error(context))
        .clone();

    let value_node_id = match operation {
        expir::Quotient::Node { node_id } => node_id,
        _ => panic!(
            "Expected node operation for {}",
            context.debug_info.node_expir(
                state.get_current_funclet_id(),
                state.get_current_node(context).as_ref().opt().unwrap()
            )
        ),
    };

    let node_location = Location::new(value_funclet_id, value_node_id);
    let value_info = context.get_node_type_information(&value_funclet_id, &value_node_id);
    let value_error_text = format!(
        " with value node {} {}",
        context
            .debug_info
            .node(&value_funclet_id, value_node_id.clone()),
        state.get_node_error(context)
    );

    let input_attempts = enumerate_fill_attempts(
        expir_inputs,
        &value_info
            .input_types
            .iter()
            .map(|type_id| context.get_type(type_id).clone())
            .collect(),
        Some(
            context
                .get_node_dependencies(&value_funclet_id, &value_node_id)
                .iter()
                .map(|d| Location::new(value_funclet_id, d.clone()))
                .collect(),
        ),
        &value_error_text,
        &state,
        context,
    );

    let output_attempts = enumerate_fill_attempts(
        expir_outputs,
        &value_info
            .output_types
            .iter()
            .map(|type_id| expir::Type::Ref {
                storage_type: state.expect_native_storage_type(type_id, context),
                storage_place: expir::Place::Local,
                buffer_flags: BufferFlags::new(),
            })
            .collect(),
        None,
        &value_error_text,
        &state,
        context,
    );

    let type_bounds = context.get_node_dependencies(&value_funclet_id, &value_node_id);
    for inputs in input_attempts.iter() {
        for outputs in output_attempts.iter() {
            let mut new_state = state.clone();

            let mut nodes_to_fill = Vec::new();
            // checks whether or not we have a storage requirement mismatch
            let mut valid_fills = true;
            for (offset, output_id) in outputs.iter().enumerate() {
                // here is where we actually use the base_offset
                // note that the off-by-one is for the extract operation
                let value_location =
                    Location::new(value_funclet_id, value_node_id + offset + base_offset);
                let output_info = state.get_node_information(output_id, context);
                let target_storage_type = state
                    .expect_native_storage_type(&value_info.output_types.get(0).unwrap(), context);

                new_state.set_instantiation(
                    output_id.clone(),
                    LocationTriple::new_value(value_location),
                    context,
                );

                let valid_fills = match &output_info.typ {
                    // if we already know the type, we're good
                    Hole::Filled(output_type) => true,
                    Hole::Empty => {
                        match attempt_empty_allocation_fill(
                            output_id.clone(),
                            expir::Place::Local,
                            target_storage_type.clone(),
                            &mut new_state,
                            context,
                        ) {
                            Some(to_fill) => {
                                nodes_to_fill.push((output_id.clone(), to_fill));
                                true
                            }
                            // we fail if we have a mismatch while attempting to fill
                            None => false,
                        }
                    }
                };
            }
            if valid_fills {
                let node = &node_builder(
                    operation.clone(),
                    inputs.clone().into_boxed_slice(),
                    outputs.clone().into_boxed_slice(),
                );
                new_state.next_node();
                match explicate_node(new_state, context) {
                    None => {}
                    Some(mut out) => {
                        out.add_node(node.clone());
                        for (output_id, node_to_fill) in nodes_to_fill.drain(..) {
                            out.add_to_fill(output_id.clone(), node_to_fill);
                        }
                        return Some(out);
                    }
                }
            }
        }
    }

    None
}

fn explicate_local_do_builtin(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
    build_do_operation(
        expir_operation,
        expir_inputs,
        expir_outputs,
        0,
        |operation, inputs, outputs| ir::Node::LocalDoBuiltin {
            operation,
            inputs,
            outputs,
        },
        state,
        context,
    )
}

fn explicate_local_do_external(
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_function_id: &Hole<expir::ExternalFunctionId>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
    let external_function_id = expir_external_function_id
        .as_ref()
        .opt()
        .expect(&state.hole_error(context))
        .clone();

    build_do_operation(
        expir_operation,
        expir_inputs,
        expir_outputs,
        1,
        |operation, inputs, outputs| ir::Node::LocalDoExternal {
            operation,
            inputs,
            outputs,
            external_function_id,
        },
        state,
        context,
    )
}

fn explicate_encode_do(
    expir_encoder: &Hole<usize>,
    expir_operation: &Hole<expir::Quotient>,
    expir_inputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_outputs: &Hole<Box<[Hole<NodeId>]>>,
    expir_external_function_id: &Hole<expir::ExternalFunctionId>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
    let external_function_id = expir_external_function_id
        .as_ref()
        .opt()
        .expect(&state.hole_error(context))
        .clone();
    let encoder = expir_encoder
        .as_ref()
        .opt()
        .expect(&state.hole_error(context))
        .clone();

    let external_function_id = expir_external_function_id
        .as_ref()
        .opt()
        .expect(&state.hole_error(context))
        .clone();

    build_do_operation(
        expir_operation,
        expir_inputs,
        expir_outputs,
        1,
        |operation, inputs, outputs| ir::Node::EncodeDoExternal {
            encoder,
            operation,
            inputs,
            outputs,
            external_function_id,
        },
        state,
        context,
    )
}

fn explicate_write_ref(
    expir_source: &Hole<usize>,
    expir_destination: &Hole<usize>,
    expir_storage_type: &Hole<ffi::TypeId>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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

    let storage_types = match expir_storage_type {
        Hole::Filled(x) => vec![x.clone()],
        Hole::Empty => todo!("{}", error),
    };

    for storage_type in storage_types.iter() {
        let sources = match expir_source {
            Hole::Filled(x) => vec![x.clone()],
            Hole::Empty => todo!("{}", error),
        };

        for source in sources.iter() {
            let destinations = match expir_destination {
                Hole::Filled(x) => vec![x.clone()],
                Hole::Empty => todo!("{}", error),
            };

            for destination in destinations.iter() {
                let mut new_state = state.clone();
                let info = state.get_node_information(&source, context);
                new_state.set_instantiation(
                    destination.clone(),
                    info.instantiation.clone().expect(&format!(
                        "Missing instantiation for node {}",
                        context
                            .debug_info
                            .node(&state.get_current_funclet_id(), source.clone())
                    )),
                    context,
                );
                let node = ir::Node::WriteRef {
                    storage_type: storage_type.clone(),
                    source: source.clone(),
                    destination: destination.clone(),
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
        }
    }

    None
}

fn explicate_borrow_ref(
    expir_storage_type: &Hole<ffi::TypeId>,
    expir_source: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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

    let storage_types = match expir_storage_type {
        Hole::Filled(x) => vec![x.clone()],
        Hole::Empty => (0..context.program.native_interface.types.len())
            .map(|i| ffi::TypeId(i))
            .collect_vec(),
    };

    for storage_type in storage_types.iter() {
        let sources = match expir_source {
            Hole::Filled(x) => vec![x.clone()],
            Hole::Empty => state
                .find_all_storage_nodes(
                    &expir::Type::Ref {
                        storage_type: storage_type.clone(),
                        storage_place: expir::Place::Local,
                        buffer_flags: expir::BufferFlags::new(),
                    },
                    context,
                )
                .iter()
                .map(|loc| loc.node_id(context).unwrap())
                .collect(),
        };

        for source in sources.iter() {
            let info = state.get_node_information(&source, context);
            let schedule_node = state.get_current_node_id().unwrap();

            let instantiation = match info.instantiation.clone() {
                Some(inst) => inst,
                None => {
                    return None;
                }
            };

            let mut new_state = state.clone();

            new_state.add_storage_node(schedule_node, info.typ.clone(), context);
            new_state.set_instantiation(
                schedule_node,
                info.instantiation.clone().expect(&format!(
                    "Missing instantiation for node {}",
                    context
                        .debug_info
                        .node(&state.get_current_funclet_id(), source.clone())
                )),
                context,
            );

            let node = ir::Node::BorrowRef {
                storage_type: storage_type.clone(),
                source: source.clone(),
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
    }

    None
}

fn explicate_read_ref(
    expir_storage_type: &Hole<ffi::TypeId>,
    expir_source: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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
    let storage_types = match expir_storage_type {
        Hole::Filled(x) => vec![x.clone()],
        Hole::Empty => (0..context.program.native_interface.types.len())
            .map(|i| ffi::TypeId(i))
            .collect_vec(),
    };

    for storage_type in storage_types.iter() {
        let sources = match expir_source {
            Hole::Filled(x) => vec![x.clone()],
            Hole::Empty => state
                .find_all_storage_nodes(
                    &expir::Type::Ref {
                        storage_type: storage_type.clone(),
                        storage_place: expir::Place::Local,
                        buffer_flags: expir::BufferFlags::new(),
                    },
                    context,
                )
                .iter()
                .map(|loc| loc.node_id(context).unwrap())
                .collect(),
        };

        for source in sources.iter() {
            let info = state.get_node_information(&source, context);
            let schedule_node = state.get_current_node_id().unwrap();
            let instantiation = match info.instantiation.clone() {
                Some(inst) => inst,
                None => {
                    return None;
                }
            };

            let mut new_state = state.clone();
            new_state.add_storage_node(
                schedule_node,
                Hole::Filled(expir::Type::NativeValue {
                    storage_type: storage_type.clone(),
                }),
                context,
            );
            new_state.set_instantiation(schedule_node, instantiation, context);
            let node = ir::Node::ReadRef {
                storage_type: storage_type.clone(),
                source: source.clone(),
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
    }
    None
}

fn explicate_local_copy(
    expir_input: &Hole<usize>,
    expir_output: &Hole<usize>,
    state: InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
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
) -> Option<StorageOutState> {
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
) -> Option<StorageOutState> {
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
) -> Option<StorageOutState> {
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
) -> Option<StorageOutState> {
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
pub fn explicate_node(state: InState, context: &StaticContext) -> Option<StorageOutState> {
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
                storage_type,
                source,
            }) => explicate_borrow_ref(storage_type, source, state, context),
            Hole::Filled(expir::Node::ReadRef {
                storage_type,
                source,
            }) => explicate_read_ref(storage_type, source, state, context),
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

fn explicate_return(
    expir_return_values: &Hole<Box<[Hole<usize>]>>,
    state: &InState,
    context: &StaticContext,
) -> Option<StorageOutState> {
    let error = format!(
        "TODO Hole in tail_edge of funclet {}",
        context.debug_info.funclet(&state.get_current_funclet_id())
    );
    let mut result = StorageOutState::new();
    let funclet_id = state.get_current_funclet_id();
    let funclet = context.get_funclet(&funclet_id);

    let return_values_todo = match expir_return_values {
        Hole::Filled(values) => values.clone(),
        Hole::Empty => funclet.output_types.iter().map(|_| Hole::Empty).collect(),
    };
    let mut return_values = Vec::new();
    let mut no_matching_type = false;
    for (index, ret) in return_values_todo.iter().enumerate() {
        match ret {
            Hole::Filled(node_id) => {
                let expected_remote = LocationTriple::new_triple_mapped(
                    spec_output,
                    funclet_id,
                    index,
                    state,
                    context,
                );
                let actual_remote = state
                    .get_node_information(node_id, context)
                    .instantiation
                    .clone()
                    .unwrap_or(LocationTriple::new());
                if expected_remote.is_subset_of(&actual_remote, context) {
                    return_values.push(node_id.clone());
                } else {
                    // note that we will fail if types don't match
                    no_matching_type = true;
                }
            }
            Hole::Empty => {
                let target_location_triple = LocationTriple::new_triple_mapped(
                    spec_output,
                    funclet_id,
                    index,
                    state,
                    context,
                )
                .triple_ignoring_none();
                let target_type = context.get_type(get_expect_box(&funclet.output_types, index));
                match state
                    .find_matching_instantiations(&target_location_triple, target_type, context)
                    .first()
                {
                    // we couldn't find anything in our funclet
                    None => {
                        no_matching_type = true;
                    }
                    Some(instantiation) => {
                        if instantiation.funclet_id != funclet_id {
                            // TODO try and explicate something
                            todo!()
                        };
                        return_values.push(instantiation.node_id(context).unwrap());
                    }
                };
            }
        }
    }
    result.set_tail_edge(ir::TailEdge::Return {
        return_values: return_values.into_boxed_slice(),
    });
    if no_matching_type {
        None
    } else {
        Some(result)
    }
}

pub fn explicate_tail_edge(state: &InState, context: &StaticContext) -> Option<StorageOutState> {
    match state.get_current_tail_edge(context) {
        Hole::Filled(tail_edge) => {
            let error = format!("Unimplemented hole in tail edge {:?}", tail_edge);
            match tail_edge {
                expir::TailEdge::Return { return_values } => {
                    explicate_return(return_values, state, context)
                }
                expir::TailEdge::Jump { join, arguments } => {
                    let mut result = StorageOutState::new();
                    let tail_edge = ir::TailEdge::Jump {
                        join: join.as_ref().opt().expect(&error).clone(),
                        arguments: arguments
                            .as_ref()
                            .opt()
                            .expect(&error)
                            .iter()
                            .map(|v| v.clone().opt().expect(&error))
                            .collect(),
                    };
                    result.set_tail_edge(tail_edge);
                    Some(result)
                }
                expir::TailEdge::ScheduleCall {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    callee_funclet_id,
                    callee_arguments,
                    continuation_join,
                } => {
                    let mut result = StorageOutState::new();
                    let tail_edge = ir::TailEdge::ScheduleCall {
                        value_operation: value_operation.clone().opt().expect(&error).clone(),
                        timeline_operation: timeline_operation.clone().opt().expect(&error).clone(),
                        spatial_operation: spatial_operation.clone().opt().expect(&error).clone(),
                        callee_funclet_id: callee_funclet_id.clone().opt().expect(&error).clone(),
                        callee_arguments: callee_arguments
                            .as_ref()
                            .opt()
                            .expect(&error)
                            .iter()
                            .map(|v| v.clone().opt().expect(&error))
                            .collect(),
                        continuation_join: continuation_join.clone().opt().expect(&error).clone(),
                    };
                    result.set_tail_edge(tail_edge);
                    Some(result)
                }
                expir::TailEdge::ScheduleSelect {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    condition,
                    callee_funclet_ids,
                    callee_arguments,
                    continuation_join,
                } => {
                    let mut result = StorageOutState::new();
                    let tail_edge = ir::TailEdge::ScheduleSelect {
                        value_operation: value_operation.clone().opt().expect(&error).clone(),
                        timeline_operation: timeline_operation.clone().opt().expect(&error).clone(),
                        spatial_operation: spatial_operation.clone().opt().expect(&error).clone(),
                        condition: condition.clone().opt().expect(&error).clone(),
                        callee_funclet_ids: callee_funclet_ids
                            .as_ref()
                            .opt()
                            .expect(&error)
                            .iter()
                            .map(|v| v.clone().opt().expect(&error))
                            .collect(),
                        callee_arguments: callee_arguments
                            .as_ref()
                            .opt()
                            .expect(&error)
                            .iter()
                            .map(|v| v.clone().opt().expect(&error))
                            .collect(),
                        continuation_join: continuation_join.clone().opt().expect(&error).clone(),
                    };
                    result.set_tail_edge(tail_edge);
                    Some(result)
                }
                expir::TailEdge::ScheduleCallYield {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    external_function_id,
                    yielded_nodes,
                    continuation_join,
                } => {
                    let mut result = StorageOutState::new();
                    let tail_edge = ir::TailEdge::ScheduleCallYield {
                        value_operation: value_operation.clone().opt().expect(&error).clone(),
                        timeline_operation: timeline_operation.clone().opt().expect(&error).clone(),
                        spatial_operation: spatial_operation.clone().opt().expect(&error).clone(),
                        external_function_id: external_function_id
                            .clone()
                            .opt()
                            .expect(&error)
                            .clone(),
                        yielded_nodes: yielded_nodes
                            .as_ref()
                            .opt()
                            .expect(&error)
                            .iter()
                            .map(|v| v.clone().opt().expect(&error))
                            .collect(),
                        continuation_join: continuation_join.clone().opt().expect(&error).clone(),
                    };
                    result.set_tail_edge(tail_edge);
                    Some(result)
                }
                expir::TailEdge::DebugHole { inputs } => {
                    let mut result = StorageOutState::new();
                    let tail_edge = ir::TailEdge::DebugHole {
                        inputs: inputs.clone(),
                    };
                    result.set_tail_edge(tail_edge);
                    Some(result)
                }
            }
        }
        Hole::Empty => {
            todo!()
        }
    }
}
