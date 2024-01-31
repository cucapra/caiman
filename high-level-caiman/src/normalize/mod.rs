mod sched_rename;

use crate::parse::ast::{Program, TopLevel};

use self::sched_rename::rename_vars;
pub use sched_rename::original_name;

/// Normalizes the AST by renaming schedule variables and flattening nested
/// expressions.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn normalize_ast(mut p: Program) -> Program {
    for decl in &mut p {
        if let TopLevel::SchedulingFunc {
            statements, input, ..
        } = decl
        {
            rename_vars(statements, input);
        }
    }
    p
}
