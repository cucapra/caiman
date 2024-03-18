mod context;
pub mod expir;
mod explicator;
mod explicator_macros;
mod util;

pub type Hole<T> = Option<T>;

use crate::ir;
use crate::stable_vec::StableVec;
use context::{InState, StaticContext};

use self::explicator::{explicate_schedule_funclet, lower_spec_funclet};

fn explicate_funclets(context: &StaticContext) -> StableVec<ir::Funclet> {
    context
        .program()
        .funclets
        .iter()
        .map(|(id, funclet)| match funclet.kind {
            ir::FuncletKind::Unknown
            | ir::FuncletKind::Value
            | ir::FuncletKind::Timeline
            | ir::FuncletKind::Spatial => lower_spec_funclet(id, context),
            ir::FuncletKind::ScheduleExplicit => {
                explicate_schedule_funclet(InState::new(id), context)
            }
        })
        .collect()
}

fn explicate_program(program: expir::Program) -> ir::Program {
    let mut context = StaticContext::new(&program);
    let explicated_funclets = explicate_funclets(&context);

    match program {
        expir::Program {
            native_interface,
            types,
            funclets,
            function_classes,
            pipelines,
        } => ir::Program {
            native_interface,
            types,
            funclets: explicated_funclets,
            function_classes,
            pipelines,
        },
    }
}

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(
    definition: crate::frontend::ExplicationDefinition,
) -> crate::frontend::Definition {
    // dbg!(&definition);
    // todo!();
    match definition {
        crate::frontend::ExplicationDefinition { version, program } => {
            let ir_program = explicate_program(program);
            crate::frontend::Definition {
                version,
                program: ir_program,
            }
        }
    }
}
