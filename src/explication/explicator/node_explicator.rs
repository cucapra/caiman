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

fn read_phi_node(expir_location: &Location, index: usize, context: &StaticContext) -> expir::Node {
    todo!()
    // let current_funclet = context.get_funclet(&location.funclet);
    // let argument = current_funclet.header.args.get(index).unwrap_or_else(|| {
    //     panic!(
    //         "Index {} out of bounds for header in location {:?}",
    //         index, location
    //     )
    // });
    // let mut remotes = Vec::new();
    // for tag in &argument.tags {
    //     let quotient = tag_quotient(tag);
    //     match quotient {
    //         None => {}
    //         Some(remote) => remotes.push(remote.clone()),
    //     }
    // }
    // let place = context.get_type_place(&argument.typ);
    // context.add_instantiation(location.node.clone(), remotes, place.cloned());
    // expir::Node::Phi { index: Some(index) }
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
        storage_type.0.clone(),
        expir::Type::Ref {
            storage_type,
            storage_place,
            buffer_flags,
        },
        context
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
    for output in expir_outputs.as_ref().opt().expect(&error).iter() {
        let output_open = output.as_ref().opt().expect(&error).clone();
        let typ = new_state.consume_allocation(Location {
            funclet: state.get_current_funclet_id(),
            node: output_open,
        });
        new_state.add_instantiation(
            output_open,
            vec![Location {
                funclet: state
                    .get_funclet_spec(
                        state.get_current_funclet_id(),
                        &expir::FuncletKind::Value,
                        context,
                    )
                    .funclet_id_opt
                    .unwrap(),
                node: node_id,
            }],
            typ,
            context
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

    let instantiation_location = state.get_node_instantiation(source, context);
    new_state.add_instantiation(
        state.get_current_node_id().unwrap(),
        vec![instantiation_location],
        expir::Type::NativeValue { storage_type },
        context
    );
    let node = ir::Node::ReadRef { storage_type, source };
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
        match state.get_current_node(context) {
            Hole::Empty => {
                let mut new_state = state.clone();
                new_state.add_explication_hole();
                explicate_node(new_state, context);
                todo!()
            }
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
            Hole::Filled(expir::Node::ReadRef {
                source,
                storage_type,
            }) => explicate_read_ref(source, storage_type, state, context),
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
