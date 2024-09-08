use std::collections::{HashMap, HashSet};

use crate::{
    enum_cast,
    error::{Info, LocalError},
    lower::tuple_id,
    parse::ast::{
        Binop, DataType, FlaggedType, IntSize, SpecExpr, SpecLiteral, SpecStmt, SpecTerm,
        TemplateArgs,
    },
    type_error,
};

use super::{
    binop_to_contraints,
    types::{DTypeConstraint, MetaVar, ValQuot},
    DTypeEnv, NodeEnv, Signature, SpecInfo, TypedBinop, UnresolvedTypedBinop,
};

/// Collects all names defined in a given spec, including inputs and outputs
fn collect_spec_names(
    stmts: &Vec<SpecStmt>,
    ctx: &SpecInfo,
) -> Result<HashSet<String>, LocalError> {
    let mut res = HashSet::new();
    for (name, _) in &ctx.sig.input {
        if res.contains(name) {
            return Err(type_error!(ctx.info, "Duplicate node: {name}"));
        }
        res.insert(name.clone());
    }
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, info, .. } => {
                for (name, _) in lhs {
                    if res.contains(name) {
                        return Err(type_error!(*info, "Duplicate node: {name}"));
                    }
                    res.insert(name.clone());
                }
            }
            SpecStmt::Returns(..) => (),
        }
    }
    for i in 0..ctx.sig.num_dims {
        res.insert(format!("_dim{i}"));
    }
    Ok(res)
}

/// Gets the input and output types of a given function class.
/// # Arguments
/// * `callee` - the name of the function class
/// * `signatures` - a map from function names to their signatures
/// * `args` - the arguments to the function
/// * `num_dests` - the number of return values from the function
/// * `info` - the location of the function call in the source code
/// # Returns
/// A tuple containing the input and output types of the function class.
/// # Errors
/// Returns an error if the function class is not found in `signatures`, if the
/// number of arguments to the function class does not match the number of arguments
fn get_target_signature(
    callee: &str,
    signatures: &HashMap<String, Signature>,
    args: &[SpecExpr],
    num_dests: usize,
    info: Info,
) -> Result<(Vec<FlaggedType>, Vec<FlaggedType>), LocalError> {
    let (input_types, output_types) = match callee {
        // (event, fence ...) -> (event, encoder)
        "encode_event" => (
            std::iter::once(DataType::Event)
                .chain(std::iter::repeat(DataType::Fence(None)).take(args.len().max(1) - 1))
                .map(FlaggedType::from)
                .collect(),
            vec![
                FlaggedType::from(DataType::Event),
                FlaggedType::from(DataType::Encoder(None)),
            ],
        ),
        // encoder -> fence
        "submit_event" => (
            vec![FlaggedType::from(DataType::Encoder(None))],
            vec![FlaggedType::from(DataType::Fence(None))],
        ),
        // (event, fence) -> event
        "sync_event" => (
            vec![
                FlaggedType::from(DataType::Event),
                FlaggedType::from(DataType::Fence(None)),
            ],
            vec![FlaggedType::from(DataType::Event)],
        ),
        _ => (
            signatures
                .get(callee)
                .ok_or_else(|| type_error!(info, "Unknown spec '{callee}' invoked"))?
                .input
                .clone(),
            signatures.get(callee).unwrap().output.clone(),
        ),
    };
    if args.len() != input_types.len() {
        return Err(type_error!(
            info,
            "Wrong number of arguments to function '{callee}': expected {}, got {}",
            input_types.len(),
            args.len()
        ));
    }
    if num_dests != output_types.len() {
        return Err(type_error!(
            info,
            "Wrong number of return values from function '{callee}': expected {}, got {}",
            output_types.len(),
            num_dests
        ));
    }
    Ok((input_types, output_types))
}

/// Returns true if `fn_name` is the name of a builtin function which returns
/// a single argument. If this function returns true, then `fn_name` doesn't
/// need extract nodes
fn is_single_return_builtin(fn_name: &str) -> bool {
    fn_name == "sync_event" || fn_name == "submit_event"
}

/// Gets a list of arguments (regular arguments + non-type template args)
/// to a function call.
/// # Panics
/// Panics if the arguments are not lowered to variables
fn get_call_arguments(args: &[SpecExpr], templates: &Option<TemplateArgs>) -> Vec<String> {
    let mut arg_nodes: Vec<_> = args
        .iter()
        .map(|arg| {
            let t = enum_cast!(SpecExpr::Term, arg);
            let name = enum_cast!(SpecTerm::Var { name, .. }, name, t);
            name.to_string()
        })
        .collect();
    if let Some(TemplateArgs::Vals(vs)) = templates {
        let mut res: Vec<_> = vs
            .iter()
            .map(|arg| {
                let t = enum_cast!(SpecExpr::Term, arg);
                let name = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                name.to_string()
            })
            .collect();
        res.extend(arg_nodes);
        arg_nodes = res;
    }
    arg_nodes
}

/// Collects types and nodes for a given assignment `lhs :- function(args)`.
#[allow(clippy::too_many_arguments)]
fn collect_spec_assign_call(
    lhs: &[(String, Option<DataType>)],
    function: &SpecExpr,
    args: &[SpecExpr],
    templates: &Option<TemplateArgs>,
    ctx: &mut SpecEnvs,
    signatures: &HashMap<String, Signature>,
    dimensions: &HashMap<String, usize>,
    info: Info,
) -> Result<String, LocalError> {
    if let SpecExpr::Term(SpecTerm::Var {
        name: func_name, ..
    }) = function
    {
        let (input_types, output_types) =
            get_target_signature(func_name, signatures, args, lhs.len(), info)?;
        let arg_nodes = get_call_arguments(args, templates);
        let single_ret_builtin = is_single_return_builtin(func_name);
        assert!(!single_ret_builtin || lhs.len() == 1);
        let tuple_name = if single_ret_builtin {
            lhs[0].0.clone()
        } else {
            tuple_id(&lhs.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>())
        };
        if single_ret_builtin {
            ctx.nodes.add_quotient(
                &tuple_name,
                ValQuot::CallOne(
                    func_name.clone(),
                    arg_nodes
                        .iter()
                        .map(|x| MetaVar::new_class_name(x))
                        .collect(),
                ),
            );
        } else {
            ctx.nodes.add_quotient(
                &tuple_name,
                ValQuot::Call(
                    func_name.clone(),
                    arg_nodes
                        .iter()
                        .map(|x| MetaVar::new_class_name(x))
                        .collect(),
                ),
            );
        }
        for (idx, ((name, annot), typ)) in lhs.iter().zip(output_types.iter()).enumerate() {
            if let Some(a) = annot {
                if a != &typ.base {
                    return Err(type_error!(
                        info,
                        "Annotation of '{name}' conflicts with return type of '{func_name}'"
                    ));
                }
            }
            ctx.types
                .add_dtype_constraint(name, typ.base.clone(), info)?;
            if !single_ret_builtin {
                ctx.nodes.add_quotient(
                    name,
                    ValQuot::Extract(MetaVar::new_class_name(&tuple_name), idx),
                );
            }
        }
        let num_dims = dimensions.get(func_name).copied().unwrap_or(0);
        for arg_name in arg_nodes.iter().take(num_dims) {
            ctx.types
                .add_dtype_constraint(arg_name, DataType::Int(IntSize::I32), info)?;
        }
        for (arg_name, arg_type) in arg_nodes.iter().skip(num_dims).zip(input_types.iter()) {
            ctx.types
                .add_dtype_constraint(arg_name, arg_type.base.clone(), info)?;
        }
        Ok(func_name.clone())
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
    ctx: &mut SpecEnvs,
    signatures: &HashMap<String, Signature>,
    dimensions: &HashMap<String, usize>,
    called_specs: &mut HashSet<String>,
) -> Result<(), LocalError> {
    match t {
        SpecTerm::Lit { lit, info } => {
            ctx.nodes.add_quotient(
                &lhs[0].0,
                match lit {
                    SpecLiteral::Int(i) => ValQuot::Int(i.clone()),
                    SpecLiteral::Bool(b) => ValQuot::Bool(*b),
                    SpecLiteral::Float(f) => ValQuot::Float(f.clone()),
                    _ => todo!("Unimplemented literal type in spec"),
                },
            );
            if let Some(annot) = lhs[0].1.as_ref() {
                ctx.types
                    .add_dtype_constraint(&lhs[0].0, annot.clone(), *info)?;
            }
            ctx.types.add_constraint(
                &lhs[0].0,
                match lit {
                    SpecLiteral::Int(_) => DTypeConstraint::Int(None),
                    SpecLiteral::Bool(_) => DTypeConstraint::Bool,
                    SpecLiteral::Float(_) => DTypeConstraint::Float(None),
                    _ => todo!("Unimplemented literal type in spec"),
                },
                *info,
            )
        }
        SpecTerm::Var { .. } => unimplemented!("Variable assignment in spec"),
        SpecTerm::Call {
            function,
            args,
            templates,
            info,
            ..
        } => {
            let r = collect_spec_assign_call(
                lhs, function, args, templates, ctx, signatures, dimensions, *info,
            )?;
            called_specs.insert(r);
            Ok(())
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
    ctx: &mut SpecEnvs,
    info: Info,
) -> Result<(), LocalError> {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
        SpecExpr::Term(SpecTerm::Var {
            name: guard,
            info: g_info,
        }),
    ) = (if_true, if_false, guard)
    {
        ctx.types
            .add_dtype_constraint(guard, DataType::Bool, *g_info)?;
        ctx.types.add_var_equiv(name1, name2, info)?;
        ctx.types.add_var_equiv(&lhs[0].0, name1, info)?;
        if let Some(t) = lhs[0].1.as_ref() {
            ctx.types.add_dtype_constraint(&lhs[0].0, t.clone(), info)?;
        }
        ctx.nodes.add_quotient(
            &lhs[0].0,
            ValQuot::Select {
                guard: MetaVar::new_class_name(guard),
                true_id: MetaVar::new_class_name(name1),
                false_id: MetaVar::new_class_name(name2),
            },
        );
    } else {
        panic!("Not lowered")
    }
    Ok(())
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
    ctx: &mut SpecEnvs,
    info: Info,
) -> Result<(), LocalError> {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
    ) = (op_l, op_r)
    {
        let (left_constraint, right_constraint, ret_constraint) =
            binop_to_contraints(op, &mut ctx.types.env);
        ctx.types
            .add_raw_constraint(&lhs[0].0, &ret_constraint, info)?;
        if let Some(annot) = &lhs[0].1 {
            ctx.types
                .add_dtype_constraint(&lhs[0].0, annot.clone(), info)?;
        }
        ctx.types
            .add_raw_constraint(name1, &left_constraint, info)?;
        ctx.types
            .add_raw_constraint(name2, &right_constraint, info)?;
        externs.insert(UnresolvedTypedBinop {
            op,
            op_l: name1.clone(),
            op_r: name2.clone(),
            ret: lhs[0].0.clone(),
        });
        ctx.nodes.add_quotient(
            &lhs[0].0,
            ValQuot::Bop(
                op,
                MetaVar::new_class_name(name1),
                MetaVar::new_class_name(name2),
            ),
        );
    } else {
        panic!("Not lowered")
    }
    Ok(())
}

/// Resolves all types for defined variables in a given spec.
fn resolve_types(
    env: &DTypeEnv,
    names: &HashSet<String>,
    ctx: &mut SpecInfo,
) -> Result<(), LocalError> {
    for name in names {
        match env.env.get_type(name) {
            Some(c) => {
                let dt = DTypeConstraint::try_from(c.clone()).map_err(|e| {
                    type_error!(ctx.info, "Failed to resolve type of variable '{name}': {e}")
                })?;
                ctx.types.insert(
                    name.clone(),
                    dt.try_into().map_err(|()| {
                        type_error!(
                            ctx.info,
                            "Failed to resolve type of variable '{name}'. Not enough constraints."
                        )
                    })?,
                );
            }
            None => return Err(type_error!(ctx.info, "Undefined variable '{name}' in spec")),
        }
    }
    Ok(())
}

fn collect_spec_sig(env: &mut SpecEnvs, ctx: &SpecInfo) -> Result<(), LocalError> {
    let info = ctx.info;
    for (arg, typ) in ctx.sig.input.clone() {
        env.types.add_dtype_constraint(&arg, typ.base, info)?;
        env.nodes.add_quotient(&arg, ValQuot::Input(arg.clone()));
    }
    for i in 0..ctx.sig.num_dims {
        let name = format!("_dim{i}");
        env.types
            .add_dtype_constraint(&name, DataType::Int(IntSize::I32), info)?;
        env.nodes.add_quotient(&name, ValQuot::Input(name.clone()));
    }
    Ok(())
}

fn collect_spec_returns(
    env: &mut SpecEnvs,
    ctx: &SpecInfo,
    e: &SpecExpr,
    info: Info,
) -> Result<(), LocalError> {
    env.nodes.set_output_classes(&ctx.sig);
    match e {
        SpecExpr::Term(SpecTerm::Var { name, .. }) => {
            if ctx.sig.output.len() != 1 {
                return Err(type_error!(
                    info,
                    "Wrong number of return values: expected {}, got {}",
                    ctx.sig.output.len(),
                    1
                ));
            }
            env.types
                .add_dtype_constraint(name, ctx.sig.output[0].1.base.clone(), info)?;
            env.nodes.add_quotient(
                &ctx.sig.output[0].0,
                ValQuot::Output(MetaVar::new_class_name(name)),
            );
            Ok(())
        }
        SpecExpr::Term(
            SpecTerm::Lit {
                lit: SpecLiteral::Tuple(rets),
                ..
            },
            ..,
        ) => {
            if rets.len() != ctx.sig.output.len() {
                return Err(type_error!(
                    info,
                    "Wrong number of return values: expected {}, got {}",
                    ctx.sig.output.len(),
                    rets.len()
                ));
            }
            let mut constraints = vec![];
            for (r, out) in rets.iter().zip(ctx.sig.output.iter()) {
                if let SpecExpr::Term(SpecTerm::Var { name, .. }) = r {
                    constraints.push((name, out.clone()));
                } else {
                    panic!("Not lowered")
                }
            }
            for (name, (class, typ)) in constraints {
                env.types.add_dtype_constraint(name, typ.base, info)?;
                env.nodes
                    .add_quotient(&class, ValQuot::Output(MetaVar::new_class_name(name)));
            }
            Ok(())
        }
        _ => panic!("Not lowered"),
    }
}

struct SpecEnvs {
    pub types: DTypeEnv,
    pub nodes: NodeEnv,
}

impl SpecEnvs {
    fn new() -> Self {
        Self {
            types: DTypeEnv::new(),
            nodes: NodeEnv::new(),
        }
    }
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
    dimensions: &HashMap<String, usize>,
) -> Result<(HashSet<TypedBinop>, HashSet<String>), LocalError> {
    let mut unresolved_externs = HashSet::new();
    let names = collect_spec_names(stmts, ctx)?;
    let mut env = SpecEnvs::new();
    let mut called_specs = HashSet::new();
    collect_spec_sig(&mut env, ctx)?;
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, rhs, .. } => match rhs {
                SpecExpr::Term(t) => {
                    collect_spec_assign_term(
                        t,
                        lhs,
                        &mut env,
                        signatures,
                        dimensions,
                        &mut called_specs,
                    )?;
                }
                SpecExpr::Conditional {
                    if_true,
                    guard,
                    if_false,
                    info,
                } => collect_spec_assign_if(lhs, if_true, if_false, guard, &mut env, *info)?,
                SpecExpr::Binop {
                    op,
                    lhs: op_l,
                    rhs: op_r,
                    info,
                } => collect_spec_assign_bop(
                    op_l,
                    op_r,
                    *op,
                    &mut unresolved_externs,
                    lhs,
                    &mut env,
                    *info,
                )?,

                SpecExpr::Uop { .. } => todo!(),
            },
            SpecStmt::Returns(info, e) => collect_spec_returns(&mut env, ctx, e, *info)?,
        }
    }
    resolve_types(&env.types, &names, ctx)?;
    ctx.nodes = env.nodes;
    Ok((
        unresolved_externs
            .into_iter()
            .map(|u| TypedBinop {
                op: u.op,
                op_l: ctx.types[&u.op_l].clone(),
                op_r: ctx.types[&u.op_r].clone(),
                ret: ctx.types[&u.ret].clone(),
            })
            .collect::<HashSet<_>>(),
        called_specs,
    ))
}
