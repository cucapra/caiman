use crate::explication::context::{FuncletOutState, InState, OpCode, StaticContext};
use crate::explication::expir;
use crate::explication::expir::{FuncletId, NodeId};
use crate::explication::util::Location;
use crate::explication::util::*;
use crate::explication::Hole;
use crate::ir::Place;
use crate::{explication, frontend, ir};

fn explicate_tag(tag: expir::Tag, context: &StaticContext) -> ir::Tag {
    ir::Tag {
        quot: tag.quot,
        flow: tag.flow.expect("Unimplemented hole"),
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
    todo!()
    // let maybe_node = state.get_current_node(context);
    // if let Some(node) = maybe_node {
    //     match node {
    //         expir::Node::None => ir::Node::None,
    //         expir::Node::Phi { index } => todo!(),
    //         expir::Node::ExtractResult { node_id, index } => todo!(),
    //         expir::Node::Constant { value, type_id } => todo!(),
    //         expir::Node::CallFunctionClass { function_id, arguments } => todo!(),
    //         expir::Node::Select { condition, true_case, false_case } => todo!(),
    //         expir::Node::AllocTemporary { place, storage_type, buffer_flags } => todo!(),
    //         expir::Node::Drop { node } => todo!(),
    //         expir::Node::StaticSubAlloc { node, place, storage_type } => todo!(),
    //         expir::Node::StaticSplit { spatial_operation, node, sizes, place } => todo!(),
    //         expir::Node::StaticMerge { spatial_operation, nodes, place } => todo!(),
    //         expir::Node::ReadRef { storage_type, source } => todo!(),
    //         expir::Node::BorrowRef { storage_type, source } => todo!(),
    //         expir::Node::WriteRef { storage_type, destination, source } => todo!(),
    //         expir::Node::LocalDoBuiltin { operation, inputs, outputs } => todo!(),
    //         expir::Node::LocalDoExternal { operation, external_function_id, inputs, outputs } => todo!(),
    //         expir::Node::LocalCopy { input, output } => todo!(),
    //         expir::Node::BeginEncoding { place, event, encoded, fences } => todo!(),
    //         expir::Node::EncodeDoExternal { encoder, operation, external_function_id, inputs, outputs } => todo!(),
    //         expir::Node::EncodeCopy { encoder, input, output } => todo!(),
    //         expir::Node::Submit { encoder, event } => todo!(),
    //         expir::Node::SyncFence { fence, event } => todo!(),
    //         expir::Node::InlineJoin { funclet, captures, continuation } => todo!(),
    //         expir::Node::SerializedJoin { funclet, captures, continuation } => todo!(),
    //         expir::Node::DefaultJoin => todo!(),
    //         expir::Node::PromiseCaptures { count, continuation } => todo!(),
    //         expir::Node::FulfillCaptures { continuation, haves, needs } => todo!(),
    //         expir::Node::EncodingEvent { local_past, remote_local_pasts } => todo!(),
    //         expir::Node::SubmissionEvent { local_past } => todo!(),
    //         expir::Node::SynchronizationEvent { local_past, remote_local_past } => todo!(),
    //         expir::Node::SeparatedBufferSpaces { count, space } => todo!(),
    //     };
    // }
    // else {
    //     todo!()
    // }
}

fn explicate_funclet_spec(
    spec: &expir::FuncletSpec,
    state: &FuncletOutState,
    context: &StaticContext,
) -> ir::FuncletSpec {
    ir::FuncletSpec {
        funclet_id_opt: spec.funclet_id_opt,
        input_tags: spec
            .input_tags
            .iter()
            .map(|t| explicate_tag(t.expect("Unimplemented Hole"), context))
            .collect(),
        output_tags: spec
            .output_tags
            .iter()
            .map(|t| explicate_tag(t.expect("Unimplemented Hole"), context))
            .collect(),
        implicit_in_tag: explicate_tag(spec.implicit_in_tag.expect("Unimplemented Hole"), context),
        implicit_out_tag: explicate_tag(
            spec.implicit_out_tag.expect("Unimplemented Hole"),
            context,
        ),
    }
}

fn explicate_spec_binding(
    funclet: FuncletId,
    state: &FuncletOutState,
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
            value: explicate_funclet_spec(value, state, context),
            spatial: explicate_funclet_spec(spatial, state, context),
            timeline: explicate_funclet_spec(timeline, state, context),
        },
    }
}

pub fn explicate_schedule_funclet(
    funclet: FuncletId,
    mut state: InState,
    context: &StaticContext,
) -> ir::Funclet {
    state.enter_funclet(funclet);
    let current = context.get_funclet(funclet);
    match explicate_node(state, context) {
        None => panic!("No explication solution found for funclet {:?}", funclet),
        Some(mut result) => {
            assert!(!result.has_fills_remaining());
            let spec_binding = explicate_spec_binding(funclet, &result, context);
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

macro_rules! generate_satisfiers {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            fn satisfy_explication_request(current: ast::Node, requested: &ast::Node) -> ast::Node {
                match (current, requested) {
                    $((ast::Node::$name { $($arg : [<$arg _one>],)* },
                    ast::Node::$name { $($arg : [<$arg _two>],)* }) => {
                        $(
                            let $arg = satisfy_argument!([<$arg _one>] [<$arg _two>] $arg_type);
                        )*
                        ast::Node::$name { $($arg,)* }
                    })*
                    (current, _) => unreachable!("Trying to request {:?} from {:?}", requested, current)
                }
            }
        }
    };
}

fn lower_spec_funclet(funclet: FuncletId, context: &StaticContext) -> ir::Funclet {
    todo!()
}