mod tmln_typing;
mod val_typing;

pub use tmln_typing::deduce_tmln_quots;
pub use tmln_typing::*;
pub use val_typing::deduce_val_quots;

use crate::{
    error::{type_error, Info, LocalError},
    lower::sched_hir::TripleTag,
    parse::ast::{Quotient, Tag},
    typing::{NodeEnv, ValQuot},
};
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
    env.add_constraint(lhs, rhs).map_err(|e| {
        type_error(
            info,
            &format!("Failed to unify node constraints of {lhs}:\n {e}"),
        )
    })?;
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
    dimension_getter: &dyn Fn(&TripleTag) -> &Tag,
) -> Result<NodeEnv, LocalError> {
    if matches!(dimension_getter(lhs_tag).quot, Some(Quotient::None)) {
        return Ok(env);
    }
    if let Some(annot) = &dimension_getter(lhs_tag).quot_var.spec_var {
        if let Some(class_constraint) = env.get_spec_node(annot) {
            if !class_constraint.alpha_equiv(&From::from(rhs)) {
                return Ok(env);
            }
        }
    }
    add_constraint(lhs, rhs, info, env)
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
        .map_err(|e| type_error(info, &format!("Failed to unify {lhs} with {var}:\n {e}")))?;
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
    env.add_node_eq(name, class_name).map_err(|e| {
        type_error(
            info,
            &format!("Failed to unify {name} with node {class_name}:\n {e}"),
        )
    })?;
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
fn add_type_annot(
    name: &str,
    annot: &TripleTag,
    info: Info,
    env: NodeEnv,
    dimension_getter: &dyn Fn(&TripleTag) -> &Tag,
) -> Result<NodeEnv, LocalError> {
    if let Some(class_name) = &dimension_getter(annot).quot_var.spec_var {
        add_node_eq(name, class_name, info, env)
    } else {
        Ok(env)
    }
}
