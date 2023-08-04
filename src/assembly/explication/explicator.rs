use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::explication::context::Context;
use crate::assembly::explication::util::*;
use crate::ir::Place;
use crate::{assembly, frontend, ir};

fn tag_quotient(tag: &ast::Tag) -> &Hole<RemoteNodeId> {
    match &tag.quot {
        ast::Quotient::None => &None,
        ast::Quotient::Node(n) => n,
        ast::Quotient::Input(n) => n,
        ast::Quotient::Output(n) => n,
    }
}

fn explicate_phi_node(
    node: &ast::NodeId,
    funclet: &ast::FuncletId,
    index: usize,
    context: &mut Context,
) {
    let current_funclet = context.get_funclet(funclet);
    let argument = current_funclet.header.args.get(index).unwrap_or_else(|| {
        panic!(
            "Index {} out of bounds for header in funclet {:?}",
            index, &funclet
        )
    });
    let mut remotes = Vec::new();
    for tag in &argument.tags {
        let quotient = tag_quotient(tag);
        match quotient {
            None => {}
            Some(remote) => remotes.push(remote.clone())
        }
    }
    let place = context.get_type_place(&argument.typ);
    context.add_instantiation(node.clone(), remotes, place.cloned());
}

// find, explicate, and return the id of an available node
// panics if no node can be found
// heavily recursive
pub fn explicate_node(funclet: &ast::FuncletId, node: &ast::NodeId, context: &mut Context) {
    match context.get_node(funclet, node).clone() {
        ast::Node::Phi { index } => {
            explicate_phi_node(node, funclet, index.unwrap(), context);
        }
        ast::Node::AllocTemporary { .. } => {}
        ast::Node::Drop { .. } => {}
        ast::Node::StaticSubAlloc { .. } => {}
        ast::Node::StaticAlloc { .. } => {}
        ast::Node::StaticDealloc { .. } => {}
        ast::Node::ReadRef { .. } => {}
        ast::Node::BorrowRef { .. } => {}
        ast::Node::WriteRef { .. } => {}
        ast::Node::LocalDoBuiltin { .. } => {}
        ast::Node::LocalDoExternal { .. } => {}
        ast::Node::LocalCopy { .. } => {}
        ast::Node::BeginEncoding { .. } => {}
        ast::Node::EncodeDoExternal { .. } => {}
        ast::Node::EncodeCopy { .. } => {}
        ast::Node::Submit { .. } => {}
        ast::Node::SyncFence { .. } => {}
        ast::Node::InlineJoin { .. } => {}
        ast::Node::SerializedJoin { .. } => {}
        ast::Node::DefaultJoin => {}
        ast::Node::PromiseCaptures { .. } => {}
        ast::Node::FulfillCaptures { .. } => {}
        _ => unreachable!("Unsupported node for explication {:?}", node),
    };
}
