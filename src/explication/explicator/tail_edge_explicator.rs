use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::explicator_macros;
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::{explication, frontend, ir};

fn explicate_return(
    return_values: &Hole<Box<[Hole<usize>]>>,
    state: &InState,
    context: &StaticContext,
) -> Option<FuncletOutState> {
    match return_values {
        Hole::Filled(values) => {
            let mut result = FuncletOutState::new();
            result.set_tail_edge(ir::TailEdge::Return {
                return_values: values
                    .iter()
                    .map(|v| v.clone().opt().expect("Unimplemented"))
                    .collect(),
            });
            Some(result)
        }
        Hole::Empty => {
            let funclet = state.get_current_funclet_id();
            dbg!(&state);
            dbg!(&context.get_funclet(&funclet).output_types);
            todo!()
        }
    }
}

pub fn explicate_tail_edge(state: &InState, context: &StaticContext) -> Option<FuncletOutState> {
    match state.get_current_tail_edge(context) {
        Hole::Filled(tail_edge) => {
            let error = format!("Unimplemented hole in tail edge {:?}", tail_edge);
            match tail_edge {
                expir::TailEdge::Return { return_values } => {
                    explicate_return(return_values, state, context)
                }
                expir::TailEdge::Jump { join, arguments } => {
                    let mut result = FuncletOutState::new();
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
                    let mut result = FuncletOutState::new();
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
                    let mut result = FuncletOutState::new();
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
                    let mut result = FuncletOutState::new();
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
                    let mut result = FuncletOutState::new();
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