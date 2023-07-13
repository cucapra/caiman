mod context;
mod explicator;
mod util;

use crate::assembly::ast;
use context::Context;
use crate::assembly::explication::util::reject_hole;

fn explicate_commands(funclet: &ast::FuncletId, context: &mut Context) {
    for node in context.static_node_ids(funclet) {
        match reject_hole(context.get_node(funclet, &node)) {
            ast::Node::AllocTemporary { .. } => {}
            ast::Node::Drop { .. } => {}
            ast::Node::StaticSubAlloc { .. } => {}
            ast::Node::StaticAlloc { .. } => {}
            ast::Node::StaticDealloc { .. } => {}
            ast::Node::ReadRef { .. } => {}
            ast::Node::BorrowRef { .. } => {}
            ast::Node::WriteRef { .. } => {}
            ast::Node::LocalDoBuiltin { operation, inputs, outputs } => {

            }
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
            _ => {}
        }
    }
}

fn explicate_funclets(context: &mut Context) {
    for funclet in context.static_schedule_funclet_ids() {
        explicate_commands(&funclet, context);
    }
}

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(mut program: &mut ast::Program) {
    let mut context = Context::new(&mut program);
    context.corrections();
    context.initialize(); // setup the context with initial "raw" information
    explicate_funclets(&mut context);
}
