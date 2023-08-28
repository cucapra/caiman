use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::explication::context::Context;
use crate::assembly::explication::context::OpCode;
use crate::assembly::explication::util::*;
use crate::ir::Place;
use crate::{assembly, frontend, ir};

fn quotient_id(quot: &ast::Quotient) -> &Hole<RemoteNodeId> {
    match quot {
        ast::Quotient::None => &None,
        ast::Quotient::Node(n) => n,
        ast::Quotient::Input(n) => n,
        ast::Quotient::Output(n) => n,
    }
}

fn tag_quotient(tag: &ast::Tag) -> &Hole<RemoteNodeId> {
    quotient_id(&tag.quot)
}

fn read_phi_node(location: &Location, index: usize, context: &mut Context) -> ast::Node {
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
    ast::Node::Phi { index: Some(index) }
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
    og_operation: Hole<ast::Quotient>,
    og_inputs: Hole<Vec<Hole<NodeId>>>,
    og_outputs: Hole<Vec<Hole<NodeId>>>,
    context: &mut Context,
) -> ast::Node {
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
        Some(f), Some(n) => {

        }
    }

    let outputs = match og_outputs {
        None => {
            match
        },
        Some(ogo) => {
            let mut result = Vec::new();
            for output in ogo {
                match output {
                    Some(out) => Some(out),
                    None => {},

                }
            }
            result
        }
    };

    // if there's stuff left to explicate, make this available and return
    if available {
        context.add_available_operation(location.node.clone(), OpCode::LocalDoBuiltin);
    }
    let operation = Some(ast::Quotient::Node(Some(deduced_op)));

}

// initially setup a node that hasn't yet been read
// distinct from explication in that we have no request to fulfill
// panics if no node can be found during any step of the recursion
fn explicate_node(location: Location, context: &mut Context) {
    let current = context.extract_node(&location.funclet, &location.node);
    let result = match current {
        ast::Node::Phi { index } => read_phi_node(&location, index.unwrap(), context),
        ast::Node::AllocTemporary { .. } => {
            context.add_available_operation(location.node.clone(), OpCode::AllocTemporary);
            current
        }
        ast::Node::Drop { .. } => {
            todo!()
        }
        ast::Node::StaticSubAlloc { .. } => {
            todo!()
        }
        ast::Node::StaticAlloc { .. } => {
            todo!()
        }
        ast::Node::StaticDealloc { .. } => {
            todo!()
        }
        ast::Node::ReadRef { .. } => {
            // dbg!(&context.program());
            todo!()
        }
        ast::Node::BorrowRef { .. } => {
            todo!()
        }
        ast::Node::WriteRef { .. } => {
            todo!()
        }
        ast::Node::LocalDoBuiltin {
            operation,
            inputs,
            outputs,
        } => explicate_local_do_builtin(&location, operation, inputs, outputs, context),
        ast::Node::LocalDoExternal { .. } => {
            todo!()
        }
        ast::Node::LocalCopy { .. } => {
            todo!()
        }
        ast::Node::BeginEncoding { .. } => {
            todo!()
        }
        ast::Node::EncodeDoExternal { .. } => {
            todo!()
        }
        ast::Node::EncodeCopy { .. } => {
            todo!()
        }
        ast::Node::Submit { .. } => {
            todo!()
        }
        ast::Node::SyncFence { .. } => {
            todo!()
        }
        ast::Node::InlineJoin { .. } => {
            todo!()
        }
        ast::Node::SerializedJoin { .. } => {
            todo!()
        }
        ast::Node::DefaultJoin => {
            todo!()
        }
        ast::Node::PromiseCaptures { .. } => {
            todo!()
        }
        ast::Node::FulfillCaptures { .. } => {
            todo!()
        }
        _ => unreachable!("Unsupported node for explication {:?}", location),
    };
    context.replace_node_hole(&location.funclet, &location.node, result);
}

pub fn explicate_command(funclet: ast::FuncletId, command: ast::NodeId, context: &mut Context) {
    let location = Location {
        funclet,
        node: command,
    };
    match context.get_command(&location.funclet, &location.node) {
        ast::Command::Hole => context.add_explication_hole(location.node.clone()),
        ast::Command::Node(n) => {
            explicate_node(location, context);
        }
        ast::Command::TailEdge(_) => {
            unreachable!("Tail Edges are explicated separately")
        }
        ast::Command::ExplicationHole => {
            unreachable!("Should not be attempting to explicate an explication hole as a command")
        }
    }
}

pub fn explicate_tail_edge(funclet: &ast::FuncletId, context: &mut Context) {
    match context.get_tail_edge(funclet) {
        None => {
            todo!()
        }
        Some(tail_edge) => match tail_edge {
            ast::TailEdge::DebugHole { .. } => {}
            ast::TailEdge::Return { .. } => {}
            ast::TailEdge::Jump { .. } => {}
            ast::TailEdge::ScheduleCall { .. } => {}
            ast::TailEdge::ScheduleSelect { .. } => {}
            ast::TailEdge::ScheduleCallYield { .. } => {}
        },
    }
    todo!()
}
