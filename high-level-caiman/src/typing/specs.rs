use std::collections::{HashMap, HashSet};

use crate::{
    enum_cast,
    error::{type_error, Info, LocalError},
    parse::ast::{Binop, DataType, SchedLiteral, SpecExpr, SpecLiteral, SpecStmt, SpecTerm},
};

use super::{
    binop_to_contraints, types::DTypeConstraint, Signature, SpecInfo, TypedBinop,
    UnresolvedTypedBinop,
};

/// Collects all names defined in a given spec, including inputs and outputs
fn collect_spec_names(stmts: &Vec<SpecStmt>, ctx: &SpecInfo) -> HashSet<String> {
    let mut res = HashSet::new();
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, .. } => {
                for (name, _) in lhs {
                    assert!(!res.contains(name), "Duplicate node: {name}");
                    res.insert(name.clone());
                }
            }
            SpecStmt::Returns(..) => (),
        }
    }
    for (name, _) in &ctx.sig.input {
        assert!(!res.contains(name), "Duplicate node: {name}");
        res.insert(name.clone());
    }
    res
}

/// Converts a spec literal to a string.
pub fn spec_lit_to_str(s: &SpecLiteral) -> String {
    match s {
        SpecLiteral::Int(i) => i.clone(),
        SpecLiteral::Bool(b) => b.to_string(),
        SpecLiteral::Float(f) => format!("{f}f"),
        _ => todo!(),
    }
}

/// Converts a sched literal to a string.
/// # Panics
/// Panics if the literal is not a sched literal.
#[must_use]
pub fn sched_lit_to_str(s: &SchedLiteral) -> String {
    match s {
        SchedLiteral::Int(i) => i.clone(),
        SchedLiteral::Bool(b) => b.to_string(),
        SchedLiteral::Float(f) => format!("{f}f"),
        _ => todo!(),
    }
}

/// Collects types and nodes for a given assignment `lhs :- function(args)`.
fn collect_spec_assign_call(
    lhs: &[(String, Option<DataType>)],
    function: &SpecExpr,
    args: &[SpecExpr],
    ctx: &mut SpecInfo,
    signatures: &HashMap<String, Signature>,
    info: Info,
) -> Result<(), LocalError> {
    if let SpecExpr::Term(SpecTerm::Var {
        name: func_name, ..
    }) = function
    {
        let input_types = signatures.get(func_name).unwrap().input.clone();
        let output_types = signatures.get(func_name).unwrap().output.clone();
        #[allow(clippy::needless_collect)]
        let arg_nodes: Vec<_> = args
            .iter()
            .map(|arg| {
                let t = enum_cast!(SpecExpr::Term, arg);
                let name = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                name.to_string()
            })
            .collect();
        if arg_nodes.len() != input_types.len() {
            return Err(type_error(
                info,
                &format!(
                    "Wrong number of arguments to function {func_name}: expected {}, got {}",
                    input_types.len(),
                    arg_nodes.len(),
                ),
            ));
        }
        if lhs.len() != output_types.len() {
            return Err(type_error(
                info,
                &format!(
                    "Wrong number of return values from function {func_name}: expected {}, got {}",
                    output_types.len(),
                    lhs.len(),
                ),
            ));
        }

        for ((name, annot), typ) in lhs.iter().zip(output_types.iter()) {
            if let Some(a) = annot {
                return Err(type_error(
                    info,
                    &format!("Annotation of {name} conflicts with return type of {func_name}",),
                ));
            }
            ctx.add_dtype_constraint(name, typ.clone(), info)?;
        }
        for (arg_name, arg_type) in arg_nodes.iter().zip(input_types.iter()) {
            ctx.add_dtype_constraint(arg_name, arg_type.clone(), info)?;
        }
        Ok(())
    } else {
        panic!("Not lowered")
    }
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- t`
/// # Returns
/// `true` if the collection failed and should be retried at the next iteration.
fn collect_spec_assign_term(
    t: &SpecTerm,
    lhs: &[(String, Option<DataType>)],
    ctx: &mut SpecInfo,
    signatures: &HashMap<String, Signature>,
) -> Result<bool, LocalError> {
    match t {
        SpecTerm::Lit { lit, info } => {
            if let Some(annot) = lhs[0].1.as_ref() {
                ctx.add_dtype_constraint(&lhs[0].0, annot.clone(), *info)?;
            } else {
                ctx.add_constraint(
                    &lhs[0].0,
                    match lit {
                        SpecLiteral::Int(_) => DTypeConstraint::Int(None),
                        SpecLiteral::Bool(_) => DTypeConstraint::Bool,
                        SpecLiteral::Float(_) => DTypeConstraint::Float(None),
                        _ => todo!(),
                    },
                    *info,
                )?;
            }
            Ok(false)
        }
        SpecTerm::Var { .. } => todo!(),
        SpecTerm::Call {
            function,
            args,
            info,
            ..
        } => {
            collect_spec_assign_call(lhs, function, args, ctx, signatures, *info)?;
            Ok(false)
        }
    }
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- if_true if guard if_false`.
///
/// Returns `true` if the collection failed and should be retried at the next iteration.
///
/// # Panics
/// Panics if the statement is not lowered or it uses a variable that is
/// undefined (i.e. not present in `names`).
fn collect_spec_assign_if(
    lhs: &[(String, Option<DataType>)],
    if_true: &SpecExpr,
    if_false: &SpecExpr,
    guard: &SpecExpr,
    ctx: &mut SpecInfo,
    info: Info,
) -> Result<bool, LocalError> {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
        SpecExpr::Term(SpecTerm::Var {
            name: guard,
            info: g_info,
        }),
    ) = (if_true, if_false, guard)
    {
        ctx.add_dtype_constraint(guard, DataType::Bool, *g_info)?;
        ctx.add_var_equiv(name1, name2, info)?;
        ctx.add_var_equiv(&lhs[0].0, name1, info)?;
        if let Some(t) = lhs[0].1.as_ref() {
            ctx.add_dtype_constraint(&lhs[0].0, t.clone(), info)?;
        }
    } else {
        panic!("Not lowered")
    }
    Ok(false)
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- op_l op op_r`.
///
/// Returns `true` if the collection failed and should be retried at the next iteration.
///
/// # Panics
/// Panics if the statement is not lowered or it uses a variable that is
/// undefined (i.e. not present in `names`).
fn collect_spec_assign_bop(
    op_l: &SpecExpr,
    op_r: &SpecExpr,
    op: Binop,
    externs: &mut HashSet<UnresolvedTypedBinop>,
    lhs: &[(String, Option<DataType>)],
    ctx: &mut SpecInfo,
    info: Info,
) -> Result<bool, LocalError> {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
    ) = (op_l, op_r)
    {
        let (left_constraint, right_constraint, ret_constraint) =
            binop_to_contraints(op, &mut ctx.env);
        ctx.add_raw_constraint(&lhs[0].0, &ret_constraint, info)?;
        if let Some(annot) = &lhs[0].1 {
            ctx.add_dtype_constraint(&lhs[0].0, annot.clone(), info)?;
        }
        ctx.add_raw_constraint(name1, &left_constraint, info)?;
        ctx.add_raw_constraint(name2, &right_constraint, info)?;
        externs.insert(UnresolvedTypedBinop {
            op,
            op_l: name1.clone(),
            op_r: name2.clone(),
            ret: lhs[0].0.clone(),
        });
    } else {
        panic!("Not lowered")
    }
    Ok(false)
}

/// Resolves all types for defined variables in a given spec.
fn resolve_types(ctx: &mut SpecInfo, names: &HashSet<String>) -> Result<(), LocalError> {
    for name in names {
        match ctx.env.get_type(name) {
            Some(c) => {
                let dt = DTypeConstraint::try_from(c.clone()).map_err(|e| {
                    type_error(
                        ctx.info,
                        &format!("Failed to resolve type of variable {name}: {e}"),
                    )
                })?;
                ctx.types.insert(name.clone(), dt.into());
            }
            None => {
                return Err(type_error(
                    ctx.info,
                    &format!("Undefined variable {name} in spec",),
                ))
            }
        }
    }
    Ok(())
}

fn collect_spec_sig(ctx: &mut SpecInfo) -> Result<(), LocalError> {
    let info = ctx.info;
    for (arg, typ) in ctx.sig.input.clone() {
        ctx.add_dtype_constraint(&arg, typ, info)?;
    }
    Ok(())
}

/// Collects all extern operations used in a given spec and collects all types
/// of variables used in the spec.
/// # Arguments
/// * `stmts` - the statements to scan
/// * `externs` - a set of all extern operations used in `stmts`. This is updated
/// as we scan `stmts` for all new extern operations.
/// * `types` - a map from variable names to their types. This is updated as
/// we scan `stmts` for all new variables.
/// * `signatures` - a map from spec names to their signatures
pub(super) fn collect_spec(
    stmts: &Vec<SpecStmt>,
    ctx: &mut SpecInfo,
    signatures: &HashMap<String, Signature>,
) -> Result<HashSet<TypedBinop>, LocalError> {
    let mut unresolved_externs = HashSet::new();
    let names = collect_spec_names(stmts, ctx);
    collect_spec_sig(ctx)?;
    let mut skipped = true;
    // specs are unordered, so iterate until no change.
    while skipped {
        skipped = false;
        for stmt in stmts {
            match stmt {
                SpecStmt::Assign { lhs, rhs, .. } => match rhs {
                    SpecExpr::Term(t) => match collect_spec_assign_term(t, lhs, ctx, signatures) {
                        Ok(true) => {
                            skipped = true;
                            continue;
                        }
                        Ok(false) => (),
                        Err(e) => return Err(e),
                    },
                    SpecExpr::Conditional {
                        if_true,
                        guard,
                        if_false,
                        info,
                    } => match collect_spec_assign_if(lhs, if_true, if_false, guard, ctx, *info) {
                        Ok(true) => {
                            skipped = true;
                            continue;
                        }
                        Ok(false) => (),
                        Err(e) => return Err(e),
                    },
                    SpecExpr::Binop {
                        op,
                        lhs: op_l,
                        rhs: op_r,
                        info,
                    } => {
                        match collect_spec_assign_bop(
                            op_l,
                            op_r,
                            *op,
                            &mut unresolved_externs,
                            lhs,
                            ctx,
                            *info,
                        ) {
                            Ok(true) => {
                                skipped = true;
                                continue;
                            }
                            Ok(false) => (),
                            Err(e) => return Err(e),
                        }
                    }
                    SpecExpr::Uop { .. } => todo!(),
                },
                SpecStmt::Returns(..) => (),
            }
        }
    }
    resolve_types(ctx, &names)?;
    Ok(unresolved_externs
        .into_iter()
        .map(|u| TypedBinop {
            op: u.op,
            op_l: ctx.types[&u.op_l].clone(),
            op_r: ctx.types[&u.op_r].clone(),
            ret: ctx.types[&u.ret].clone(),
        })
        .collect::<HashSet<_>>())
}
