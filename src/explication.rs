mod context;
mod explicator_macros;
mod explicator;
mod util;
pub mod expir;

pub type Hole<T> = Option<T>;

use context::{InState, StaticContext};
use crate::stable_vec::StableVec;
use crate::ir;

fn explicate_funclets(context: &StaticContext) -> StableVec<ir::Funclet> {
    todo!()
}

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(program: crate::frontend::ExplicationDefinition) -> crate::frontend::Definition {
    dbg!(&program);
    // explicate_funclets(StaticContext::new(program));

    // dbg!(&context);
    todo!()
}
