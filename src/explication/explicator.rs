use crate::explication::context::{InState, FuncletOutState, StaticContext};
use crate::explication::util::*;
use crate::ir::Place;
use crate::{explication, frontend, ir};
use crate::explication::expir;
use crate::explication::expir::{RemoteNodeId, NodeId, FuncletId};

fn quotient_id(quot: &expir::Quotient) -> &Hole<RemoteNodeId> {
    match quot {
        expir::Quotient::None => &None,
        expir::Quotient::Node(n) => n,
        expir::Quotient::Input(n) => n,
        expir::Quotient::Output(n) => n,
    }
}

fn tag_quotient(tag: &expir::Tag) -> &Hole<RemoteNodeId> {
    quotient_id(&tag.quot)
}

fn read_phi_node(location: &Location, index: usize, context: &mut Context) -> expir::Node {
    let current_funclet = context.get_funclet(&location.funclet);
    let argument = current_funclet.header.args.get(index).unwrap_or_else(|| {
        panic!(
            "Index {} out of bounds for header in location {:?}",
            index, location
        )
    });
    let mut remotes = Vec::new();
    for tag in &argument.tags {
        let quotient = tag_quotient(tag);
        match quotient {
            None => {}
            Some(remote) => remotes.push(remote.clone()),
        }
    }
    let place = context.get_type_place(&argument.typ);
    context.add_instantiation(location.node.clone(), remotes, place.cloned());
    expir::Node::Phi { index: Some(index) }
}

// the function that handles "ok, I have an output, now figure out how to get there"
// searches exactly the given spec language of the "location" funclet
fn deduce_operation(
    location: &Location,
    outputs: &Hole<Vec<Hole<NodeId>>>,
    spec: &SpecLanguage,
    context: &mut Context,
) -> RemoteNodeId {
    let spec_funclet = context.get_spec_funclet(&location.funclet, spec);
    match outputs {
        None => RemoteNodeId {
            funclet: Some(spec_funclet.clone()),
            node: None,
        },
        Some(outs) => {
            let output_specs: Vec<Hole<&NodeId>> = outs
                .iter()
                .map(|hole| {
                    hole.as_ref().and_then(|output| {
                        context.get_spec_instantiation(&location.funclet, output, spec)
                    })
                })
                .collect();
            let spec_node = context.get_matching_operation(&location.funclet, output_specs);
            RemoteNodeId {
                funclet: Some(spec_funclet.clone()),
                node: spec_node.cloned(),
            }
        }
    }
}

fn explicate_local_do_builtin(
    location: &Location,
    og_operation: Hole<expir::Quotient>,
    og_inputs: Hole<Vec<Hole<NodeId>>>,
    og_outputs: Hole<Vec<Hole<NodeId>>>,
    context: &mut Context,
) -> expir::Node {
    let mut available = false;

    let deduced_op = match og_operation {
        Some(q) => {
            let op =
                quotient_id(&q).unwrap_or_else(|| panic!("Assuming operations must not be Nones"));
            match &op.node {
                Some(n) => op,
                None => {
                    // kinda stupid, we just ignore the funclet here
                    // but that's ok I think cause a bad funclet will be caught by typechecking
                    deduce_operation(&location, &og_outputs, &SpecLanguage::Value, context)
                }
            }
        }
        None => deduce_operation(&location, &og_outputs, &SpecLanguage::Value, context),
    };

    available = available || deduced_op.funclet.is_none() || deduced_op.node.is_none();

    let mut expected_inputs = Vec::new();
    let mut expected_outputs = Vec::new();
    match (&deduced_op.funclet, &deduced_op.node) {
        (Some(f), Some(n)) => {}
    }

    let outputs = match og_outputs {
        None => {
            // match
        }
        Some(ogo) => {
            let mut result = Vec::new();
            for output in ogo {
                match output {
                    Some(out) => Some(out),
                    None => {}
                }
            }
            result
        }
    };

    // if there's stuff left to explicate, make this available and return
    if available {
        context.add_available_operation(location.node.clone(), OpCode::LocalDoBuiltin);
    }
    let operation = Some(expir::Quotient::Node(Some(deduced_op)));
}

fn explicate_hold(state: InState, context: &Context) {}

// initially setup a node that hasn't yet been read
// distinct from explication in that we have no request to fulfill
// panics if no node can be found during any step of the recursion
fn explicate_node(location: Location, context: &mut Context) {
    let current = context.extract_node(&location.funclet, &location.node);
    let result = match current {
        expir::Node::Phi { index } => read_phi_node(&location, index.unwrap(), context),
        expir::Node::AllocTemporary { .. } => {
            context.add_available_operation(location.node.clone(), OpCode::AllocTemporary);
            current
        }
        expir::Node::Drop { .. } => {
            todo!()
        }
        expir::Node::StaticSubAlloc { .. } => {
            todo!()
        }
        expir::Node::StaticAlloc { .. } => {
            todo!()
        }
        expir::Node::StaticDealloc { .. } => {
            todo!()
        }
        expir::Node::ReadRef { .. } => {
            // dbg!(&context.program());
            todo!()
        }
        expir::Node::BorrowRef { .. } => {
            todo!()
        }
        expir::Node::WriteRef { .. } => {
            todo!()
        }
        expir::Node::LocalDoBuiltin {
            operation,
            inputs,
            outputs,
        } => explicate_local_do_builtin(&location, operation, inputs, outputs, context),
        expir::Node::LocalDoExternal { .. } => {
            todo!()
        }
        expir::Node::LocalCopy { .. } => {
            todo!()
        }
        expir::Node::BeginEncoding { .. } => {
            todo!()
        }
        expir::Node::EncodeDoExternal { .. } => {
            todo!()
        }
        expir::Node::EncodeCopy { .. } => {
            todo!()
        }
        expir::Node::Submit { .. } => {
            todo!()
        }
        expir::Node::SyncFence { .. } => {
            todo!()
        }
        expir::Node::InlineJoin { .. } => {
            todo!()
        }
        expir::Node::SerializedJoin { .. } => {
            todo!()
        }
        expir::Node::DefaultJoin => {
            todo!()
        }
        expir::Node::PromiseCaptures { .. } => {
            todo!()
        }
        expir::Node::FulfillCaptures { .. } => {
            todo!()
        }
        _ => unreachable!("Unsupported node for explication {:?}", location),
    };
    context.replace_node_hole(&location.funclet, &location.node, result);
}

fn explicate_tail_edge(state: InState, context: &Context) -> FuncletOutState {
    match context.get_tail_edge(funclet) {
        None => {
            todo!()
        }
        Some(tail_edge) => match tail_edge {
            expir::TailEdge::DebugHole { .. } => {}
            expir::TailEdge::Return { .. } => {}
            expir::TailEdge::Jump { .. } => {}
            expir::TailEdge::ScheduleCall { .. } => {}
            expir::TailEdge::ScheduleSelect { .. } => {}
            expir::TailEdge::ScheduleCallYield { .. } => {}
        },
    }
    todo!()
}

fn explicate_command(mut state: InState, context: &StaticContext) -> Option<FuncletOutState> {
    match context.get_command(&state.funclet, &location.node) {
        expir::Command::Hole => {
            state.add_explication_hole();
            state.get_latest_scope_mut().advance_node();
            explicate_command(state, context)
        }
        expir::Command::Node(n) => explicate_node(state, context),
        expir::Command::TailEdge(_) => explicate_tail_edge(state, context),
        expir::Command::ExplicationHole => {
            unreachable!("Should not be attempting to explicate an explication hole as a command")
        }
    }
}

pub fn explicate_funclet(
    kind: ir::FuncletKind,
    header: expir::FuncletHeader,
    state: InState,
    context: &StaticContext,
) -> expir::Funclet {
    match explicate_command(state, context) {
        None => panic!(
            "No explication solution found for {:?}",
            state.get_latest_scope().funclet
        ),
        Some(result) =>  {
            assert!(!result.has_fills_remaining());
            expir::Funclet {
            kind,
            header,
            commands: result.drain_commands(),
        }},
    }
}
