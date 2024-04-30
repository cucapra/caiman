mod node_explicator;
mod tail_edge_explicator;

use priority_queue::PriorityQueue;
use std::collections::HashMap;

use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
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

/*
 * counts the number of node dependencies of every node in this spec funclet
 * returns all of these counts as a map from node to count
 * note that "dead nodes" not dependended on by the tail edge will be skipped
 */
fn count_node_dependencies(
    funclet_id: &FuncletId,
    context: &StaticContext,
) -> HashMap<NodeId, usize> {
    fn count_node_dependencies_rec(
        found: &mut HashMap<NodeId, usize>,
        funclet_id: &FuncletId,
        node_id: &NodeId,
        context: &StaticContext,
    ) -> usize {
        if found.contains_key(node_id) {
            return found.get(node_id).unwrap().clone();
        };
        let mut dependency_count = 0;
        for dependency in context.get_node_dependencies(funclet_id, node_id) {
            dependency_count +=
                1 + count_node_dependencies_rec(found, funclet_id, dependency, context);
        }
        found.insert(node_id.clone(), dependency_count);
        dependency_count
    }
    let mut result = HashMap::new();
    for node_id in context.get_tail_edge_dependencies(funclet_id) {
        count_node_dependencies_rec(&mut result, funclet_id, node_id, context);
    }
    result
}

fn find_available_explication_holes(
    funclet_id: &FuncletId,
    context: &StaticContext,
) -> Vec<expir::NodeId> {
    let mut result = Vec::new();
    for (index, node) in context.get_funclet(funclet_id).nodes.iter().enumerate().reverse() {
        match node {
            Hole::Empty => result.push(index),
            Hole::Filled(_) => {}
        }
    };
    result
}

/*
 * Given a vector of nodes, finds the last _operation_ with any hole
 */
fn find_best_operation_hole_with_args(
    nodes: &StableVec<expir::Node>,
    args: usize,
    context: &StaticContext,
) -> Vec<expir::NodeId> {
    let mut result = Vec::new();
    for (index, node) in context.get_funclet(funclet_id).nodes.iter().enumerate() {
        match node {
            expir::Node::LocalDoBuiltin { operation, inputs, outputs } => todo!(),
            expir::Node::LocalDoExternal { operation, external_function_id, inputs, outputs } => todo!(),
            expir::Node::EncodeDoExternal { encoder, operation, external_function_id, inputs, outputs } => todo!(),
            Hole::Filled(_) => todo!(),
            _ => {}
        }
    }
    result
}

/*
 * Adds local_do, encode_do, and local_builtin operations to the graph in type order
 * This is done by looping through every type that has an already-filled dependency
 */
fn add_do_operations(funclet_id: &FuncletId, context: &StaticContext) -> StableVec<expir::Node> {
    let hole_error = format!(
        "TODO Hole in funclet {}",
        context.debug_info.funclet(funclet_id,)
    );
    let funclet = context.get_funclet(funclet_id);
    let dependency_counts = count_node_dependencies(funclet_id, context);
    let mut result = StableVec::new();

    // get the spec information
    let (value_spec, timeline_spec, spatial_spec) = match &funclet.spec_binding {
        expir::FuncletSpecBinding::ScheduleExplicit {
            value,
            timeline,
            spatial,
        } => (value, timeline, spatial),
        spec_binding => panic!(
            "Expected Schedule binding, got {:?} for funclet {}",
            &funclet.spec_binding,
            context.debug_info.funclet(funclet_id)
        ),
    };
    let value_id = &value_spec.funclet_id_opt.unwrap();
    let timeline_spec_id = &timeline_spec.funclet_id_opt.unwrap();
    let spatial_id = &spatial_spec.funclet_id_opt.unwrap();
    let tail_edge_deps = context.get_tail_edge_dependencies(funclet_id);

    // start with the outputs of this scheduling function
    let mut target_value_types = PriorityQueue::new();
    for output_tag in value_spec.output_tags.iter() {
        let n = match &output_tag.as_ref().opt().expect(&hole_error).quot {
            ir::Quotient::None => None,
            ir::Quotient::Node { node_id } => Some(node_id),
            ir::Quotient::Input { index } => Some(index),
            ir::Quotient::Output { index } => Some(tail_edge_deps.get(index.clone()).unwrap()),
        };
        match n {
            None => {}
            Some(node_id) => {
                let priority = dependency_counts.get(node_id).unwrap();
                target_value_types.push(node_id.clone(), priority.clone());
            }
        }
    }

    /*
     * loop until we have either found an existing encode of each node or added one
     * when we need to fill a `?`, use the node with the highest priority (most dependencies)
     * if we cannot fill the dependencies, simply panic
     * we will only fill a ??? when we need to in order to manage our dependency chain
     *   aka when we don't have any "do" operation with a ? available where we're at
     * the intuition is to "work backwards"
     */
    while !target_value_types.is_empty() {}
    result
}

/*
 * The first pass of explication, where we fill in ??? with necessary operations
 *   we also fill in specific operations that are purely type-directed
 *   we do not attempt to actually put data in storage
 */
pub fn type_link_schedule_funclet(
    funclet_id: &FuncletId,
    context: &StaticContext,
) -> Vec<expir::Funclet> {
    let updated_nodes = add_do_operations(funclet_id, context);
    todo!()
}

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

/*
 * The second pass of explication, where we assume we have the operations we need
 *   and now we need to actually put the stuff in the correct storage at the right time
 */
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
