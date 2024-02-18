//! This module provides functions for transforming a cfg into SSA form.
//! Our SSA form is a bit different from the standard SSA form in that
//! we consider a reference store to be a definition of the variable being
//! stored to. This is because a reference store may change the quotient
//! of a variable, and thus, we want it to have a different name so we can
//! refer to it independently from the version of the variable before the
//! store.
//!
//! Mostly taken from
//! [here](https://github.com/stephenverderame/cs6120-bril/blob/main/ssa/src/main.rs)
//!
//! ## Why two SSA passes?
//! Please see the comment in high-level-caiman/src/normalize/sched_rename.rs

#![allow(clippy::module_name_repetitions)]
use std::collections::HashMap;

use regex::Regex;

use crate::lower::sched_hir::{
    cfg::{Cfg, START_BLOCK_ID},
    Hir, HirBody, UseType,
};

use super::{
    dominators::{compute_dominators, DomTree},
    InOutFacts, LiveVars,
};

/// Inserts a phi node as the first instruction of a block if one with the
/// same destination does not already exist. If one does exist, nothing
/// happens.
///
/// The phi node is inserted with the incoming blocks set as all blocks
/// that are predecessors of the block and that have the variable as live-out.
/// # Arguments
/// * `cfg` - The cfg
/// * `block` - The block to insert the phi node into
/// * `var` - The variable to insert the phi node for
/// # Returns
/// * True if a phi node was inserted, false otherwise
fn add_phi_to_block(
    cfg: &mut Cfg,
    block: usize,
    var: &str,
    live_vars: &InOutFacts<LiveVars>,
) -> bool {
    let inputs: HashMap<_, _> = cfg
        .predecessors(block)
        .iter()
        .filter(|pred| live_vars.get_out_fact(**pred).live_set.contains(var))
        .map(|pred| (*pred, var.to_string()))
        .collect();
    if inputs.is_empty() {
        return false;
    }
    let bb = cfg.blocks.get_mut(&block).unwrap();
    for instr in &bb.stmts {
        if let HirBody::Phi { original, .. } = instr {
            if original == var {
                return false;
            }
        }
    }
    bb.stmts.insert(
        0,
        HirBody::Phi {
            dest: var.to_string(),
            inputs,
            original: var.to_string(),
        },
    );
    true
}

/// Gets a map from variable name to the blocks that define each variable.
/// Function arguments are not considered, and we consider ref stores to be
/// definitions of the variable being stored to.
/// # Arguments
/// * `cfg` - The cfg
/// # Returns
/// * A map from variable name to a vector of tuples of defining block ids
fn find_vars(cfg: &Cfg) -> HashMap<String, Vec<usize>> {
    let mut res: HashMap<_, Vec<_>> = HashMap::new();
    for (blk_id, block) in &cfg.blocks {
        for stmt in &block.stmts {
            if let Some(defs) = stmt.get_defs() {
                for def in defs {
                    res.entry(def.clone()).or_default().push(*blk_id);
                }
            }
            if let HirBody::RefStore { lhs, .. } = stmt {
                res.entry(lhs.clone()).or_default().push(*blk_id);
            }
        }
        if let Some(defs) = block.terminator.get_defs() {
            for def in &defs {
                res.entry(def.clone()).or_default().push(*blk_id);
            }
        }
    }
    res
}

/// Inserts phi nodes into the cfg.
fn add_phi_nodes(cfg: &mut Cfg, doms: &DomTree, live_vars: &InOutFacts<LiveVars>) {
    let vars = find_vars(cfg);
    for (var, mut def_blocks) in vars {
        while let Some(def_block) = def_blocks.pop() {
            for dom in doms.dom_frontier(def_block, cfg) {
                if add_phi_to_block(cfg, dom, &var, live_vars) {
                    def_blocks.push(dom);
                }
            }
        }
    }
}

/// Renames a definition by appending 1+ the number of times the variable has been
/// defined to the end of the variable name. If the variable has not been
/// defined before, `name.0` is returned. Updates the `cur_names` map
/// with the new number of times the variable has been defined.
fn rename_def(
    name: &str,
    cur_names: &mut HashMap<String, u64>,
    latest_names: &mut HashMap<String, u64>,
) -> String {
    let new_id = latest_names
        .entry(name.to_string())
        .and_modify(|x| *x += 1)
        .or_insert(0);
    cur_names.insert(name.to_string(), *new_id);
    format!("{name}.{new_id}")
}

/// Renames a use by appending the number of times the variable has been
/// defined to the end of the variable name. If the variable has not been
/// defined, `name` is returned.
fn rename_use(name: &str, cur_names: &HashMap<String, u64>) -> String {
    cur_names
        .get(name)
        .map_or_else(|| name.to_string(), |id| format!("{name}.{id}"))
}

/// Renames variables in the cfg to ensure that each variable is only defined
/// once in each block. This is done by renaming each variable to
/// `{original}.{id}` where `{id}` is the number of times the variable has been
/// defined in the function.
///
/// For our purposes, we consider a reference store to be a definition of the
/// variable being stored to.
/// # Arguments
/// * `cfg` - The cfg
/// * `block` - The block to rename variables in
/// * `cur_names` - The map from original name to current id of the variable
/// * `latest_names` - The map from original name to latest id of the variable
fn ssa_rename_vars(
    cfg: &mut Cfg,
    block: usize,
    mut cur_names: HashMap<String, u64>,
    latest_names: &mut HashMap<String, u64>,
    doms: &DomTree,
) {
    let bb = cfg.blocks.get_mut(&block).unwrap();
    for stmt in &mut bb.stmts {
        // rename reads first (true uses)
        stmt.rename_uses(&mut |name, use_type| {
            if use_type == UseType::Read {
                rename_use(name, &cur_names)
            } else {
                name.to_string()
            }
        });
        // consider a write as a def
        stmt.rename_uses(&mut |name, use_type| {
            if use_type == UseType::Write {
                rename_def(name, &mut cur_names, latest_names)
            } else {
                name.to_string()
            }
        });
        stmt.rename_defs(&mut |name| rename_def(name, &mut cur_names, latest_names));
    }
    bb.terminator.rename_uses(&mut |name, ut| {
        assert_eq!(ut, UseType::Read);
        rename_use(name, &cur_names)
    });
    bb.terminator
        .rename_defs(&mut |name| rename_def(name, &mut cur_names, latest_names));

    // add phi args for successors
    for succ in cfg.successors(block) {
        // if a successor has a phi instruction for a variable that's live-out of
        // this block, add an argument to the phi instruction for this block
        for instr in &mut cfg.blocks.get_mut(&succ).unwrap().stmts {
            if let HirBody::Phi {
                inputs, original, ..
            } = instr
            {
                // only insert incoming variable if phi node has an
                // incoming edge for this block
                if inputs.contains_key(&block) {
                    inputs.insert(block, rename_use(original, &cur_names));
                }
            }
        }
    }

    // recurse on immediate dominated
    for imm_dom in doms.immediately_dominated(block) {
        ssa_rename_vars(cfg, imm_dom, cur_names.clone(), latest_names, doms);
    }
}

/// Transforms a cfg into SSA form. For our purposes, we consider a reference
/// store to be a definition of the variable being stored to. This is because
/// a reference store may change the quotient of a variable, and thus, we want
/// it to have a different name so we can refer to it independently from the
/// version of the variable before the store.
///
/// Each SSA variable is named `{original}.{id}` where `{id}` is the number of
/// times the variable has been defined in the function. Each definition in SSA
/// form is a version of a variable which may have different quotients.
///
/// ## Example (from a source-level representation)
///
/// ```text
/// let x = 1;
/// var v;
/// if x > 0 {
///    let x = x + 1;
///    let c = x < 2;
///    v = x;
/// } else {
///     let x = x - 1;
///     let c = x * 2;
///     v = x;
/// }
/// ```
///
/// becomes:
///
/// ```text
/// let x.0 = 1;
/// var v.0;
/// if x.0 > 0 {
///     let x.1 = x.0 + 1;
///     let c.0 = x.1 < 2;
///     v.1 = x.1;
/// } else {
///     let x.2 = x.0 - 1;
///     let c.1 = x.2 * 2;
///     v.2 = x.2;
/// }
/// v.3 = phi(v.1, v.2);
/// ```
#[must_use]
pub fn transform_to_ssa(mut cfg: Cfg, live_vars: &InOutFacts<LiveVars>) -> Cfg {
    let doms = compute_dominators(&cfg);
    add_phi_nodes(&mut cfg, &doms, live_vars);
    ssa_rename_vars(
        &mut cfg,
        START_BLOCK_ID,
        HashMap::new(),
        &mut HashMap::new(),
        &doms,
    );
    cfg
}

/// Returns the original name of a variable. For example, `x.0` would become
/// `x`. If the variable name is not fromatted correctly, this function will return the
/// passed in name.
fn original_name(name: &str) -> String {
    Regex::new(r"\.\d+$").unwrap().replace(name, "").to_string()
}

/// Transforms out of SSA simply by removing Phi nodes and renaming
/// variables to be their original name. So for example, `x.0` would
/// become `x`.
///
/// Therefore, we assume that SSA form is used only for analysis, and
/// no transformations are done on the SSA form.
///
/// # Example (from a source-level representation)
///
/// ```text
/// let x.0 = 1;
/// var v.0;
/// if x.0 > 0 {
///     let x.1 = x.0 + 1;
///     let c.0 = x.1 < 2;
///     v.1 = x.1;
/// } else {
///     let x.2 = x.0 - 1;
///     let c.1 = x.2 * 2;
///     v.2 = x.2;
/// }
/// v.3 = phi(v.1, v.2);
/// ```
///
/// becomes:
///
/// ```text
/// let x = 1;
/// var v;
/// if x > 0 {
///    let x = x + 1;
///    let c = x < 2;
///    v = x;
/// } else {
///     let x = x - 1;
///     let c = x * 2;
///     v = x;
/// }
/// ```
#[must_use]
pub fn transform_out_ssa(mut cfg: Cfg) -> Cfg {
    for bb in cfg.blocks.values_mut() {
        bb.stmts.retain(|stmt| !matches!(stmt, HirBody::Phi { .. }));
        for stmt in &mut bb.stmts {
            stmt.rename_uses(&mut |name, _| original_name(name));
            stmt.rename_defs(&mut original_name);
        }
        bb.terminator
            .rename_uses(&mut |name, _| original_name(name));
        bb.terminator.rename_defs(&mut original_name);
    }
    cfg
}
