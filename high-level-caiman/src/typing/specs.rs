use std::collections::{HashMap, HashSet};

use crate::{
    enum_cast,
    lower::{binop_to_str, tuple_id},
    parse::ast::{
        Binop, DataType, FloatSize, IntSize, SchedLiteral, SpecExpr, SpecLiteral, SpecStmt,
        SpecTerm,
    },
};

use super::{op_output_type, SpecInfo, SpecNode, TypedBinop};

/// Collects all names defined in a given spec.
fn collect_spec_names(stmts: &Vec<SpecStmt>) -> HashSet<String> {
    let mut res = HashSet::new();
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, .. } => {
                for (name, _) in lhs {
                    res.insert(name.clone());
                }
            }
            SpecStmt::Returns(..) => (),
        }
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
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) {
    if let SpecExpr::Term(SpecTerm::Var {
        name: func_name, ..
    }) = function
    {
        #[allow(clippy::needless_collect)]
        let arg_nodes: Vec<_> = args
            .iter()
            .map(|arg| {
                let t = enum_cast!(SpecExpr::Term, arg);
                let name = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                name.to_string()
            })
            .collect();
        let mut nodes = vec![];
        let tuple_name = tuple_id(&lhs.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>());
        ctx.nodes.insert(
            tuple_name.clone(),
            SpecNode::Op(func_name.clone(), arg_nodes),
        );
        for (idx, ((name, annot), typ)) in lhs
            .iter()
            .zip(signatures.get(func_name).unwrap().1.iter())
            .enumerate()
        {
            if let Some(a) = annot {
                assert_eq!(a, typ);
            }
            ctx.types.insert(name.clone(), typ.clone());
            nodes.push((name.clone(), SpecNode::Extract(tuple_name.clone(), idx)));
        }
        for (name, node) in nodes {
            ctx.nodes.insert(name, node);
        }
    } else {
        panic!("Not lowered")
    }
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- t`
fn collect_spec_assign_term(
    t: &SpecTerm,
    lhs: &[(String, Option<DataType>)],
    ctx: &mut SpecInfo,
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) -> bool {
    match t {
        SpecTerm::Lit { lit, .. } => match lit {
            SpecLiteral::Int(_) => {
                ctx.types
                    .insert(lhs[0].0.clone(), DataType::Int(IntSize::I64));
                ctx.nodes
                    .insert(lhs[0].0.clone(), SpecNode::Lit(spec_lit_to_str(lit)));
                false
            }
            SpecLiteral::Bool(_) => {
                ctx.types.insert(lhs[0].0.clone(), DataType::Bool);
                ctx.nodes
                    .insert(lhs[0].0.clone(), SpecNode::Lit(spec_lit_to_str(lit)));
                false
            }
            SpecLiteral::Float(_) => {
                ctx.types
                    .insert(lhs[0].0.clone(), DataType::Float(FloatSize::F64));
                ctx.nodes
                    .insert(lhs[0].0.clone(), SpecNode::Lit(spec_lit_to_str(lit)));
                false
            }
            _ => todo!(),
        },
        SpecTerm::Var { .. } => todo!(),
        SpecTerm::Call { function, args, .. } => {
            collect_spec_assign_call(lhs, function, args, ctx, signatures);
            false
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
    names: &HashSet<String>,
    ctx: &mut SpecInfo,
) -> bool {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
        SpecExpr::Term(SpecTerm::Var { name: guard, .. }),
    ) = (if_true, if_false, guard)
    {
        if !ctx.types.contains_key(guard)
            || !ctx.types.contains_key(name1)
            || !ctx.types.contains_key(name2)
        {
            assert!(names.contains(guard), "Undefined node: {guard}");
            assert!(names.contains(name1), "Undefined node: {name1}");
            assert!(names.contains(name2), "Undefined node: {name2}");
            return true;
        }
        assert_eq!(ctx.types[guard], DataType::Bool);
        assert_eq!(
            ctx.types[name1], ctx.types[name2],
            "Conditional types must be equal"
        );
        ctx.types.insert(lhs[0].0.clone(), ctx.types[name1].clone());
        ctx.nodes.insert_or_remove_if_dup(
            lhs[0].0.clone(),
            SpecNode::Op(
                "if".to_string(),
                vec![guard.clone(), name1.clone(), name2.clone()],
            ),
        );
    } else {
        panic!("Not lowered")
    }
    false
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
    externs: &mut HashSet<TypedBinop>,
    lhs: &[(String, Option<DataType>)],
    names: &HashSet<String>,
    ctx: &mut SpecInfo,
) -> bool {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
    ) = (op_l, op_r)
    {
        if !ctx.types.contains_key(name1) || !ctx.types.contains_key(name2) {
            assert!(names.contains(name1), "Undefined node: {name1}");
            assert!(names.contains(name2), "Undefined node: {name2}");
            return true;
        }
        let ret = op_output_type(op, &ctx.types[name1], &ctx.types[name2]);
        ctx.types.insert(lhs[0].0.clone(), ret.clone());
        ctx.nodes.insert_or_remove_if_dup(
            lhs[0].0.clone(),
            SpecNode::Op(
                binop_to_str(
                    op,
                    &format!("{}", ctx.types[name1]),
                    &format!("{}", ctx.types[name2]),
                ),
                vec![name1.clone(), name2.clone()],
            ),
        );
        externs.insert(TypedBinop {
            op,
            op_l: ctx.types[name1].clone(),
            op_r: ctx.types[name2].clone(),
            ret,
        });
    } else {
        panic!("Not lowered")
    }
    false
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
    externs: &mut HashSet<TypedBinop>,
    ctx: &mut SpecInfo,
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) {
    let names = collect_spec_names(stmts);
    let mut skipped = true;
    // specs are unordered, so iterate until no change.
    while skipped {
        skipped = false;
        for stmt in stmts {
            match stmt {
                SpecStmt::Assign { lhs, rhs, .. } => match rhs {
                    SpecExpr::Term(t) => {
                        if collect_spec_assign_term(t, lhs, ctx, signatures) {
                            skipped = true;
                            continue;
                        }
                    }
                    SpecExpr::Conditional {
                        if_true,
                        guard,
                        if_false,
                        ..
                    } => {
                        if collect_spec_assign_if(lhs, if_true, if_false, guard, &names, ctx) {
                            skipped = true;
                            continue;
                        }
                    }
                    SpecExpr::Binop {
                        op,
                        lhs: op_l,
                        rhs: op_r,
                        ..
                    } => {
                        if collect_spec_assign_bop(op_l, op_r, *op, externs, lhs, &names, ctx) {
                            skipped = true;
                            continue;
                        }
                    }
                    SpecExpr::Uop { .. } => todo!(),
                },
                SpecStmt::Returns(..) => (),
            }
        }
    }
}
