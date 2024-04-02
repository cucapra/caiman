mod context;
pub mod expir;
mod explicator;
mod explicator_macros;
mod util;

use crate::stable_vec::StableVec;
use crate::{debug_info::DebugInfo, ir};
use context::{InState, StaticContext};
use serde_derive::{Deserialize, Serialize};

use self::explicator::{explicate_schedule_funclet, lower_spec_funclet};

// Explication and frontend AST

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Hole<T> {
    Empty,
    Filled(T),
}

impl<T> Hole<T> {
    pub fn as_ref(&self) -> Hole<&T> {
        match self {
            Hole::Empty => Hole::Empty,
            Hole::Filled(x) => Hole::Filled(x),
        }
    }

    pub fn opt(self) -> Option<T> {
        self.into()
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Hole::Empty => true,
            Hole::Filled(_) => false,
        }
    }

    pub fn is_filled(&self) -> bool {
        !self.is_empty()
    }
}

impl<T> From<Option<T>> for Hole<T> {
    fn from(x: Option<T>) -> Self {
        match x {
            Some(x) => Hole::Filled(x),
            None => Hole::Empty,
        }
    }
}

impl<T> From<Hole<T>> for Option<T> {
    fn from(x: Hole<T>) -> Self {
        match x {
            Hole::Filled(x) => Some(x),
            Hole::Empty => None,
        }
    }
}

impl<T> std::fmt::Display for Hole<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            // don't write "Filled" when it's filled
            Self::Filled(arg0) => write!(f, "{}", arg0),
        }
    }
}

fn explicate_funclets(context: &StaticContext) -> StableVec<ir::Funclet> {
    dbg!(&context);
    todo!();
    context
        .program()
        .funclets
        .iter()
        .map(|(id, funclet)| match funclet.kind {
            ir::FuncletKind::Unknown
            | ir::FuncletKind::Value
            | ir::FuncletKind::Timeline
            | ir::FuncletKind::Spatial => lower_spec_funclet(&id, context),
            ir::FuncletKind::ScheduleExplicit => {
                explicate_schedule_funclet(InState::new(id), context)
            }
        })
        .collect()
}

fn explicate_program(program: expir::Program, debug_info: &DebugInfo) -> ir::Program {
    let mut context = StaticContext::new(&program, debug_info);
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
        crate::frontend::ExplicationDefinition {
            version,
            debug_info,
            program,
        } => {
            let ir_program = explicate_program(program, &debug_info);
            crate::frontend::Definition {
                version,
                debug_info,
                program: ir_program,
            }
        }
    }
}
