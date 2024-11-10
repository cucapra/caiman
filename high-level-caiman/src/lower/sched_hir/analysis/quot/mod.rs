//! Quotient type checking and type deduction

mod tmln_typing;
mod val_typing;

pub use tmln_typing::deduce_tmln_quots;
pub use val_typing::deduce_val_quots;
pub use val_typing::{fill_fn_input_overrides, fill_val_quots};

use crate::{
    error::{hir_to_source_name, Info, LocalError},
    lower::sched_hir::{cfg::START_BLOCK_ID, TripleTag},
    parse::ast::{Quotient, QuotientReference, SpecType, Tag},
    type_error,
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
        type_error!(
            info,
            "Failed to unify node constraints of '{}':\n {e}",
            hir_to_source_name(lhs)
        )
    })?;
    Ok(env)
}

/// Adds a type constraint to the environment, allowing
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
    env.add_var_eq(lhs, var).map_err(|e| {
        type_error!(
            info,
            "Failed to unify '{}' with '{}':\n {e}",
            hir_to_source_name(lhs),
            hir_to_source_name(var)
        )
    })?;
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
        type_error!(
            info,
            "Failed to unify '{}' with node '{class_name}':\n {e}",
            hir_to_source_name(name)
        )
    })?;
    Ok(env)
}

/// Adds a type annotation for `name` to the environement if the given annotation
/// provides a node matching.
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

/// Fills the quotient spec node id in `tag` for `name`. If the quotient is unspecified,
/// The deduced quotient will always be `node` unless the variable is an input,
/// in which case it will be `input`.
///
/// Does nothing if the environement does not contain `name`.
/// # Arguments
/// * `name` - The name of the variable
/// * `tag` - The tag to fill
/// * `env` - The current environment
/// * `specs` - The specs
/// * `spec_type` - The type of the spec node
/// * `skip_if_filled` - If true, the function will not fill the tag if it is already filled
/// with a value that conflicts with the information in `env`.
/// # Panics
/// If the quotient spec id is already filled with a value that
/// conflicts with the information in `env`.
fn fill_quotient(
    name: &str,
    tag: &mut TripleTag,
    env: &NodeEnv,
    block_id: usize,
    spec_type: SpecType,
    skip_if_filled: bool,
    tag_getter: &dyn Fn(&mut TripleTag) -> &mut Tag,
) {
    if let Some(node) = env.get_node_name(name) {
        let quot = tag_getter(tag).quot;
        let flow = tag_getter(tag).flow;
        let old_spec_var = tag_getter(tag).quot_var.spec_var.as_ref();
        if !skip_if_filled {
            assert!(
                old_spec_var.is_none() || old_spec_var.unwrap() == &node,
                "Cannot fill {spec_type:?} {name} node {node} into {}",
                old_spec_var.unwrap()
            );
        }
        #[allow(clippy::unnecessary_unwrap)]
        let node = if skip_if_filled && old_spec_var.is_some() {
            old_spec_var.unwrap().clone()
        } else {
            node
        };
        *tag_getter(tag) = Tag {
            quot: Some(quot.unwrap_or_else(|| {
                if env.get_input_classes().contains(&node) && block_id == START_BLOCK_ID {
                    Quotient::Input
                } else {
                    Quotient::Node
                }
            })),
            quot_var: QuotientReference {
                spec_var: Some(node),
                spec_type,
            },
            flow,
        };
    }
}
