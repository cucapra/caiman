mod context;
mod explicator;
mod explicator_macros;
mod util;

use crate::assembly::ast;
use crate::assembly::explication::util::reject_hole;
use context::Context;

fn explicate_commands(funclet: &ast::FuncletId, context: &mut Context) -> bool {
    context.enter_funclet(funclet.clone());
    for node in context.static_command_ids(funclet) {
        // we need to clone so we can potentially update the node in the context
        explicator::explicate_command(funclet.clone(), node, context);
    }
    dbg!(&context);
    todo!();
    // explicator::explicate_tail_edge(&funclet, context);
    // context.exit_funclet()
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
pub fn explicate(program: &mut ast::Program) {
    let mut context = Context::new(program);
    explicate_funclets(&mut context);

    // dbg!(&context);
    todo!()
}
