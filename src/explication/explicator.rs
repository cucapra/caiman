mod operation_explicator;
mod storage_explicator;

use priority_queue::PriorityQueue;
use std::collections::HashMap;

use crate::explication::context::{InState, OperationOutState, StaticContext, StorageOutState};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::stable_vec::StableVec;
use crate::{explication, frontend, ir};

use super::expir::Funclet;
use super::explicator_macros::force_lower_node;

fn explicate_tag(tag: expir::Tag, context: &StaticContext) -> ir::Tag {
    let error = format!("Unimplemented flow hole in with quotient {:?}", &tag);
    ir::Tag {
        quot: tag.quot,
        flow: tag.flow.opt().expect(&error),
    }
}

fn explicate_funclet_spec(
    spec: &expir::FuncletSpec,
    state: &StorageOutState,
    context: &StaticContext,
) -> ir::FuncletSpec {
    let error = format!("Unimplemented Hole in specification {:?}", spec);
    ir::FuncletSpec {
        funclet_id_opt: spec.funclet_id_opt,
        input_tags: spec
            .input_tags
            .iter()
            .map(|t| explicate_tag(t.clone().opt().unwrap_or_default(), context))
            .collect(),
        output_tags: spec
            .output_tags
            .iter()
            .map(|t| explicate_tag(t.clone().opt().unwrap_or_default(), context))
            .collect(),
        implicit_in_tag: explicate_tag(spec.implicit_in_tag.clone().opt().expect(&error), context),
        implicit_out_tag: explicate_tag(
            spec.implicit_out_tag.clone().opt().expect(&error),
            context,
        ),
    }
}

fn explicate_spec_binding(
    funclet: &FuncletId,
    state: Option<&StorageOutState>,
    context: &StaticContext,
) -> ir::FuncletSpecBinding {
    let current = context.get_funclet(&funclet);
    match &current.spec_binding {
        expir::FuncletSpecBinding::None => ir::FuncletSpecBinding::None,
        expir::FuncletSpecBinding::Value {
            value_function_id_opt,
        } => ir::FuncletSpecBinding::Value {
            value_function_id_opt: value_function_id_opt.clone(),
        },
        expir::FuncletSpecBinding::Timeline {
            function_class_id_opt,
        } => ir::FuncletSpecBinding::Timeline {
            function_class_id_opt: function_class_id_opt.clone(),
        },
        expir::FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } => ir::FuncletSpecBinding::ScheduleExplicit {
            value: explicate_funclet_spec(value, state.unwrap(), context),
            spatial: explicate_funclet_spec(spatial, state.unwrap(), context),
            timeline: explicate_funclet_spec(timeline, state.unwrap(), context),
        },
    }
}

/*
 * The first pass of explication, where we fill in ??? with necessary operations
 *   we also fill in specific operations that are purely type-directed
 *   we do not attempt to actually put data in storage
 * Returns the updated funclet first, then any new funclets to add second
 */
pub fn explicate_schedule_funclet_operation(
    funclet_id: FuncletId,
    context: &StaticContext,
) -> (expir::Funclet, Vec<expir::Funclet>) {
    let mut state = InState::new_operation(funclet_id, context);
    state.next_node();
    let funclet = context.get_funclet(&funclet_id);
    match operation_explicator::explicate_node(state, context) {
        None => panic!(
            "No explication solution found for funclet {:?}",
            context.debug_info.funclet(&funclet_id)
        ),
        Some(mut result) => (
            expir::Funclet {
                kind: funclet.kind.clone(),
                spec_binding: funclet.spec_binding.clone(),
                input_types: funclet.input_types.clone(),
                output_types: funclet.output_types.clone(),
                tail_edge: result.take_tail_edge().into(),
                nodes: result.drain_nodes().into_boxed_slice(),
            },
            vec![],
        ),
    }
}

/*
 * The second pass of explication, where we assume we have the operations we need
 *   and now we need to actually put the stuff in the correct storage at the right time
 */
pub fn explicate_schedule_funclet_storage(
    funclet_id: FuncletId,
    context: &StaticContext,
) -> ir::Funclet {
    let mut state = InState::new_storage(funclet_id, context);
    state.next_node();
    let funclet = context.get_funclet(&funclet_id);
    match storage_explicator::explicate_node(state, context) {
        None => panic!(
            "No explication solution found for funclet {:?}",
            context.debug_info.funclet(&funclet_id)
        ),
        Some(mut result) => {
            let spec_binding = explicate_spec_binding(&funclet_id, Some(&result), context);
            ir::Funclet {
                kind: funclet.kind.clone(),
                spec_binding,
                input_types: funclet.input_types.clone(),
                output_types: funclet.output_types.clone(),
                tail_edge: result.expect_tail_edge(),
                nodes: result.drain_nodes().into_boxed_slice(),
            }
        }
    }
}

/*
 * Forcibly lowers a tail edge, specifically used for spec functions
 * The funclet id is passed in rather than the tail edge for error context
 */
fn lower_spec_tail_edge(funclet: &FuncletId, context: &StaticContext) -> ir::TailEdge {
    let debug_funclet = context.debug_info.funclet(funclet);
    let tail_edge = context
        .get_funclet(funclet)
        .tail_edge
        .as_ref()
        .opt()
        .expect(&format!("Missing tail edge for funclet {}", &debug_funclet));
    let error = format!(
        "Tail edge {:?} is part of the spec funclet {} and cannot have holes in it",
        tail_edge, debug_funclet
    );
    match tail_edge {
        expir::TailEdge::Return { return_values } => ir::TailEdge::Return {
            return_values: return_values
                .as_ref()
                .opt()
                .expect(&error)
                .iter()
                .map(|v| v.clone().opt().expect(&error))
                .collect(),
        },
        expir::TailEdge::Jump { join, arguments } => ir::TailEdge::Jump {
            join: join.as_ref().opt().expect(&error).clone(),
            arguments: arguments
                .as_ref()
                .opt()
                .expect(&error)
                .iter()
                .map(|v| v.clone().opt().expect(&error))
                .collect(),
        },
        expir::TailEdge::DebugHole { inputs } => ir::TailEdge::DebugHole {
            inputs: inputs.clone(),
        },
        edge => {
            panic!(
                "Spec funclet {} has disallowed tail edge {:?}",
                debug_funclet, edge
            )
        }
    }
}

pub fn lower_spec_funclet(funclet: &FuncletId, context: &StaticContext) -> ir::Funclet {
    let func = context.get_funclet(&funclet);
    let kind = func.kind.clone();
    let spec_binding = explicate_spec_binding(funclet, None, context);
    let input_types = func.input_types.clone();
    let output_types = func.output_types.clone();
    let debug_funclet = &context.debug_info.funclet(&funclet);
    let error = format!("Cannot have a hole in spec funclet {:?}", debug_funclet);
    let nodes = func
        .nodes
        .iter()
        .map(|n| {
            explicator_macros::force_lower_node(&n.as_ref().opt().expect(&error), &debug_funclet)
        })
        .collect();
    let tail_edge = lower_spec_tail_edge(&funclet, context);

    ir::Funclet {
        kind,
        spec_binding,
        input_types,
        output_types,
        nodes,
        tail_edge,
    }
}
