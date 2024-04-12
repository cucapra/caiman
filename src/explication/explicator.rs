mod tail_edge_explicator;
mod node_explicator;

use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
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

// the function that handles "ok, I have an output, now figure out how to get there"
// searches exactly the given spec language of the "location" funclet
fn deduce_operation(
    location: &Location,
    outputs: &Hole<Vec<Hole<NodeId>>>,
    spec: &SpecLanguage,
    context: &StaticContext,
) -> Location {
    todo!()
    // let spec_funclet = context.get_spec_funclet(&location.funclet, spec);
    // match outputs {
    //     None => Location {
    //         funclet: Some(spec_funclet.clone()),
    //         node: None,
    //     },
    //     Some(outs) => {
    //         let output_specs: Vec<Hole<&NodeId>> = outs
    //             .iter()
    //             .map(|hole| {
    //                 hole.as_ref().and_then(|output| {
    //                     context.get_spec_instantiation(&location.funclet, output, spec)
    //                 })
    //             })
    //             .collect();
    //         let spec_node = context.get_matching_operation(&location.funclet, output_specs);
    //         Location {
    //             funclet: Some(spec_funclet.clone()),
    //             node: spec_node.cloned(),
    //         }
    //     }
    // }
}

fn explicate_funclet_spec(
    spec: &expir::FuncletSpec,
    state: &FuncletOutState,
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
    state: Option<&FuncletOutState>,
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

pub fn explicate_schedule_funclet(mut state: InState, context: &StaticContext) -> ir::Funclet {
    let funclet = state.get_current_funclet_id();
    let current = context.get_funclet(&funclet);
    state.next_node();
    match node_explicator::explicate_node(state, context) {
        None => panic!(
            "No explication solution found for funclet {:?}",
            context.debug_info.funclet(&funclet)
        ),
        Some(mut result) => {
            assert!(!result.has_fills_remaining());
            let spec_binding = explicate_spec_binding(&funclet, Some(&result), context);
            ir::Funclet {
                kind: current.kind.clone(),
                spec_binding,
                input_types: current.input_types.clone(),
                output_types: current.output_types.clone(),
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
