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
//! compression, it's linear.
//! TODO: once explication is added, add a timeout that will chuck anything
//! that takes too long to unify to the explicator.
//!
//! Right now, the algorithm will keep trying for eternity to unify the trees.
//! Also, although not implemented right now, any thing that doesn't get a
//! concrete type after unification (perhaps due to future black-box operations)
//! will be sent to the explicator.
//!
//! The way we do unification is to add *class names* to the equivalence classes
//! in the union-find data structure. The class names are the names of the
//! value spec nodes that the schedule nodes are equivalent to. This allows us
//! to get the canonical representative (spec node id) of any equivalence class.

use std::collections::HashMap;

use crate::{
    error::{type_error, Info, LocalError},
    lower::{
        sched_hir::{
            cfg::{Cfg, Edge, START_BLOCK_ID},
            HirBody, HirFuncCall, HirOp, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{Binop, Quotient, QuotientReference, SchedLiteral, SchedTerm, SpecType, Tag},
    typing::{Context, MetaVar, NodeEnv, SchedOrExtern, SpecInfo, ValQuot},
};

use super::continuations::compute_pretinuations;

/// Deduces the quotients for the value specification. Returns an error
/// if unification fails, otherwise, writes the deduced quotients to the tags
/// of the instructions in the cfg.
pub fn deduce_val_quots(
    inputs: &mut [(String, TripleTag)],
    outputs: &mut [TripleTag],
    cfg: &mut Cfg,
    spec_info: &SpecInfo,
    ctx: &Context,
) -> Result<(), LocalError> {
    let env = spec_info.nodes.clone();
    let (env, selects) = unify_nodes(
        inputs.iter().map(|(name, _)| name),
        cfg,
        ctx,
        Info::default(),
        env,
    )?;
    fill_type_info(&env, cfg, &selects);
    fill_io_type_info(inputs, outputs, &env);
    Ok(())
}

/// Adds a type constraint to the environment
/// # Arguments
/// * `lhs` - The name of the variable to constrain
/// * `rhs` - The constraint to apply to the type variable
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment
#[allow(clippy::unnecessary_wraps)]
fn add_constraint(
    lhs: &str,
    rhs: &ValQuot,
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    env.add_constraint(lhs, rhs)
        .map_err(|e| {
            type_error(
                info,
                &format!("Failed to unify node constraints of {lhs}: {e}"),
            )
        })
        .unwrap();
    Ok(env)
}

/// Constrains two type variables to be equal
/// # Arguments
/// * `lhs` - The name of the first variable
/// * `rhs` - The name of the second variable
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment
#[allow(clippy::unnecessary_wraps)]
fn add_var_constraint(
    lhs: &str,
    var: &str,
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    env.add_var_eq(lhs, var)
        .map_err(|e| type_error(info, &format!("Failed to unify {lhs} with {var}: {e}")))
        .unwrap();
    Ok(env)
}

/// Adds a node with the given name to match the class name (spec node id)
/// # Arguments
/// * `name` - The name of the type variable
/// * `class_name` - The name of the class that the type variable must match with
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment
#[allow(clippy::unnecessary_wraps)]
fn add_node_eq(
    name: &str,
    class_name: &str,
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    env.add_node_eq(name, class_name)
        .map_err(|e| {
            type_error(
                info,
                &format!("Failed to unify {name} with node {class_name}: {e}"),
            )
        })
        .unwrap();
    Ok(env)
}

/// Adds a type annotation for `name` to the environement if the given annotation
/// provides a value node matching.
/// # Arguments
/// * `name` - The name of the variable to annotate
/// * `annot` - The annotation to add
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn add_type_annot(name: &str, annot: &TripleTag, env: NodeEnv) -> Result<NodeEnv, LocalError> {
    if let Some(class_name) = &annot.value.quot_var.spec_var {
        add_node_eq(name, class_name, Info::default(), env)
    } else {
        Ok(env)
    }
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
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    match rhs {
        SchedTerm::Lit {
            lit: SchedLiteral::Int(i),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), env)?;
            env = add_constraint(lhs, &ValQuot::Int(i.clone()), *info, env)?;
        }
        SchedTerm::Lit {
            lit: SchedLiteral::Bool(b),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), env)?;
            env = add_constraint(lhs, &ValQuot::Bool(*b), *info, env)?;
        }
        SchedTerm::Lit {
            lit: SchedLiteral::Float(f),
            info,
            tag,
        } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), env)?;
            env = add_constraint(lhs, &ValQuot::Float(f.clone()), *info, env)?;
        }
        SchedTerm::Var { name, info, tag } => {
            env = add_type_annot(lhs, &TripleTag::from_opt(tag), env)?;
            env = add_var_constraint(lhs, name, *info, env)?;
        }
        x => todo!("{x:#?}"),
    }
    add_type_annot(lhs, lhs_tag, env)
}

/// Converts an `HirOp` to a `Binop`
/// # Panics
/// If the `HirOp` is not a binary operator
fn hir_op_to_binop(op: &HirOp) -> Binop {
    match op {
        HirOp::Binary(binop) => *binop,
        HirOp::FFI(name) => {
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
    dest: &str,
    dest_tag: &TripleTag,
    op: &HirOp,
    args: &[SchedTerm],
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    let mut arg_names = vec![];
    for arg in args {
        match arg {
            SchedTerm::Var { name, tag, .. } => {
                env = add_type_annot(name, &TripleTag::from_opt(tag), env)?;
                arg_names.push(name.clone());
            }
            _ => unreachable!(),
        }
    }
    env = add_constraint(
        dest,
        &ValQuot::Bop(
            hir_op_to_binop(op),
            MetaVar::new_var_name(&arg_names[0]),
            MetaVar::new_var_name(&arg_names[1]),
        ),
        info,
        env,
    )?;
    add_type_annot(dest, dest_tag, env)
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
    if let Terminator::Select { guard, .. } = &split_block.terminator {
        if let Edge::Select {
            true_branch,
            false_branch,
        } = cfg.graph[&split_point]
        {
            // TODO: Info
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
                    Info::default(),
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
                    Info::default(),
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
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    // TODO: info
    let val_spec = value_name(&call.target, ctx);
    let f_class = ctx.specs[&val_spec].feq.clone().unwrap();
    let tuple_name = tuple_id(
        &dests
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>(),
    );
    env = add_type_annot(&tuple_name, &call.tag, env)?;
    env = add_constraint(
        &tuple_name,
        &ValQuot::Call(
            f_class,
            call.args.iter().map(MetaVar::new_var_name).collect(),
        ),
        Info::default(),
        env,
    )?;
    for (idx, (dest, tag)) in dests.iter().enumerate() {
        env = add_type_annot(dest, tag, env)?;
        env = add_constraint(
            dest,
            &ValQuot::Extract(MetaVar::new_var_name(&tuple_name), idx),
            Info::default(),
            env,
        )?;
    }
    Ok(env)
}

/// Unifies nodes of a schedule with that of the value specification
/// # Arguments
/// * `inputs` - The names of the input variables
/// * `cfg` - The cfg
/// * `specs` - The specs
/// * `ctx` - The context
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment and a map from block id to select node name if the block contains
/// a deduced select statement, or an error if the unification fails
fn unify_nodes<'a, T: Iterator<Item = &'a String>>(
    inputs: T,
    cfg: &Cfg,
    ctx: &Context,
    info: Info,
    mut env: NodeEnv,
) -> Result<(NodeEnv, HashMap<usize, String>), LocalError> {
    let pretinuations = compute_pretinuations(cfg);
    let mut selects = HashMap::new();
    for (in_name, class_name) in inputs.zip(env.get_input_classes().to_vec()) {
        env = add_node_eq(in_name, &class_name, info, env)?;
    }
    for block in cfg.blocks.values() {
        for stmt in &block.stmts {
            env = match stmt {
                HirBody::ConstDecl {
                    lhs, lhs_tag, rhs, ..
                } => unify_decl(lhs, lhs_tag, rhs, env)?,
                HirBody::VarDecl {
                    lhs, lhs_tag, rhs, ..
                } => {
                    if let Some(rhs) = rhs {
                        unify_decl(lhs, lhs_tag, rhs, env)?
                    } else {
                        env
                    }
                }
                HirBody::RefStore {
                    lhs, lhs_tags, rhs, ..
                } => unify_decl(lhs, lhs_tags, rhs, env)?,
                HirBody::RefLoad { dest, src, .. } => {
                    add_var_constraint(dest, src, Info::default(), env)?
                }
                HirBody::InAnnotation(_, tags) | HirBody::OutAnnotation(_, tags) => {
                    for (name, tag) in tags {
                        env = add_type_annot(name, tag, env)?;
                    }
                    env
                }
                HirBody::Hole(_) => env,
                HirBody::Op {
                    info,
                    dest,
                    dest_tag,
                    op,
                    args,
                } => unify_op(dest, dest_tag, op, args, *info, env)?,
                HirBody::Phi { dest, inputs, .. } => unify_phi(
                    dest,
                    inputs,
                    &pretinuations,
                    cfg,
                    block.id,
                    &mut selects,
                    env,
                )?,
            }
        }
        env = match &block.terminator {
            Terminator::CaptureCall { dests, call, .. } => unify_call(dests, call, ctx, env)?,
            Terminator::Call(..) => unreachable!(),
            Terminator::Return { dests, rets, .. } => {
                // pass through is ignored (like next)
                // the destination tag is the tag for the merged node, we handle this
                for ((dest, _), ret) in dests.iter().zip(rets.iter()) {
                    env = add_var_constraint(dest, ret, Info::default(), env)?;
                }
                env
            }
            Terminator::FinalReturn(ret_names) => {
                let output_classes: Vec<_> = env.get_output_classes().to_vec();
                assert_eq!(ret_names.len(), output_classes.len());
                for (ret_name, class) in ret_names.iter().zip(output_classes.into_iter()) {
                    env = add_constraint(
                        &format!("{ret_name}!"),
                        &ValQuot::Output(MetaVar::new_var_name(ret_name)),
                        info,
                        env,
                    )?;
                    env = add_node_eq(&format!("{ret_name}!"), &class, Info::default(), env)?;
                }
                env
            }
            Terminator::Select { .. }
            | Terminator::None
            | Terminator::Next(..)
            | Terminator::Yield(_) => env,
        }
    }
    Ok((env, selects))
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
    if let Some(node) = env.get_node_name(name) {
        let quot = tag.value.quot;
        let flow = tag.value.flow;
        let old_spec_var = tag.value.quot_var.spec_var.as_ref();
        assert!(
            old_spec_var.is_none() || old_spec_var.unwrap() == &node,
            "Cannot unify output class {name} with unequal nodes {node} and {}",
            old_spec_var.unwrap()
        );
        tag.value = Tag {
            quot: Some(quot.unwrap_or_else(|| {
                if env.get_input_classes().contains(&node) && block_id == START_BLOCK_ID {
                    Quotient::Input
                } else {
                    Quotient::Node
                }
            })),
            quot_var: QuotientReference {
                spec_var: Some(node),
                spec_type: SpecType::Value,
            },
            flow,
        };
    }
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
                }
                | HirBody::Op {
                    dest: lhs,
                    dest_tag: lhs_tag,
                    ..
                } => {
                    fill_val_quotient(lhs, lhs_tag, env, block.id);
                }
                HirBody::InAnnotation(_, tags) | HirBody::OutAnnotation(_, tags) => {
                    for (name, tag) in tags {
                        fill_val_quotient(name, tag, env, block.id);
                    }
                }
                HirBody::Hole(_) | HirBody::RefLoad { .. } => {}
                HirBody::Phi { dest, .. } => {
                    insertions.push((
                        idx,
                        HirBody::InAnnotation(
                            Info::default(),
                            vec![(dest.clone(), construct_new_tag(dest, env, block.id))],
                        ),
                    ));
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
            Terminator::Call(..) | Terminator::None => unreachable!(),
            // TODO: check the return, I think this is right bc returns should be handled
            // by Phi nodes
            Terminator::Next(..)
            | Terminator::FinalReturn(..)
            | Terminator::Return { .. }
            | Terminator::Yield(_) => {}
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
fn fill_io_type_info(inputs: &mut [(String, TripleTag)], outputs: &mut [TripleTag], env: &NodeEnv) {
    for (name, tag) in inputs.iter_mut() {
        fill_val_quotient(name, tag, env, START_BLOCK_ID);
    }
    let output_classes = env.get_output_classes().to_vec();
    assert_eq!(output_classes.len(), outputs.len());
    for (tag, output_class) in outputs.iter_mut().zip(output_classes) {
        tag.value.quot = Some(Quotient::Node);
        fill_val_quotient(
            &MetaVar::new_class_name(&output_class).into_string(),
            tag,
            env,
            START_BLOCK_ID,
        );
    }
}
