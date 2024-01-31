use std::collections::HashMap;

use regex::Regex;

use crate::parse::ast::{
    ArgsOrEnc, FullType, SchedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm,
};

/// Returns the internal name of a variable, given its original name and id.
fn internal_name(name: &str, id: u64) -> String {
    format!("{name}_{id}")
}

/// Returns the original name of a variable, given its internal name.
/// For example, `original_name("x_0")` returns `"x"`. If the
/// name is not an internal name, it is returned unchanged.
/// # Panics
/// Regex compilation errors
#[must_use]
pub fn original_name(name: &str) -> String {
    if name.contains(Regex::new(r"_[0-9]+$").unwrap().as_str()) {
        name.rfind('_').map(|i| name[..i].to_string()).unwrap()
    } else {
        name.to_string()
    }
}

/// Recursive helper to `rename_vars`.
fn rename_vars_rec<'a, T: Iterator<Item = &'a mut SchedStmt>>(
    stmts: T,
    latest_names: &mut HashMap<String, u64>,
    mut cur_names: HashMap<String, u64>,
) {
    for s in stmts {
        match s {
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
                *lhs = get_cur_name(lhs, &cur_names);
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
            SchedStmt::Call(_, SchedFuncCall { args, .. }) => {
                if let ArgsOrEnc::Args(args) = &mut **args {
                    for arg in args {
                        rename_expr_uses(arg, &cur_names);
                    }
                }
            }
            SchedStmt::Hole(_) => {}
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
        SchedExpr::Term(SchedTerm::Call(_, SchedFuncCall { args, .. })) => {
            if let ArgsOrEnc::Args(args) = &mut **args {
                for arg in args {
                    rename_expr_uses(arg, cur_names);
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
        SchedExpr::Term(SchedTerm::Lit { .. } | SchedTerm::Hole(_)) => {}
        SchedExpr::Conditional { .. } => {
            panic!("Conditional expressions are not supported in schedules")
        }
    }
}
