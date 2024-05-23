use std::collections::HashMap;

use crate::{
    enum_cast,
    error::{type_error, Info, LocalError},
    parse::ast::{
        Binop, DataType, EncodedCommand, EncodedStmt, FlaggedType, FullType, SchedExpr,
        SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm, TimelineOperation, Uop, WGPUFlags,
    },
};
use std::iter::once;

use super::{
    binop_to_contraints, types::DTypeConstraint, uop_to_contraints, Context, DTypeEnv, Mutability,
};

/// Collects all defined names in a spec and errors if any constants
/// are redefined.
/// # Arguments
/// * `stmts` - The statements to collect names from.
/// * `names` - Mapping of defined names from names to whether they are constant.
pub fn collect_sched_names<'a, T: Iterator<Item = &'a SchedStmt>>(
    stmts: T,
    names: &mut HashMap<String, Mutability>,
) -> Result<(), LocalError> {
    let is_const_to_mut = |is_const| {
        if is_const {
            Mutability::Const
        } else {
            Mutability::Mut
        }
    };
    for s in stmts {
        match s {
            SchedStmt::Seq {
                dests,
                block,
                is_const,
                ..
            } => {
                for (d, _) in dests {
                    names.insert(d.clone(), is_const_to_mut(*is_const));
                }
                collect_sched_names(once(&**block), names)?;
            }
            SchedStmt::Decl {
                lhs,
                is_const,
                info,
                ..
            } => {
                for (d, _) in lhs {
                    if names.contains_key(d) {
                        return Err(type_error(*info, &format!("Variable {d} already defined")));
                    }
                    names.insert(d.clone(), is_const_to_mut(*is_const));
                }
            }
            SchedStmt::If {
                true_block,
                false_block,
                ..
            } => {
                collect_sched_names(true_block.iter(), names)?;
                collect_sched_names(false_block.iter(), names)?;
            }
            SchedStmt::Block(_, stmts) => collect_sched_names(stmts.iter(), names)?,
            SchedStmt::Assign { lhs, .. } => {
                let lhs = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, lhs)
                );
                assert!(names.contains_key(lhs));
            }
            SchedStmt::Encode { stmt, .. } => {
                for (s, _) in &stmt.lhs {
                    names.insert(s.clone(), Mutability::Const);
                }
            }
            SchedStmt::Call(..)
            | SchedStmt::Return(..)
            | SchedStmt::InEdgeAnnotation { .. }
            | SchedStmt::OutEdgeAnnotation { .. }
            | SchedStmt::Hole(_) => (),
        }
    }
    Ok(())
}

/// Collects contraints for a binop.
fn collect_bop(
    dest: &[(String, Option<FullType>)],
    op: Binop,
    lhs: &SchedExpr,
    rhs: &SchedExpr,
    env: &mut DTypeEnv,
    info: Info,
) -> Result<(), LocalError> {
    let lhs_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, lhs)
    );
    let rhs_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, rhs)
    );
    let (left_c, right_c, ret_c) = binop_to_contraints(op, &mut env.env);
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!(
                "{info}: Operator {op:?} has 1 destination, found {}",
                dest.len()
            ),
        ));
    }
    let (dest_name, dest_annot) = &dest[0];
    if let Some(FullType {
        base: Some(anot), ..
    }) = dest_annot
    {
        env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
    }
    env.add_raw_constraint(dest_name, &ret_c, info)?;
    env.add_raw_constraint(lhs_name, &left_c, info)?;
    env.add_raw_constraint(rhs_name, &right_c, info)
}

fn collect_assign_uop(
    dest: &[(String, Option<FullType>)],
    op: Uop,
    expr: &SchedExpr,
    env: &mut DTypeEnv,
    info: Info,
    mutables: &mut HashMap<String, Info>,
) -> Result<(), LocalError> {
    let expr_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, expr)
    );
    if op == Uop::Ref {
        mutables.insert(expr_name.clone(), info);
    }
    let (expr_c, ret_c) = uop_to_contraints(op, &mut env.env);
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!(
                "{info}: Operator {op:?} has 1 destination, found {}",
                dest.len()
            ),
        ));
    }
    let (dest_name, dest_annot) = &dest[0];
    if let Some(FullType {
        base: Some(anot), ..
    }) = dest_annot
    {
        env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
    }
    env.add_raw_constraint(dest_name, &ret_c, info)?;
    env.add_raw_constraint(expr_name, &expr_c, info)
}

/// Collects constraints for a literal assignment.
fn collect_assign_lit(
    dest: &[(String, Option<FullType>)],
    lit: &SchedLiteral,
    env: &mut DTypeEnv,
    info: Info,
) -> Result<(), LocalError> {
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!("{info}: Literal has 1 destination, found {}", dest.len()),
        ));
    }
    let (dest_name, dest_annot) = &dest[0];
    if let Some(FullType {
        base: Some(anot), ..
    }) = dest_annot
    {
        env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
    }
    match lit {
        SchedLiteral::Int(_) => env.add_constraint(dest_name, DTypeConstraint::Int(None), info),
        SchedLiteral::Float(_) => env.add_constraint(dest_name, DTypeConstraint::Float(None), info),
        SchedLiteral::Bool(_) => env.add_constraint(dest_name, DTypeConstraint::Bool, info),
        _ => todo!(),
    }
}

/// Collects constraints for a variable assignment.
fn collect_assign_var(
    dest: &[(String, Option<FullType>)],
    var: &str,
    env: &mut DTypeEnv,
    info: Info,
) -> Result<(), LocalError> {
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!("{info}: Variable has 1 destination, found {}", dest.len()),
        ));
    }
    let (dest_name, dest_annot) = &dest[0];
    if let Some(FullType {
        base: Some(anot), ..
    }) = dest_annot
    {
        env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
    }
    env.add_var_equiv(dest_name, var, info)
}

/// Collects constraints for the body of a sequence.
/// The body of the sequence must be an `if` statement or a block which
/// ends in an `if` statement.
/// # Returns
/// A tuple of the return values of the true and false branches of the `if`.
/// # Panics
/// If the body of the sequence is not an `if` statement or a block which
/// ends in an `if` statement.
fn collect_seq_body(
    ctx: &Context,
    env: &mut DTypeEnv,
    stmt: &SchedStmt,
    mutables: &mut HashMap<String, Info>,
) -> Result<(Vec<String>, Vec<String>), LocalError> {
    match stmt {
        SchedStmt::If {
            true_block,
            false_block,
            ..
        } => {
            let true_rets =
                collect_sched_helper(ctx, env, true_block.iter(), true_block.len(), mutables)?;
            let false_rets =
                collect_sched_helper(ctx, env, false_block.iter(), false_block.len(), mutables)?;
            Ok((true_rets, false_rets))
        }
        x => {
            collect_sched_helper(ctx, env, std::iter::once(x), 1, mutables).map(|x| (x.clone(), x))
        }
    }
    // if let SchedStmt::If {
    //     true_block,
    //     false_block,
    //     ..
    // } = stmt
    // {
    //     let true_rets =
    //         collect_sched_helper(ctx, env, true_block.iter(), true_block.len(), mutables)?;
    //     let false_rets =
    //         collect_sched_helper(ctx, env, false_block.iter(), false_block.len(), mutables)?;
    //     Ok((true_rets, false_rets))
    // } else {
    //     unreachable!()
    // }
}

/// Collects constraints for a sequence of statements.
fn collect_seq(
    ctx: &Context,
    env: &mut DTypeEnv,
    dests: &[(String, Option<FullType>)],
    block: &SchedStmt,
    info: Info,
    mutables: &mut HashMap<String, Info>,
) -> Result<(), LocalError> {
    let (rets_a, rets_b) = collect_seq_body(ctx, env, block, mutables)?;
    if dests.len() != rets_a.len() || dests.len() != rets_b.len() {
        return Err(type_error(
            info,
            &format!(
                "{info}: Expected {} return values from both branches of if, found {} and {}",
                dests.len(),
                rets_a.len(),
                rets_b.len()
            ),
        ));
    }
    for ((d, r_a), r_b) in dests.iter().zip(rets_a.iter()).zip(rets_b.iter()) {
        if let Some(FullType {
            base: Some(anot), ..
        }) = &d.1
        {
            env.add_dtype_constraint(&d.0, anot.base.clone(), info)?;
        }
        env.add_var_equiv(&d.0, r_a, info)?;
        env.add_var_equiv(&d.0, r_b, info)?;
    }
    Ok(())
}

/// Collects constraints for an if statement.
fn collect_if(
    ctx: &Context,
    env: &mut DTypeEnv,
    guard: &SchedExpr,
    true_block: &[SchedStmt],
    false_block: &[SchedStmt],
    info: Info,
    mutables: &mut HashMap<String, Info>,
) -> Result<(), LocalError> {
    let guard_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, guard)
    );
    env.add_constraint(guard_name, DTypeConstraint::Bool, info)?;
    let true_rets = collect_sched_helper(ctx, env, true_block.iter(), true_block.len(), mutables)?;
    let false_rets =
        collect_sched_helper(ctx, env, false_block.iter(), false_block.len(), mutables)?;
    // ifs that return values should have been converted to sequences
    assert!(true_rets.is_empty() && false_rets.is_empty());
    Ok(())
}

/// Collects constraints for a function call.
fn collect_assign_call(
    ctx: &Context,
    env: &mut DTypeEnv,
    dest: &[(String, Option<FullType>)],
    call_info: &SchedFuncCall,
    info: Info,
) -> Result<(), LocalError> {
    let mut arg_names = Vec::new();
    for arg in &call_info.args {
        let arg_name = enum_cast!(
            SchedTerm::Var { name, .. },
            name,
            enum_cast!(SchedExpr::Term, arg)
        );
        arg_names.push(arg_name);
    }
    let fn_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, &*call_info.target)
    );
    let sig = ctx
        .scheds
        .get(fn_name)
        .ok_or_else(|| type_error(info, &format!("{info}: Function {fn_name} not found")))?
        .sig();
    if arg_names.len() != sig.input.len() {
        return Err(type_error(
            info,
            &format!(
                "{info}: Expected {} arguments, found {}",
                sig.input.len(),
                arg_names.len()
            ),
        ));
    }
    if dest.len() != sig.output.len() {
        return Err(type_error(
            info,
            &format!(
                "{info}: Expected {} return values, found {}",
                sig.output.len(),
                dest.len()
            ),
        ));
    }
    for (arg, arg_type) in arg_names.iter().zip(sig.input.iter()) {
        env.add_dtype_constraint(arg, arg_type.base.clone(), info)?;
    }
    for ((dest_name, dest_annot), typ) in dest.iter().zip(sig.output.iter()) {
        if let Some(FullType {
            base: Some(anot), ..
        }) = &dest_annot
        {
            if anot.base != typ.base {
                return Err(type_error(
                    info,
                    &format!(
                        "{info}: Annotation for {dest_name} is incompatible with return type of {fn_name}",
                    ),
                ));
            }
        }
        env.add_dtype_constraint(dest_name, typ.base.clone(), info)?;
    }
    Ok(())
}

/// Collects an empty declaration with no RHS.
fn collect_null_decl(
    env: &mut DTypeEnv,
    dest: &[(String, Option<FullType>)],
    info: Info,
) -> Result<(), LocalError> {
    for (dest_name, dest_annot) in dest {
        if let Some(FullType {
            base: Some(anot), ..
        }) = dest_annot
        {
            env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
        }
    }
    Ok(())
}

/// Collects constraints for the binary dot operator.
fn collect_dot(
    env: &mut DTypeEnv,
    dest: &[(String, Option<FullType>)],
    _lhs: &SchedExpr,
    rhs: &SchedExpr,
    info: Info,
) -> Result<(), LocalError> {
    let rhs_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, rhs)
    );
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!(
                "{info}: Dot operator has 1 destination, found {}",
                dest.len()
            ),
        ));
    }
    let (dest_name, dest_annot) = &dest[0];
    if let Some(FullType {
        base: Some(anot), ..
    }) = dest_annot
    {
        env.add_dtype_constraint(dest_name, anot.base.clone(), info)?;
    }
    env.add_var_equiv(dest_name, rhs_name, info)?;
    env.add_usage(rhs_name, WGPUFlags::MapRead);
    Ok(())
}

/// Collects constraints for a timeline operation (submit or await).
fn collect_timeline_op(
    env: &mut DTypeEnv,
    op: TimelineOperation,
    dest: &[(String, Option<FullType>)],
    arg: &SchedExpr,
    info: Info,
) -> Result<(), LocalError> {
    let arg_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, arg)
    );
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!(
                "{info}: Timeline operation has 1 destination, found {}",
                dest.len()
            ),
        ));
    }
    let dest_name = &dest[0].0;
    match op {
        TimelineOperation::Submit => {
            env.add_dtype_constraint(dest_name, DataType::Fence(None), info)?;
            env.add_dtype_constraint(arg_name, DataType::Encoder(None), info)
        }
        TimelineOperation::Await => {
            // TODO
            env.add_constraint(dest_name, DTypeConstraint::Any, info)?;
            env.add_dtype_constraint(arg_name, DataType::Fence(None), info)
        }
    }
}

/// Collects constraints for an encode begin operation.
fn collect_begin_encode(
    env: &mut DTypeEnv,
    dest: &[(String, Option<FullType>)],
    defs: &[(String, Option<FullType>)],
    info: Info,
) -> Result<(), LocalError> {
    if dest.len() != 1 {
        return Err(type_error(
            info,
            &format!(
                "{info}: EncodeBegin has 1 destination, found {}",
                dest.len()
            ),
        ));
    }
    let dest_name = &dest[0].0;
    env.add_dtype_constraint(dest_name, DataType::Encoder(None), info)?;
    for (def_name, def_annot) in defs {
        if let Some(FullType {
            base: Some(anot), ..
        }) = def_annot
        {
            env.add_dtype_constraint(def_name, anot.base.clone(), info)?;
        }
    }
    Ok(())
}

/// Unifies base types for a schedule.
/// # Returns
/// The type variables of the values being returned to the parent scope.
#[allow(clippy::too_many_lines)]
fn collect_sched_helper<'a, T: Iterator<Item = &'a SchedStmt>>(
    ctx: &Context,
    env: &mut DTypeEnv,
    stmts: T,
    mut num_stmts: usize,
    mutables: &mut HashMap<String, Info>,
) -> Result<Vec<String>, LocalError> {
    for stmt in stmts {
        num_stmts -= 1;
        match stmt {
            SchedStmt::Decl {
                lhs: dest,
                expr: None,
                info,
                ..
            } => collect_null_decl(env, dest, *info)?,
            SchedStmt::Decl {
                lhs: dest,
                expr:
                    Some(SchedExpr::Binop {
                        info,
                        op: Binop::Dot,
                        lhs,
                        rhs,
                    }),
                ..
            } => collect_dot(env, dest, lhs, rhs, *info)?,
            SchedStmt::Decl {
                lhs: dest,
                expr: Some(SchedExpr::Binop { info, op, lhs, rhs }),
                ..
            } => collect_bop(dest, *op, lhs, rhs, env, *info)?,
            SchedStmt::Decl {
                lhs: dest,
                expr: Some(SchedExpr::Uop { info, op, expr }),
                ..
            } => collect_assign_uop(dest, *op, expr, env, *info, mutables)?,
            SchedStmt::Decl {
                lhs: dest,
                expr: Some(SchedExpr::Term(SchedTerm::Lit { lit, .. })),
                info,
                ..
            } => collect_assign_lit(dest, lit, env, *info)?,
            SchedStmt::Decl {
                lhs: dest,
                expr: Some(SchedExpr::Term(SchedTerm::Var { name, .. })),
                info,
                ..
            } => collect_assign_var(dest, name, env, *info)?,
            SchedStmt::Decl {
                lhs: dest,
                expr: Some(SchedExpr::Term(SchedTerm::Call(_, call_info))),
                info,
                ..
            } => {
                collect_assign_call(ctx, env, dest, call_info, *info)?;
            }
            SchedStmt::Assign {
                lhs: SchedExpr::Term(SchedTerm::Var { name: dest, .. }),
                rhs: SchedExpr::Term(SchedTerm::Var { name, .. }),
                info,
                lhs_is_ref,
                ..
            } => {
                mutables.insert(dest.clone(), *info);
                if *lhs_is_ref {
                    let x = env.env.get_type(name).unwrap();
                    env.add_constraint(dest, DTypeConstraint::Ref(x), *info)?;
                } else {
                    env.add_var_equiv(dest, name, *info)?;
                }
            }
            SchedStmt::Block(_, b) => {
                assert_eq!(num_stmts, 0);
                return collect_sched_helper(ctx, env, b.iter(), b.len(), mutables);
            }
            SchedStmt::Seq {
                info, dests, block, ..
            } => collect_seq(ctx, env, dests, block, *info, mutables)?,
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                info,
                ..
            } => collect_if(ctx, env, guard, true_block, false_block, *info, mutables)?,
            // TODO: implement timeline operations
            SchedStmt::Decl {
                lhs,
                expr: Some(SchedExpr::Term(SchedTerm::TimelineOperation { op, arg, info, .. })),
                ..
            } => {
                collect_timeline_op(env, *op, lhs, arg, *info)?;
            }
            SchedStmt::Decl {
                lhs,
                expr: Some(SchedExpr::Term(SchedTerm::EncodeBegin { info, defs, .. })),
                ..
            } => {
                collect_begin_encode(env, lhs, defs, *info)?;
            }
            SchedStmt::InEdgeAnnotation { .. }
            | SchedStmt::OutEdgeAnnotation { .. }
            | SchedStmt::Hole(_) => (),
            SchedStmt::Call(info, call_info) => {
                collect_assign_call(ctx, env, &[], call_info, *info)?;
            }
            SchedStmt::Return(_, e) => {
                assert_eq!(num_stmts, 0);
                match e {
                    SchedExpr::Term(SchedTerm::Var { name, .. }) => {
                        return Ok(vec![name.clone()]);
                    }
                    SchedExpr::Term(SchedTerm::Lit {
                        lit: SchedLiteral::Tuple(rets),
                        ..
                    }) => {
                        let mut ret_names = Vec::new();
                        for ret in rets {
                            let ret_name = enum_cast!(
                                SchedTerm::Var { name, .. },
                                name,
                                enum_cast!(SchedExpr::Term, ret)
                            );
                            ret_names.push(ret_name.clone());
                        }
                        return Ok(ret_names);
                    }
                    _ => unreachable!(),
                }
            }
            SchedStmt::Encode {
                stmt, cmd, info, ..
            } => collect_encode(ctx, env, stmt, *cmd, *info)?,
            _ => unreachable!("{:#?}", stmt),
        }
    }
    Ok(vec![])
}

fn collect_encode(
    _ctx: &Context,
    env: &mut DTypeEnv,
    stmt: &EncodedStmt,
    cmd: EncodedCommand,
    info: Info,
) -> Result<(), LocalError> {
    match cmd {
        EncodedCommand::Copy => {
            let src = enum_cast!(
                SchedTerm::Var { name, .. },
                name,
                enum_cast!(SchedExpr::Term, &stmt.rhs)
            );
            let dest = stmt.lhs[0].0.clone();
            env.add_var_equiv(src, &dest, info)?;
            env.add_usage(&dest, WGPUFlags::CopyDst);
            Ok(())
        }
        EncodedCommand::Invoke => {
            // TODO: typing encode-do (should GPU args be refs or values?)
            // if let SchedTerm::Call(info, call) = enum_cast!(SchedExpr::Term, &stmt.rhs) {
            //     collect_assign_call(
            //         ctx,
            //         env,
            //         &stmt
            //             .lhs
            //             .iter()
            //             .map(|(x, _)| (x.clone(), None))
            //             .collect::<Vec<_>>(),
            //         call,
            //         *info,
            //     )
            // } else {
            //     unreachable!();
            // }
            env.add_usage(&stmt.lhs[0].0, WGPUFlags::MapRead);
            Ok(())
        }
    }
}

/// Collects constraints for a schedule. Requires that the input constraints
/// have already been added to the environment.
/// # Arguments
/// * `ctx` - The context of the schedule.
/// * `env` - The environment to add constraints to.
/// * `stmts` - The statements to collect constraints from.
/// * `fn_out` - The return types of the function.
/// * `sig_outs` - The return types of the function signature.
/// * `info` - The info of the schedule.
/// * `fn_name` - The name of the function.
/// # Returns
/// A mapping of mutable names to their info.
#[allow(clippy::too_many_arguments)]
pub fn collect_schedule(
    ctx: &Context,
    env: &mut DTypeEnv,
    stmts: &[SchedStmt],
    fn_out: &[FullType],
    fn_in: &[(String, Option<FullType>)],
    info: Info,
) -> Result<HashMap<String, Info>, LocalError> {
    let mut mutables = HashMap::new();
    let rets = collect_sched_helper(ctx, env, stmts.iter(), stmts.len(), &mut mutables)?;
    if rets.len() != fn_out.len() {
        return Err(type_error(
            info,
            &format!(
                "{info}: Expected {} return values, found {}",
                fn_out.len(),
                rets.len()
            ),
        ));
    }
    for (ret_name, fn_t) in rets.iter().zip(fn_out.iter()) {
        if let FullType {
            base: Some(FlaggedType { base, flags, .. }),
            ..
        } = fn_t
        {
            env.add_dtype_constraint(ret_name, base.clone(), info)?;
            for flag in flags {
                env.add_usage(ret_name, *flag);
            }
        } else {
            panic!("Function return type has no base type");
        }
    }
    for (var, typ) in fn_in {
        if let Some(FullType {
            base: Some(FlaggedType { flags, .. }),
            ..
        }) = &typ
        {
            for flag in flags {
                env.add_usage(var, *flag);
            }
        }
    }
    Ok(mutables)
}
