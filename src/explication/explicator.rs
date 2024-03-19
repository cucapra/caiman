use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::{explication, frontend, ir};

use super::explicator_macros::force_lower_node;

fn explicate_tag(tag: expir::Tag, context: &StaticContext) -> ir::Tag {
    let error = format!("Unimplemented flow hole in tag {:?}", &tag);
    ir::Tag {
        quot: tag.quot,
        flow: tag
            .flow
            .opt()
            .expect(&error),
    }
}

fn read_phi_node(location: &Location, index: usize, context: &StaticContext) -> expir::Node {
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

fn explicate_local_do_builtin(
    location: &Location,
    og_operation: Hole<expir::Quotient>,
    og_inputs: Hole<Vec<Hole<NodeId>>>,
    og_outputs: Hole<Vec<Hole<NodeId>>>,
    context: &StaticContext,
) -> expir::Node {
    todo!()
    // let mut available = false;

    // let deduced_op = match og_operation {
    //     Some(q) => {
    //         let op =
    //             quotient_id(&q).unwrap_or_else(|| panic!("Assuming operations must not be Nones"));
    //         match &op.node {
    //             Some(n) => op,
    //             None => {
    //                 // kinda stupid, we just ignore the funclet here
    //                 // but that's ok I think cause a bad funclet will be caught by typechecking
    //                 deduce_operation(&location, &og_outputs, &SpecLanguage::Value, context)
    //             }
    //         }
    //     }
    //     None => deduce_operation(&location, &og_outputs, &SpecLanguage::Value, context),
    // };

    // available = available || deduced_op.funclet.is_none() || deduced_op.node.is_none();

    // let mut expected_inputs = Vec::new();
    // let mut expected_outputs = Vec::new();
    // match (&deduced_op.funclet, &deduced_op.node) {
    //     (Some(f), Some(n)) => {}
    // }

    // let outputs = match og_outputs {
    //     None => {
    //         // match
    //     }
    //     Some(ogo) => {
    //         let mut result = Vec::new();
    //         for output in ogo {
    //             match output {
    //                 Some(out) => Some(out),
    //                 None => {}
    //             }
    //         }
    //         result
    //     }
    // };

    // // if there's stuff left to explicate, make this available and return
    // if available {
    //     context.add_available_operation(location.node.clone(), OpCode::LocalDoBuiltin);
    // }
    // let operation = Some(expir::Quotient::Node(Some(deduced_op)));
}

// initially setup a node that hasn't yet been read
// distinct from explication in that we have no request to fulfill
// panics if no node can be found during any step of the recursion
fn explicate_node(state: InState, context: &StaticContext) -> Option<FuncletOutState> {
    if state.is_end_of_funclet(context) {
        explicate_tail_edge(&state, context)
    } else {
        match state.get_current_node(context) {
            Hole::Empty => {
                todo!()
            }
            Hole::Filled(node) => {
                let mut new_state = state.clone();
                new_state.next_node();
                match explicate_node(new_state, context) {
                    None => None,
                    Some(mut out) => {
                        out.add_node(force_lower_node(node));
                        Some(out)
                    }
                }
            }
        }
    }
}

fn explicate_tail_edge(state: &InState, context: &StaticContext) -> Option<FuncletOutState> {
    let tail_edge = match state.get_current_tail_edge(context) {
        Hole::Filled(tail_edge) => {
            let error = format!("Unimplemented hole in tail edge {:?}", tail_edge);
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
                expir::TailEdge::ScheduleCall {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    callee_funclet_id,
                    callee_arguments,
                    continuation_join,
                } => ir::TailEdge::ScheduleCall {
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
                },
                expir::TailEdge::ScheduleSelect {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    condition,
                    callee_funclet_ids,
                    callee_arguments,
                    continuation_join,
                } => ir::TailEdge::ScheduleSelect {
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
                },
                expir::TailEdge::ScheduleCallYield {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    external_function_id,
                    yielded_nodes,
                    continuation_join,
                } => ir::TailEdge::ScheduleCallYield {
                    value_operation: value_operation.clone().opt().expect(&error).clone(),
                    timeline_operation: timeline_operation.clone().opt().expect(&error).clone(),
                    spatial_operation: spatial_operation.clone().opt().expect(&error).clone(),
                    external_function_id: external_function_id.clone().opt().expect(&error).clone(),
                    yielded_nodes: yielded_nodes
                        .as_ref()
                        .opt()
                        .expect(&error)
                        .iter()
                        .map(|v| v.clone().opt().expect(&error))
                        .collect(),
                    continuation_join: continuation_join.clone().opt().expect(&error).clone(),
                },
                expir::TailEdge::DebugHole { inputs } => ir::TailEdge::DebugHole {
                    inputs: inputs.clone(),
                },
            }
        }
        Hole::Empty => {
            todo!()
        }
    };
    let mut result = FuncletOutState::new();
    result.set_tail_edge(tail_edge);
    Some(result)
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
            .map(|t| explicate_tag(t.clone().opt().expect(&error), context))
            .collect(),
        output_tags: spec
            .output_tags
            .iter()
            .map(|t| explicate_tag(t.clone().opt().expect(&error), context))
            .collect(),
        implicit_in_tag: explicate_tag(spec.implicit_in_tag.clone().opt().expect(&error), context),
        implicit_out_tag: explicate_tag(spec.implicit_out_tag.clone().opt().expect(&error), context),
    }
}

fn explicate_spec_binding(
    funclet: FuncletId,
    state: Option<&FuncletOutState>,
    context: &StaticContext,
) -> ir::FuncletSpecBinding {
    let current = context.get_funclet(funclet);
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
    let funclet = state.get_current_funclet();
    let current = context.get_funclet(funclet);
    state.next_node();
    match explicate_node(state, context) {
        None => panic!("No explication solution found for funclet {:?}", funclet),
        Some(mut result) => {
            assert!(!result.has_fills_remaining());
            let spec_binding = explicate_spec_binding(funclet, Some(&result), context);
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
 */
fn lower_spec_tail_edge(tail_edge: &expir::TailEdge, context: &StaticContext) -> ir::TailEdge {
    let error = format!("Tail edge {:?} cannot have holes in it", tail_edge);
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
            panic!("Tail edge {:?} not allowed in spec function", &edge)
        }
    }
}

pub fn lower_spec_funclet(funclet: FuncletId, context: &StaticContext) -> ir::Funclet {
    let func = context.get_funclet(funclet);
    let kind = func.kind.clone();
    let spec_binding = explicate_spec_binding(funclet, None, context);
    let input_types = func.input_types.clone();
    let output_types = func.output_types.clone();
    let error = format!("Cannot have a hole in spec funclet {:?}", &funclet);
    let nodes = func
        .nodes
        .iter()
        .map(|n| explicator_macros::force_lower_node(&n.as_ref().opt().expect(&error)))
        .collect();
    let tail_edge = lower_spec_tail_edge(&func.tail_edge.as_ref().opt().expect(&error), context);

    ir::Funclet {
        kind,
        spec_binding,
        input_types,
        output_types,
        nodes,
        tail_edge,
    }
}
