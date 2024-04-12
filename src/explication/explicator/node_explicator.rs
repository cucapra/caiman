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

fn spec_location(
    spec: &expir::FuncletSpec,
    funclet_id: FuncletId,
    node_id: NodeId,
    error: &str,
    context: &StaticContext,
) -> Option<Location> {
    let index_error = format!(
        "Funclet {} does not have enough arguments for phi node {}",
        context.debug_info.funclet(&funclet_id),
        context.debug_info.node(&funclet_id, node_id)
    );
    match &spec.input_tags.get(node_id).expect(&index_error) {
        Hole::Empty => None,
        Hole::Filled(t) => match t.quot {
            ir::Quotient::None => None,
            ir::Quotient::Node { node_id } | ir::Quotient::Input { index: node_id } => {
                Some(Location {
                    funclet: spec.funclet_id_opt.unwrap(),
                    node: node_id.clone(),
                })
            }
            ir::Quotient::Output { index } => panic!(
                "Not sure to do with an output as an input for node {}",
                context.debug_info.node(&funclet_id, node_id)
            ),
        },
    }
}

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
            new_state.add_allocation(node_id, node_type.clone(), context);
        }
        _ => {}
    }

    let (value_spec, timeline_spec, spatial_spec) =
        match &context.get_funclet(&funclet_id).spec_binding {
            expir::FuncletSpecBinding::ScheduleExplicit {
                value,
                timeline,
                spatial,
            } => (value, timeline, spatial),
            _ => {
                unreachable!(
                    "{} is not a scheduling funclet",
                    context.debug_info.funclet(&funclet_id)
                )
            }
        };

    let location = LocationTriple {
        value: spec_location(value_spec, funclet_id, node_id, &error, context),
        timeline: spec_location(timeline_spec, funclet_id, node_id, &error, context),
        spatial: spec_location(spatial_spec, funclet_id, node_id, &error, context),
    };

    let value_location = new_state.set_instantiation(node_id, location, node_type, context);
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
    new_state.add_allocation(
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
        let typ = new_state.read_allocation(Location {
            funclet: state.get_current_funclet_id(),
            node: output_open,
        });
        new_state.set_instantiation(
            output_open,
            LocationTriple::new_value(Location {
                funclet: state
                    .get_funclet_spec(
                        state.get_current_funclet_id(),
                        &expir::FuncletKind::Value,
                        context,
                    )
                    .funclet_id_opt
                    .unwrap(),
                node: node_id,
            }),
            typ,
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
        let typ = new_state.read_allocation(Location {
            funclet: state.get_current_funclet_id(),
            node: output_open,
        });
        new_state.set_instantiation(
            output_open,
            LocationTriple::new_value(Location {
                funclet: state
                    .get_funclet_spec(
                        state.get_current_funclet_id(),
                        &expir::FuncletKind::Value,
                        context,
                    )
                    .funclet_id_opt
                    .unwrap(),
                node: node_id + offset + 1,
            }),
            typ,
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
        let typ = new_state.read_allocation(Location {
            funclet: state.get_current_funclet_id(),
            node: output_open,
        });
        let value_location = Location {
            funclet: state
                .get_funclet_spec(
                    state.get_current_funclet_id(),
                    &expir::FuncletKind::Value,
                    context,
                )
                .funclet_id_opt
                .unwrap(),
            node: node_id + offset + 1,
        };
        new_state.set_instantiation(
            output_open,
            LocationTriple {
                value: Some(value_location),
                // TODO: add timeline
                timeline: None,
                spatial: None,
            },
            typ,
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
    let (instantiation_location, _) = state.get_node_information(source, context);
    // assume that the typechecker will find a misaligned storage type
    let allocation_type = state.read_allocation(Location {
        funclet: state.get_current_funclet_id(),
        node: destination,
    });
    new_state.set_instantiation(
        destination,
        instantiation_location.clone(),
        allocation_type,
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

    let (instantiation_location, _) = state.get_node_information(source, context);
    let schedule_node = state.get_current_node_id().unwrap();
    let ref_type = expir::Type::Ref {
        storage_type: storage_type.clone(),
        storage_place: expir::Place::Local,
        buffer_flags: expir::BufferFlags::new(),
    };

    new_state.add_allocation(schedule_node, ref_type.clone(), context);
    new_state.set_instantiation(
        schedule_node,
        instantiation_location.clone(),
        ref_type,
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

    let (instantiation_location, _) = state.get_node_information(source, context);
    new_state.set_instantiation(
        state.get_current_node_id().unwrap(),
        instantiation_location.clone(),
        expir::Type::NativeValue { storage_type },
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

fn explicate_copy(
    expir_input: &Hole<usize>,
    expir_output: &Hole<usize>,
    expir_encoder: Option<&Hole<usize>>,
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
    let (instantiation, typ) = state.get_node_information(input, context);
    new_state.set_instantiation(output, instantiation.clone(), typ.clone(), context);
    let node = match expir_encoder {
        None => ir::Node::LocalCopy { input, output },
        Some(e) => {
            let encoder = e.as_ref().opt().expect(&error).clone();
            ir::Node::EncodeCopy {
                encoder,
                input,
                output,
            }
        }
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
                explicate_copy(input, output, None, state, context)
            }
            Hole::Filled(expir::Node::EncodeCopy {
                input,
                output,
                encoder,
            }) => explicate_copy(input, output, Some(encoder), state, context),
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
