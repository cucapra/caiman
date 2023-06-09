mod context;
mod corrections;
mod explicator;
mod util;

use crate::assembly::ast;
use context::Context;

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(mut program: &mut ast::Program) {
    let mut context = Context::new(&mut program);
    corrections::correct(&mut context);
}
