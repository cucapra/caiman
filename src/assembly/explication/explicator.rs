use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    CommandId, ExternalFunctionId, FFIType, FuncletId, FunctionClassId, RemoteNodeId,
    StorageTypeId, TypeId,
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

fn read_phi_node(location: Location, index: usize, context: &mut Context) {
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
    context.add_instantiation(location.command, remotes, place.cloned());
}

// the function that handles "ok, I have an output, now figure out how to get there"
fn read_output(
    location: Location,
    operation: &Hole<ast::Quotient>,
    inputs: &Hole<Vec<Hole<CommandId>>>,
    output: &CommandId,
    context: &mut Context,
) {
}

fn explicate_local_do_builtin(
    location: Location,
    operation: &Hole<ast::Quotient>,
    inputs: &Hole<Vec<Hole<CommandId>>>,
    outputs: &Hole<Vec<Hole<CommandId>>>,
    context: &mut Context,
) {
    let remote_loc = operation.as_ref().map(|q| quotient_id(q));
    let mut to_instantiate = Vec::new();
    let mut available = false;
    match outputs {
        None => {}
        Some(v) => {
            for output in v {
                match output {
                    None => {}
                    Some(n) => {
                        // if there's an allocation we're using that we don't yet know
                        // figure out what it instantiates
                        to_instantiate
                            .push((n, context.get_spec_instantiation(&location.funclet, n)));
                    }
                }
            }
        }
    };

    // if there's stuff left to explicate, make this available and return
    if available {
        context.add_available_operation(location.command, OpCode::LocalDoBuiltin);
    }
}

// initially setup a node that hasn't yet been read
// distinct from explication in that we have no request to fulfill
// panics if no node can be found during any step of the recursion
fn explicate_node(location: Location, current: ast::Node, context: &mut Context) {
    match current {
        ast::Node::Phi { index } => {
            read_phi_node(location, index.unwrap(), context);
        }
        ast::Node::AllocTemporary { .. } => {
            context.add_available_operation(location.command, OpCode::AllocTemporary)
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
        } => explicate_local_do_builtin(location, operation, inputs, outputs, context),
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
}

pub fn explicate_command(funclet: ast::FuncletId, command: ast::CommandId, context: &mut Context) {
    let location = Location { funclet, command };
    match context.get_command(&location.funclet, &location.command) {
        ast::Command::Hole => {
            context.add_explication_hole(location.command.clone())
        }
        ast::Command::Node(n) => {
            explicate_node(
                location,
                context.extract_node(&location.funclet, &location.command),
                context,
            );
        }
        ast::Command::TailEdge(_) => {
            todo!()
        }
        ast::Command::ExplicationHole => {
            unreachable!("Should not be attempting to explicate an explication hole as a command")
        }
    }
}
