//! This module contains functions for renaming constants at the source (AST) level
//! of a scheduling function. We rename variables so that variabls in differnt scopes
//! (which are semantically different variables) have different names. This is done
//! so that type deduction, which happens globally at the AST level, can distinguish
//! between variables in different scopes.
//!
//! For example:
//!
//! ```text
//! let x = 1;
//! var v;
//! if x > 0 {
//!    let x = x + 1;
//!    let c = x < 2;
//!    v = x;
//! } else {
//!     let x = x - 1;
//!     let c = x * 2;
//!     v = x;
//! }
//! ```
//! becomes:
//!
//! ```text
//! let x_0 = 1;
//! var v_0;
//! if x_0 > 0 {
//!     let x_1 = x_0 + 1;
//!     let c_0 = x_1 < 2;
//!     v_0 = x_1;
//! } else {
//!     let x_2 = x_0 - 1;
//!     let c_1 = x_2 * 2;
//!     v_0 = x_2;
//! }
//! ```
//!
//! Note that there is only one instance of `v`, since although it's been reassigned,
//! it is still the same variable. Also note that `c` now has two versions, and
//! each version has a differnt base type.
//!
//! ## Why two SSA passes?
//!
//! You may notice that there are two passes of SSA renaming. The first pass
//! (this one) is done at the AST level, and the second pass is done at the
//! HIR level. This pass supports datatype deduction, which happens at the AST
//! level, and the second supports quotient deduction which happens at the HIR
//! level. I decided to do this because when/if we add more complicated datatypes
//! that don't really exist in the caiman IR such as tuples, slices, etc. we'd
//! probably want to type check/deduce them at this level of abstraction and
//! then lower them to something else in the HIR (convert a tuple argument into
//! multiple arguments, for example).
//!
//! We do quotient deduction at the HIR level because at that level, the schedule
//! can be more closely matched to the spec (no nested expressions, no if blocks, etc.)
//! It's also worth noting that the second SSA pass also affects references so that
//! updated references become new variables. This is required because updating
//! a reference may change it's quotient, but it won't change it's datatype,
//! hence why we don't do that here.
//!

use std::collections::HashMap;

use crate::parse::ast::{
    FullType, SchedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm, SpecExpr, SpecLiteral,
    SpecTerm, TemplateArgs,
};

/// Returns the internal name of a variable, given its original name and id.
fn internal_name(name: &str, id: u64) -> String {
    format!("{name}_{id}")
}

/// Recursive helper to `rename_vars`.
/// # Arguments
/// * `stmts` - The statements to rename
/// * `latest_names` - The latest name of each variable, used to generate unique names
/// * `cur_names` - The current name of each variable, used to rename uses
/// * `encode_names` - The current name of each variable in each encoder, used to rename uses
fn rename_vars_rec<'a, T: Iterator<Item = &'a mut SchedStmt>>(
    stmts: T,
    latest_names: &mut HashMap<String, u64>,
    mut cur_names: HashMap<String, u64>,
) {
    for s in stmts {
        match s {
            //TODO: rename timeline operations
            SchedStmt::Decl { lhs, expr, .. } => {
                if let Some(expr) = expr {
                    rename_expr_uses(expr, &cur_names);
                }
                for (name, _) in lhs {
                    *name = get_new_name(name, latest_names, &mut cur_names);
                }
            }
            SchedStmt::Assign { lhs, rhs, .. } => {
                rename_expr_uses(rhs, &cur_names);
                rename_expr_uses(lhs, &cur_names);
            }
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                ..
            } => {
                rename_expr_uses(guard, &cur_names);
                rename_vars_rec(true_block.iter_mut(), latest_names, cur_names.clone());
                rename_vars_rec(false_block.iter_mut(), latest_names, cur_names.clone());
            }
            SchedStmt::Block(_, stmts) => {
                rename_vars_rec(stmts.iter_mut(), latest_names, cur_names.clone());
            }
            SchedStmt::Seq { dests, block, .. } => {
                rename_vars_rec(
                    std::iter::once(&mut **block),
                    latest_names,
                    cur_names.clone(),
                );
                for (dest, _) in dests {
                    *dest = get_new_name(dest, latest_names, &mut cur_names);
                }
            }
            SchedStmt::InEdgeAnnotation { tags, .. }
            | SchedStmt::OutEdgeAnnotation { tags, .. } => {
                for (var, _) in tags {
                    *var = get_cur_name(var, &cur_names);
                }
            }
            SchedStmt::Return(_, e) => {
                rename_expr_uses(e, &cur_names);
            }
            SchedStmt::Call(
                _,
                SchedFuncCall {
                    args, templates, ..
                },
            ) => {
                for arg in args {
                    rename_expr_uses(arg, &cur_names);
                }
                if let Some(TemplateArgs::Vals(vs)) = templates {
                    for v in vs {
                        rename_spec_expr_uses(v, &cur_names);
                    }
                }
            }
            SchedStmt::Hole(_) => {}
            SchedStmt::Encode { encoder, stmt, .. } => {
                // TODO: support encoder aliasing?
                rename_expr_uses(&mut stmt.rhs, &cur_names);
                *encoder = get_cur_name(encoder, &cur_names);
                // TODO: handle renaming dests for multiple encoders?
            }
        }
    }
}

/// Renames all variables in a scheduling function to their internal names.
/// Internal names are unique, SSA-style names such that no two variables have the same name,
/// and scoping rules are followed. Ie. Redeclaring a variable shadows the previous declaration.
///
/// After this function runs, all constants (non references) will be in SSA form.
pub fn rename_vars(stmts: &mut [SchedStmt], inputs: &mut [(String, Option<FullType>)]) {
    let mut latest_names = HashMap::new();
    let mut cur_names = HashMap::new();
    for (name, _) in inputs {
        *name = get_new_name(name, &mut latest_names, &mut cur_names);
    }
    rename_vars_rec(stmts.iter_mut(), &mut latest_names, cur_names);
}

/// Returns the next internal name of a variable, given its original name.
fn get_new_name(
    name: &str,
    latest_names: &mut HashMap<String, u64>,
    cur_names: &mut HashMap<String, u64>,
) -> String {
    let id = latest_names.entry(name.to_string()).or_insert(0);
    let s = internal_name(name, *id);
    cur_names.insert(name.to_string(), *id);
    *id += 1;
    s
}

/// Returns the current internal name of a variable, given its original name.
fn get_cur_name(name: &str, cur_names: &HashMap<String, u64>) -> String {
    cur_names
        .get(name)
        .map_or_else(|| name.to_string(), |id| internal_name(name, *id))
}

/// Renames all uses of variables in an expression to their current internal names.
fn rename_expr_uses(expr: &mut SchedExpr, cur_names: &HashMap<String, u64>) {
    match expr {
        SchedExpr::Term(SchedTerm::Var { name, .. }) => {
            *name = get_cur_name(name, cur_names);
        }
        SchedExpr::Binop { lhs, rhs, .. } => {
            rename_expr_uses(lhs, cur_names);
            rename_expr_uses(rhs, cur_names);
        }
        SchedExpr::Uop { expr, .. } => {
            rename_expr_uses(expr, cur_names);
        }
        SchedExpr::Term(SchedTerm::Call(
            _,
            SchedFuncCall {
                args, templates, ..
            },
        )) => {
            for arg in args {
                rename_expr_uses(arg, cur_names);
            }
            if let Some(TemplateArgs::Vals(templates)) = templates {
                for t in templates {
                    rename_spec_expr_uses(t, cur_names);
                }
            }
        }
        SchedExpr::Term(SchedTerm::Lit {
            lit: SchedLiteral::Tuple(e) | SchedLiteral::Array(e),
            ..
        }) => {
            for e in e {
                rename_expr_uses(e, cur_names);
            }
        }
        SchedExpr::Term(
            SchedTerm::Lit { .. } | SchedTerm::Hole(_) | SchedTerm::EncodeBegin { .. },
        ) => {}
        SchedExpr::Term(SchedTerm::TimelineOperation { arg, .. }) => {
            rename_expr_uses(arg, cur_names);
        }
        SchedExpr::Conditional { .. } => {
            panic!("Conditional expressions are not supported in schedules")
        }
    }
}

/// Renames all uses of variables in a spec expression to their current internal names.
fn rename_spec_expr_uses(expr: &mut SpecExpr, cur_names: &HashMap<String, u64>) {
    match expr {
        SpecExpr::Term(SpecTerm::Var { name, .. }) => {
            *name = get_cur_name(name, cur_names);
        }
        SpecExpr::Binop { lhs, rhs, .. } => {
            rename_spec_expr_uses(lhs, cur_names);
            rename_spec_expr_uses(rhs, cur_names);
        }
        SpecExpr::Uop { expr, .. } => {
            rename_spec_expr_uses(expr, cur_names);
        }
        SpecExpr::Term(SpecTerm::Lit {
            lit: SpecLiteral::Tuple(e) | SpecLiteral::Array(e),
            ..
        }) => {
            for e in e {
                rename_spec_expr_uses(e, cur_names);
            }
        }
        SpecExpr::Conditional { .. } => {
            panic!("Conditional expressions are not supported in templates")
        }
        SpecExpr::Term(SpecTerm::Call { .. }) => {
            panic!("Function calls are not supported in templates")
        }
        SpecExpr::Term(SpecTerm::Lit { .. }) => (),
    }
}
