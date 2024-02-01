mod context;
mod explicator;
mod explicator_macros;
mod util;

use crate::assembly::ast;
use crate::assembly::explication::util::reject_hole;
use context::{InState, StaticContext};

fn explicate_funclets(context: StaticContext) -> Vec<ast::Declaration> {
    for funclet in context.schedule_funclet_ids() {
        let state = InState::new(funclet);
        explicator::explicate_funclet(state, &context);
    }
}

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(mut program: ast::Program) -> ast::Program {
    give_names(&mut program);
    explicate_funclets(StaticContext::new(program));

    // dbg!(&context);
    todo!()
}
