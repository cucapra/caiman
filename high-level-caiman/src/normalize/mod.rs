mod flatten_expr;
mod if_to_seq;
mod record_expansion;
mod sched_rename;
mod yields;

use crate::{
    error::LocalError,
    parse::ast::{ClassMembers, Program, SpecFunclet, TopLevel},
};

use self::{
    flatten_expr::{flatten_schedule, flatten_spec},
    if_to_seq::final_if_to_seq,
    sched_rename::rename_vars,
    yields::CallGraph,
};

/// Normalizes the AST by renaming schedule variables, flattening nested
/// expressions, converting conditional returns to sequences, and inserting
/// yields.
/// # Errors
/// If there is a type error in the AST caught during normalization.
#[allow(clippy::module_name_repetitions)]
pub fn normalize_ast(mut p: Program) -> Result<Program, LocalError> {
    for decl in &mut p {
        match decl {
            TopLevel::SchedulingFunc {
                statements, input, ..
            } => {
                let stmts = final_if_to_seq(std::mem::take(statements))?;
                *statements = flatten_schedule(stmts);
                rename_vars(statements, input);
            }
            TopLevel::FunctionClass { members, .. } => {
                for member in members {
                    match member {
                        ClassMembers::ValueFunclet(SpecFunclet { statements, .. })
                        | ClassMembers::TimelineFunclet(SpecFunclet { statements, .. })
                        | ClassMembers::SpatialFunclet(SpecFunclet { statements, .. }) => {
                            let stmts = std::mem::take(statements);
                            *statements = flatten_spec(stmts);
                        }
                        ClassMembers::Extern { .. } => {}
                    }
                }
            }
            _ => (),
        }
    }
    let mut cg = CallGraph::new(&mut p);
    cg.insert_yields();
    Ok(p)
}

/// Performs a second pass of normalization on the AST that require type information.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn post_typecheck_norm(mut p: Program) -> Program {
    for decl in &mut p {
        if let TopLevel::SchedulingFunc { input, output, .. } = decl {
            record_expansion::expand_record_io(input, output);
        }
    }
    p
}
