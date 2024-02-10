mod flatten_expr;
mod sched_rename;

use crate::parse::ast::{ClassMembers, Program, TopLevel};

use self::{
    flatten_expr::{flatten_schedule, flatten_spec},
    sched_rename::rename_vars,
};
pub use sched_rename::original_name;

/// Normalizes the AST by renaming schedule variables and flattening nested
/// expressions.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn normalize_ast(mut p: Program) -> Program {
    for decl in &mut p {
        match decl {
            TopLevel::SchedulingFunc {
                statements, input, ..
            } => {
                *statements = flatten_schedule(std::mem::take(statements));
                rename_vars(statements, input);
            }
            TopLevel::FunctionClass { members, .. } => {
                for member in members {
                    if let ClassMembers::ValueFunclet { statements, .. } = member {
                        let stmts = std::mem::take(statements);
                        *statements = flatten_spec(stmts);
                    }
                }
            }
            _ => (),
        }
    }
    p
}
