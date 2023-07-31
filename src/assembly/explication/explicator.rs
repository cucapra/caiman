use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::ir::Place;
use crate::assembly::explication::context::Context;
use crate::assembly::explication::util::*;
use crate::{assembly, frontend, ir};

fn read_tag (tag: &ast::Tag) {

}

fn explicate_phi_node(
    node: &ast::NodeId,
    funclet: &ast::FuncletId,
    index: usize,
    context: &mut Context
) {
    //context.add_instantiation();
}

// find, explicate, and return the id of an available node
// panics if no node can be found
// heavily recursive
pub fn explicate_node (
    node: &ast::NodeId,
    funclet: &ast::FuncletId,
    context: &mut Context,
) {
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
        _ => unreachable!("Unsupported node for explication {:?}", node)
    };
}