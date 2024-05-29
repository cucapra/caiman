use std::collections::HashMap;

use crate::{
    error::{Info, LocalError},
    lower::sched_hir::{
        cfg::{Cfg, START_BLOCK_ID},
        HirBody, TripleTag,
    },
    parse::ast::{DataType, Quotient},
    typing::{is_timeline_dtype, Context, NodeEnv, SpecInfo, ValQuot},
};

pub fn deduce_tmln_quots(
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
    )?;
    Ok(())
}

/// Adds constraints to the environment based on input and output annotations.
/// Any unspecified annotations are going to be assumed to match up with the
/// spec. Requires that the input and output variables of a given dimension
/// (timeline, value, etc.) are kept in the same relative order as the spec.
fn add_io_constraints(
    mut env: NodeEnv,
    inputs: &mut [(String, TripleTag)],
    input_overrides: &[(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    dtypes: &HashMap<String, DataType>,
    info: Info,
) -> Result<NodeEnv, LocalError> {
    env.override_output_classes(
        output_dtypes.iter().zip(outputs.iter().map(|t| &t.value)),
        &is_timeline_dtype,
    );
    for (name, tag) in input_overrides {
        for (n2, t2) in inputs.iter_mut() {
            if n2 == name {
                t2.set_specified_info(tag.clone());
            }
        }
    }
    for (idx, (arg_name, fn_in_tag)) in inputs
        .iter()
        .filter(|(arg, _)| is_timeline_dtype(&dtypes[arg]))
        .enumerate()
    {
        if fn_in_tag.timeline.quot == Some(Quotient::None) {
            continue;
        }
        let class_name = if let Some(annoted_quot) = &fn_in_tag.timeline.quot_var.spec_var {
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
    super::add_overrideable_constraint(lhs, lhs_tag, rhs, info, env, &|dt| &dt.timeline)
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
    super::add_type_annot(name, annot, info, env, &|dt| &dt.timeline)
}
