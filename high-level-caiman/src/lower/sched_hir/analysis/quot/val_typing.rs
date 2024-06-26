//! This module contains functions for deducing the value quotients of variables.
//! The general idea is that each value in the value spec can be seen as a tree
//! of operations that have been computed to result in that value. For example,
//!
//! ```text
//! a :- 1
//! b :- 2
//! c :- a + b
//! ```
//!
//! can be thought of as the tree:
//!
//! ```text
//! 1   2
//! |   |
//! a   b
//!  \ /
//!   + = c
//! ```
//!
//! A similar computation in the schedule can be mapped to an isomorphic tree.
//!
//! So, in the spec we compute and return some results whose computation tree
//! we can build from the operations that yielded those results in the spec. In
//! the schedule, we also have some results whose computation tree we can build
//! from the various operations that yielded those results.
//!
//! The idea is that we can unify the spec forest with the schedule forest,
//! matching the nodes in the schedule to the nodes in the spec.
//!
//! Right now, this approach works for pretty much everything that exists in the
//! language right now. I suspect that down the line, there might be some
//! black-box operation in the schedule that disconnects a schedule tree.
//! In the previous example, this would like not being able to infer that `c` is
//! the result of `a + b`.
//!
//! I'm not sure if any kind of black-box operation would ever come up, but to
//! hedge my bets, the algorithm will also immediately unify any subtrees in the
//! schedule that unambiguously matches a subtree in the spec. For example, if
//! there is only one `1` in the spec, and only one `1` in the schedule, then
//! the algorithm will immediately unify those two nodes.
//!
//! Unification is technically Big-O exponential, but in practice with path
//! compression, it's linear. Actually, we don't have generics right now,
//! so I think it's polynomial. (The added unification to convergence makes
//! this non-linear)
//!
//! The way we do unification is to add *class names* to the equivalence classes
//! in the union-find data structure. (Ie. construct an e-graph).
//! The class names are the names of the
//! value spec nodes that the schedule nodes are equivalent to. This allows us
//! to get the canonical representative (spec node id) of any equivalence class.
//!
//! Class names are prefixed with a `$` to distinguish them from regular variable.
//! In this implementation, all spec nodes would be class names. As such, two
//! "equivalent" things that are specified independently in the spec would not be
//! part of the same equivalence class. For example:
//!
//! ```text
//! a :- 1
//! b :- 1
//! returns a + b
//! ```
//!
//! CANNOT be implemented with the following schedule:
//!
//! ```text
//! let c = 1;
//! c + c
//! ```
//!
//! This is because two different `1`s are used in the schedule, and the algorithm
//! treats them as different equivalence classes. This particular example could
//! work with a small tweak to the algorithm, but we decided it's doesn't really
//! preserve the meaning of the spec.
//!

use std::collections::HashMap;

use crate::{
    error::{type_error, Info, LocalError},
    lower::{
        sched_hir::{
            cfg::{BasicBlock, Cfg, Edge, START_BLOCK_ID},
            HirBody, HirFuncCall, HirOp, OpType, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{
        Binop, DataType, Quotient, QuotientReference, SchedLiteral, SchedTerm, SpecType, Tag,
    },
    typing::{is_value_dtype, Context, MetaVar, NodeEnv, SchedOrExtern, SpecInfo, ValQuot},
};

use super::{
    super::{continuations::compute_pretinuations, ssa},
    add_constraint, add_node_eq, add_var_constraint, fill_quotient,
};

/// Deduces the quotients for the value specification. Returns an error
/// if unification fails, otherwise, writes the deduced quotients to the tags
/// of the instructions in the cfg.
#[allow(clippy::too_many_arguments)]
pub fn deduce_val_quots(
    inputs: &mut [(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    cfg: &mut Cfg,
    spec_info: &SpecInfo,
    ctx: &Context,
    dtypes: &HashMap<String, DataType>,
    info: Info,
) -> Result<(), LocalError> {
    let env = spec_info.nodes.clone();
    let mut overrides = Vec::new();
    for i in &cfg.blocks[&START_BLOCK_ID].stmts {
        if let HirBody::InAnnotation(_, tags) = i {
            overrides.extend(tags.iter().cloned());
        }
    }
    let env = add_io_constraints(
        env,
        inputs,
        &overrides,
        outputs,
        output_dtypes,
        dtypes,
        info,
        spec_info.sig.num_dims,
    )?;
    let (mut env, selects) = unify_nodes(cfg, ctx, info, dtypes, env)?;
    env.converge_types()
        .map_err(|e| type_error(Info::default(), &format!("Convergence failure: {e}")))?;
    fill_type_info(&env, cfg, &selects);
    fill_io_type_info(inputs, outputs, output_dtypes, &env);
    Ok(())
}

/// Adds constraints to the environment based on input and output annotations.
/// Any unspecified annotations are going to be assumed to match up with the
/// spec. Requires that the input and output variables of a given dimension
/// (timeline, value, etc.) are kept in the same relative order as the spec.
#[allow(clippy::too_many_arguments)]
fn add_io_constraints(
    mut env: NodeEnv,
    inputs: &mut [(String, TripleTag)],
    input_overrides: &[(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    dtypes: &HashMap<String, DataType>,
    info: Info,
    num_dims: usize,
) -> Result<NodeEnv, LocalError> {
    env.override_output_classes(
        output_dtypes.iter().zip(outputs.iter().map(|t| &t.value)),
        &is_value_dtype,
        0,
    );
    for (name, tag) in input_overrides {
        for (n2, t2) in inputs.iter_mut() {
            if n2 == name {
                t2.set_specified_info(tag.clone());
            }
        }
    }
    for i in 0..num_dims {
        env = super::add_node_eq(&format!("_dim{i}"), &format!("_dim{i}"), info, env)?;
    }
    for (idx, (arg_name, fn_in_tag)) in inputs
        .iter()
        .filter(|(arg, _)| is_value_dtype(&dtypes[&ssa::original_name(arg)]))
        .enumerate()
    {
        if fn_in_tag.value.quot == Some(Quotient::None) {
            continue;
        }
        let class_name = if let Some(annoted_quot) = &fn_in_tag.value.quot_var.spec_var {
            annoted_quot.clone()
        } else {
            let spec_classes = env.get_input_classes();
            if idx < spec_classes.len() {
                spec_classes[idx].clone()
            } else {
                continue;
            }
        };
        env = super::add_node_eq(arg_name, &class_name, info, env)?;
    }
    Ok(env)
}

/// Adds a type constraint to the environment, allowing value
/// information from `TripleTag` to override the constraint.
/// # Arguments
/// * `lhs` - The name of the variable to constrain
/// * `rhs` - The constraint to apply to the type variable
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment
#[allow(clippy::unnecessary_wraps)]
fn add_overrideable_constraint(
    lhs: &str,
    lhs_tag: &TripleTag,
    rhs: &ValQuot,
    info: Info,
    env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    super::add_overrideable_constraint(lhs, lhs_tag, rhs, info, env, &|dt| &dt.value)
}

/// Adds a type annotation for `name` to the environement if the given annotation
/// provides a value node matching.
/// # Arguments
/// * `name` - The name of the variable to annotate
/// * `annot` - The annotation to add
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn add_type_annot(
    name: &str,
    annot: &TripleTag,
    info: Info,
    env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    super::add_type_annot(name, annot, info, env, &|dt| &dt.value)
}

/// Unifies an assignment to the variable `lhs` with the given rhs.
/// The lhs name will become a type variable and the rhs will be a constraint on it.
/// # Arguments
/// * `lhs` - The name of the variable being assigned to
/// * `lhs_tag` - The type annotation for the variable being assigned to
/// * `rhs` - The value being assigned to the variable
/// * `specs` - The specs
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn unify_decl(
    lhs: &str,
    lhs_tag: &TripleTag,
    rhs: &SchedTerm,
    decl_info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    match rhs {
        SchedTerm::Lit {
            lit: SchedLiteral::Int(i),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), *info, env)?;
            env = add_constraint(lhs, &ValQuot::Int(i.clone()), *info, env)?;
        }
        SchedTerm::Lit {
            lit: SchedLiteral::Bool(b),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), *info, env)?;
            env = add_constraint(lhs, &ValQuot::Bool(*b), *info, env)?;
        }
        SchedTerm::Lit {
            lit: SchedLiteral::Float(f),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), *info, env)?;
            env = add_constraint(lhs, &ValQuot::Float(f.clone()), *info, env)?;
        }
        SchedTerm::Var { name, info, tag } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), *info, env)?;
            env = add_var_constraint(lhs, name, *info, env)?;
        }
        x => todo!("{x:#?}"),
    }
    add_type_annot(lhs, lhs_tag, decl_info, env)
}

/// Converts an `HirOp` to a `Binop`
/// # Panics
/// If the `HirOp` is not a binary operator
fn hir_op_to_binop(op: &HirOp) -> Binop {
    match op {
        HirOp::Binary(binop) => *binop,
        HirOp::FFI(name, OpType::Binary) => {
            let mut parts: Vec<_> = name.split('_').collect();
            match parts.swap_remove(1) {
                "add" => Binop::Add,
                "sub" => Binop::Sub,
                "mul" => Binop::Mul,
                "div" => Binop::Div,
                "mod" => Binop::Mod,
                "and" => Binop::And,
                "or" => Binop::Or,
                "xor" => Binop::Xor,
                "shl" => Binop::Shl,
                "shr" => Binop::Shr,
                "eq" => Binop::Eq,
                "neq" => Binop::Neq,
                "lt" => Binop::Lt,
                "leq" => Binop::Leq,
                "gt" => Binop::Gt,
                "geq" => Binop::Geq,
                "ashr" => Binop::AShr,
                "land" => Binop::Land,
                "lor" => Binop::Lor,
                x => panic!("Unrecognized FFI binop: {x}"),
            }
        }
        HirOp::Unary(_) => panic!("Not a binary operator"),
        HirOp::FFI(_, b) => panic!("Unexpected op type: {b:?}"),
    }
}

/// Unifies a built-in operation with the given name and arguments
/// # Arguments
/// * `dest` - The name of the variable to assign the result to
/// * `dest_tag` - The type annotation for the variable being assigned to
/// * `op` - The operation to perform
/// * `args` - The arguments to the operation
/// * `info` - The source info for the operation
/// * `specs` - The specs
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn unify_op(
    dests: &[(String, TripleTag)],
    op: &HirOp,
    args: &[SchedTerm],
    info: Info,
    ctx: &Context,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    let mut arg_names = vec![];
    for arg in args {
        match arg {
            SchedTerm::Var { name, tag, info } => {
                env = add_type_annot(name, &TripleTag::from_opt(tag), *info, env)?;
                arg_names.push(name.clone());
            }
            _ => unreachable!(),
        }
    }
    match op {
        HirOp::FFI(_, OpType::Binary) => {
            assert_eq!(dests.len(), 1);
            env = add_constraint(
                &dests[0].0,
                &ValQuot::Bop(
                    hir_op_to_binop(op),
                    MetaVar::new_var_name(&arg_names[0]),
                    MetaVar::new_var_name(&arg_names[1]),
                ),
                info,
                env,
            )?;
        }
        HirOp::FFI(target, OpType::External) => {
            // The name of an external function is the name of its value spec
            let f_class = ctx.specs[target].feq.clone();
            let dest_tuple = tuple_id(
                &dests
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>(),
            );
            env = add_constraint(
                &format!("!{dest_tuple}"),
                &ValQuot::Call(
                    f_class,
                    arg_names.iter().map(|x| MetaVar::new_var_name(x)).collect(),
                ),
                info,
                env,
            )?;
            for (id, (dest, _)) in dests.iter().enumerate() {
                env = add_constraint(
                    dest,
                    &ValQuot::Extract(MetaVar::new_var_name(&format!("!{dest_tuple}")), id),
                    info,
                    env,
                )?;
            }
        }
        HirOp::FFI(..) => todo!("Unimplemented operator"),
        _ => unreachable!(),
    }
    for (dest, dest_tag) in dests {
        env = add_type_annot(dest, dest_tag, info, env)?;
    }
    Ok(env)
}

/// Unifies a phi node with the given name and inputs
/// # Arguments
/// * `dest` - The name of the phi node
/// * `inputs` - The inputs to the phi node
/// * `pretinuations` - A map from each block to the block that contains the split point
/// for that block
/// * `cfg` - The cfg
/// * `block` - The block that contains the phi node
/// * `selects` - A map from each block with a select node to the name of the variable
/// which maps to the select node. Updated by this function.
/// * `env` - The current environment
fn unify_phi(
    dest: &str,
    incoming_edges: &HashMap<usize, String>,
    pretinuations: &HashMap<usize, usize>,
    cfg: &Cfg,
    block: usize,
    selects: &mut HashMap<usize, String>,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    let split_point = pretinuations[&block];
    let split_block = &cfg.blocks[&split_point];
    if let Terminator::Select { guard, info, .. } = &split_block.terminator {
        if let Edge::Select {
            true_branch,
            false_branch,
        } = cfg.graph[&split_point]
        {
            let incoming_edges: Vec<_> = incoming_edges.iter().collect();
            assert_eq!(incoming_edges.len(), 2);
            if cfg.succs.succs[&true_branch].contains(incoming_edges[0].0) {
                assert!(cfg.succs.succs[&false_branch].contains(incoming_edges[1].0));
                env = add_constraint(
                    dest,
                    &ValQuot::Select {
                        guard: MetaVar::new_var_name(guard),
                        true_id: MetaVar::new_var_name(incoming_edges[0].1),
                        false_id: MetaVar::new_var_name(incoming_edges[1].1),
                    },
                    *info,
                    env,
                )?;
            } else {
                assert!(cfg.succs.succs[&false_branch].contains(incoming_edges[0].0));
                assert!(cfg.succs.succs[&true_branch].contains(incoming_edges[1].0));
                env = add_constraint(
                    dest,
                    &ValQuot::Select {
                        guard: MetaVar::new_var_name(guard),
                        true_id: MetaVar::new_var_name(incoming_edges[1].1),
                        false_id: MetaVar::new_var_name(incoming_edges[0].1),
                    },
                    *info,
                    env,
                )?;
            }
            selects.insert(split_point, dest.to_string());
            Ok(env)
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

/// The name of the value specification that `sched_name` implements.
fn value_name(sched_name: &str, ctx: &Context) -> String {
    match ctx.scheds.get(sched_name).unwrap() {
        SchedOrExtern::Sched(sched) => sched.value.clone(),
        SchedOrExtern::Extern(_) => sched_name.to_string(),
    }
}

/// Unifies a call to a HIR function.
/// # Arguments
/// * `dests` - The names of the variables to assign the results to
/// * `call` - The call to the HIR function
/// * `ctx` - The context
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn unify_call(
    dests: &[(String, TripleTag)],
    call: &HirFuncCall,
    ctx: &Context,
    dtypes: &HashMap<String, DataType>,
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    let val_spec = value_name(&call.target, ctx);
    let f_class = ctx.specs[&val_spec].feq.clone();
    let tuple_name = tuple_id(
        &dests
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>(),
    );
    env = add_type_annot(&tuple_name, &call.tag, info, env)?;
    env = add_overrideable_constraint(
        &tuple_name,
        &call.tag,
        &ValQuot::SchedCall(
            f_class,
            call.args
                .iter()
                .filter_map(|arg| {
                    let t = dtypes.get(&ssa::original_name(arg)).unwrap_or_else(|| {
                        panic!("Missing type info for {}", ssa::original_name(arg))
                    });
                    if is_value_dtype(t) {
                        Some(MetaVar::new_var_name(arg))
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        info,
        env,
    )?;
    for (idx, (dest, tag)) in dests
        .iter()
        .filter(|(x, _)| {
            is_value_dtype(
                dtypes
                    .get(&ssa::original_name(x))
                    .unwrap_or_else(|| panic!("Missing type info for {}", ssa::original_name(x))),
            )
        })
        .enumerate()
    {
        env = add_type_annot(dest, tag, info, env)?;
        env = add_overrideable_constraint(
            dest,
            tag,
            &ValQuot::Extract(MetaVar::new_var_name(&tuple_name), idx),
            info,
            env,
        )?;
    }
    Ok(env)
}

/// Unifies nodes of a schedule with that of the value specification. Also
/// unifies the output classes of the schedule based on the arguments to the
/// final return statement.
/// # Arguments
/// * `cfg` - The cfg
/// * `specs` - The specs
/// * `ctx` - The context
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment and a map from block id to select node name if the block contains
/// a deduced select statement, or an error if the unification fails
fn unify_nodes(
    cfg: &Cfg,
    ctx: &Context,
    fn_info: Info,
    dtypes: &HashMap<String, DataType>,
    mut env: NodeEnv,
) -> Result<(NodeEnv, HashMap<usize, String>), LocalError> {
    let pretinuations = compute_pretinuations(cfg);
    let mut selects = HashMap::new();
    for block in cfg.blocks.values() {
        for stmt in &block.stmts {
            env = match stmt {
                HirBody::ConstDecl {
                    lhs, lhs_tag, rhs, info, ..
                } => unify_decl(lhs, lhs_tag, rhs, *info, env)?,
                HirBody::VarDecl {
                    lhs, lhs_tag, rhs, info, ..
                } => {
                    if let Some(rhs) = rhs {
                        unify_decl(lhs, lhs_tag, rhs, *info, env)?
                    } else {
                        env
                    }
                }
                HirBody::RefStore {
                    lhs, lhs_tags, rhs, info, ..
                } => unify_decl(lhs, lhs_tags, rhs, *info, env)?,
                HirBody::RefLoad { dest, src, info, .. } => {
                    add_var_constraint(dest, src, *info, env)?
                }
                HirBody::InAnnotation(info, tags) | HirBody::OutAnnotation(info, tags) => {
                    for (name, tag) in tags {
                        env = add_type_annot(name, tag, *info, env)?;
                    }
                    env
                }
                HirBody::Op {
                    info,
                    dests,
                    op,
                    args,
                } => unify_op(dests, op, args, *info, ctx, env)?,
                HirBody::Phi { dest, inputs, .. }
                    if !matches!(
                        dtypes.get(&ssa::original_name(dest)),
                        Some(DataType::Fence(_) | DataType::Encoder(_) | DataType::Event),
                    ) =>
                {
                    unify_phi(
                        dest,
                        inputs,
                        &pretinuations,
                        cfg,
                        block.id,
                        &mut selects,
                        env,
                    )?
                }
                HirBody::DeviceCopy {
                    dest,
                    src,
                    info,
                    dest_tag,
                    ..
                } => unify_decl(
                    dest,
                    dest_tag,
                    &SchedTerm::Var {
                        info: *info,
                        name: src.clone(),
                        tag: None,
                    },
                    *info,
                    env,
                )?,
                HirBody::EncodeDo { dests, func, info, .. } => unify_call(dests, func, ctx, dtypes, *info, env)?,
                HirBody::Sync { dests, srcs, info, ..} => {
                    assert_eq!(dests.processed().len() + 1, srcs.processed().len());
                    for ((dest, dest_tag), src) in dests.processed().iter().zip(srcs.processed().iter().skip(1)) {
                        env = unify_decl(dest, dest_tag, &SchedTerm::Var { name: src.clone(), info: *info, tag: None }, fn_info, env)?;
                    }
                    env
                }
                HirBody::BeginEncoding { .. }
                | HirBody::Submit { .. }
                | HirBody::Hole(..)
                // ignore PHIs for non-value types
                | HirBody::Phi { .. } => env,
            }
        }
        env = unify_terminator(block, ctx, fn_info, dtypes, env)?;
    }
    Ok((env, selects))
}

/// Unifies the terminator of a basic block with the value specification
fn unify_terminator(
    block: &BasicBlock,
    ctx: &Context,
    fn_info: Info,
    dtypes: &HashMap<String, DataType>,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    match &block.terminator {
        Terminator::CaptureCall { dests, call, .. } => {
            unify_call(dests, call, ctx, dtypes, fn_info, env)
        }
        Terminator::Call(..) => unreachable!(),
        Terminator::Return {
            dests, rets, info, ..
        } => {
            // pass through is ignored (like next)
            // the destination tag is the tag for the merged node, we handle this
            for ((dest, _), ret) in dests.iter().zip(rets.iter()) {
                env = add_var_constraint(dest, ret, *info, env)?;
            }
            Ok(env)
        }
        Terminator::FinalReturn(info, ret_names) => {
            let output_classes: Vec<_> = env.get_function_output_classes().to_vec();
            for (idx, (ret_name, class)) in ret_names
                .iter()
                .filter(|rname| {
                    is_value_dtype(dtypes.get(&ssa::original_name(rname)).unwrap_or_else(|| {
                        panic!("{info}: Missing dtype for {}", ssa::original_name(rname))
                    }))
                })
                .zip(output_classes.into_iter())
                .enumerate()
            {
                if let Some(func_class) = class {
                    if idx < env.get_spec_output_classes().len()
                        && env.get_spec_output_classes()[idx] == func_class
                    {
                        env = add_constraint(
                            &format!("{ret_name}!"),
                            &ValQuot::Output(MetaVar::new_var_name(ret_name)),
                            fn_info,
                            env,
                        )?;
                        env = add_node_eq(&format!("{ret_name}!"), &func_class, *info, env)?;
                    } else {
                        env = add_node_eq(ret_name, &func_class, fn_info, env)?;
                    }
                }
            }
            Ok(env)
        }
        Terminator::Select { .. }
        | Terminator::None(..)
        | Terminator::Next(..)
        | Terminator::Yield(..) => Ok(env),
    }
}

/// Fills the value quotient spec node id in `tag` for `name`. If the quotient is unspecified,
/// The deduced quotient will always be `node` unless the variable is an input,
/// in which case it will be `input`.
///
/// Does nothing if the environement does not contain `name`.
/// # Arguments
/// * `name` - The name of the variable
/// * `tag` - The tag to fill
/// * `env` - The current environment
/// * `specs` - The specs
/// # Panics
/// If the value quotient spec id is already filled with a value that
/// conflicts with the information in `env`.
fn fill_val_quotient(name: &str, tag: &mut TripleTag, env: &NodeEnv, block_id: usize) {
    fill_quotient(name, tag, env, block_id, SpecType::Value, false, &|dt| {
        &mut dt.value
    });
}

/// Constructs a new triple tag based on information from the environment.
/// Any information the environment does not have is left as `None`.
fn construct_new_tag(name: &str, env: &NodeEnv, block_id: usize) -> TripleTag {
    env.get_node_name(name)
        .map_or_else(TripleTag::new_unspecified, |node| TripleTag {
            value: Tag {
                quot: Some(
                    if env.get_input_classes().contains(&node) && block_id == START_BLOCK_ID {
                        Quotient::Input
                    } else {
                        Quotient::Node
                    },
                ),
                quot_var: QuotientReference {
                    spec_var: Some(node),
                    spec_type: SpecType::Value,
                },
                flow: None,
            },
            spatial: Tag::new_unspecified(SpecType::Spatial),
            timeline: Tag::new_unspecified(SpecType::Timeline),
        })
}

/// Fills the value quotient spec ids in the tags for the all variables in
/// the cfg, with the result of the unification. If the quotient deduction
/// could not deduce a spec id for a particular variable, the spec id will
/// not be changed. The quotients will always be `node` unless the variable
/// is an input, in which case it will be `input`.
/// # Arguments
/// * `env` - The current environment and result of the unification
/// * `cfg` - The cfg (mutated)
/// * `specs` - The specs
/// * `selects` - A map from each block with a select node to the name of the spec variable
/// which maps to the select node.
fn fill_type_info(env: &NodeEnv, cfg: &mut Cfg, selects: &HashMap<usize, String>) {
    // eprintln!("{env:#?}");
    for block in cfg.blocks.values_mut() {
        let mut insertions = vec![];
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            match stmt {
                HirBody::ConstDecl { lhs, lhs_tag, .. }
                | HirBody::VarDecl { lhs, lhs_tag, .. }
                | HirBody::RefStore {
                    lhs,
                    lhs_tags: lhs_tag,
                    ..
                } => {
                    fill_val_quotient(lhs, lhs_tag, env, block.id);
                }
                HirBody::Op { dests, .. } => {
                    for (d, t) in dests {
                        fill_val_quotient(d, t, env, block.id);
                    }
                }
                HirBody::InAnnotation(_, tags) | HirBody::OutAnnotation(_, tags) => {
                    for (name, tag) in tags {
                        fill_val_quotient(name, tag, env, block.id);
                    }
                }
                HirBody::EncodeDo { dests, func, .. } => {
                    fill_val_quotient(
                        &tuple_id(&dests.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>()),
                        &mut func.tag,
                        env,
                        block.id,
                    );
                    for (dest, tag) in dests {
                        fill_val_quotient(dest, tag, env, block.id);
                    }
                }
                HirBody::DeviceCopy { dest, dest_tag, .. } => {
                    fill_val_quotient(dest, dest_tag, env, block.id);
                }
                HirBody::Hole(_)
                | HirBody::RefLoad { .. }
                | HirBody::BeginEncoding { .. }
                | HirBody::Submit { .. } => {}
                HirBody::Phi { dest, info, .. } => {
                    insertions.push((
                        idx,
                        HirBody::InAnnotation(
                            *info,
                            vec![(dest.clone(), construct_new_tag(dest, env, block.id))],
                        ),
                    ));
                }
                HirBody::Sync { dests, .. } => {
                    for (dest, dest_tag) in dests.processed_mut() {
                        fill_val_quotient(dest, dest_tag, env, block.id);
                    }
                }
            }
        }
        match &mut block.terminator {
            Terminator::CaptureCall { dests, call, .. } => {
                for (dest, tag) in dests.iter_mut() {
                    fill_val_quotient(dest, tag, env, block.id);
                }
                fill_val_quotient(
                    &tuple_id(&dests.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>()),
                    &mut call.tag,
                    env,
                    block.id,
                );
            }
            Terminator::Select { dests, tag, .. } => {
                for (dest, tag) in dests {
                    fill_val_quotient(dest, tag, env, block.id);
                }
                fill_val_quotient(&selects[&block.id], tag, env, block.id);
            }
            Terminator::Call(..) | Terminator::None(..) => unreachable!(),
            // I think this is right bc returns to parents should be handled
            // by Phi nodes
            Terminator::Next(..)
            | Terminator::FinalReturn(..)
            | Terminator::Return { .. }
            | Terminator::Yield(..) => {}
        }
        while let Some((idx, stmt)) = insertions.pop() {
            block.stmts.insert(idx, stmt);
        }
    }
}

/// Fills the value quotient information in the type information for the inputs and outputs
/// of the schedule.
/// # Arguments
/// * `inputs` - The names of the input variables
/// * `outputs` - The type information for the outputs
/// * `env` - The current environment
fn fill_io_type_info(
    inputs: &mut [(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    env: &NodeEnv,
) {
    for (name, tag) in inputs.iter_mut() {
        fill_val_quotient(name, tag, env, START_BLOCK_ID);
    }
    let output_classes = env.get_function_output_classes().to_vec();
    for (tag, output_class) in outputs
        .iter_mut()
        .zip(output_dtypes.iter())
        .filter_map(|(t, dt)| if is_value_dtype(dt) { Some(t) } else { None })
        .zip(output_classes)
    {
        if tag.value.quot.is_none() {
            tag.value.quot = Some(Quotient::Node);
        }
        if let Some(output_class) = output_class {
            fill_val_quotient(
                &MetaVar::new_class_name(&output_class).into_string(),
                tag,
                env,
                START_BLOCK_ID,
            );
        }
    }
}
