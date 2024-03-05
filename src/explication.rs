// mod context;
// mod explicator;
// mod util;
pub mod expir;

pub type Hole<T> = Option<T>;

// use context::{InState, StaticContext};

// fn explicate_funclets(context: StaticContext) -> StableVec<Funclet> {
//     context.program().declarations.iter().map(|decleration| match decleration {
//         expir::Funclet(funclet) => {
//             let state = InState::new(funclet.header.name.clone());
    
//             expir::Declaration::Funclet(
//                 explicator::explicate_funclet(
//                     funclet.kind.clone(),
//                     funclet.header.clone(),
//                     state,
//                     &context,
//                 )
//             )
//         },
//         d => d
//     }).collect()
// }

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(mut program: expir::Program) -> crate::frontend::Definition {
    // explicate_funclets(StaticContext::new(program));

    // dbg!(&context);
    todo!()
}
