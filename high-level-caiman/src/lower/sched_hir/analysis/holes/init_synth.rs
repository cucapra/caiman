//! The frontend "meta-synthesizer" that sort of optimally identifies where to
//! intialize mutable variables. Once we identify where best to initialize variables
//! we can guide the explicator to generate initializtion code where we want by
//! setting the value return flow to `usable` of the initializing hole's basic
//! block.
//!
//! We first place initializations points as late as possible, under the heuristic
//! that placing them earlier can result in initializations that need to generate
//! dependencies that the we or the user may have placed later in the program.
//!
//! We are guaranteed that if an initializing set exists for a variable, then
//! this first step will find some initializing set for that variable.
//!
//! Next, we iteratively hoist initializations to earlier program points that
//! only require a subset of the unavailable dependent nodes of the original
//! initialization point. Unavailable dependent nodes would need to be recreated by
//! the explicator. This is sort of like a partial redundancy elimination.
//!
//! If we iterate until convergence in the hoist step, we get, for each variable,
//! an initializing set such that for any other initializing set, there exists a path
//! requiring generation of a set of dependent nodes that is not a subset of the
//! dependent nodes we need to generate. Colloqially, that is to say every other
//! initializing set is potentially worse on some path. In that sense, we achieve
//! a kind of local optimum.

use std::collections::{HashMap, HashSet};

use crate::{
    lower::sched_hir::{
        analysis::DomInfo,
        cfg::{can_reach_goal, Cfg, CollectiveDom, Loc, FINAL_BLOCK_ID, START_BLOCK_ID},
        Hir, HirBody, HirInstr, Terminator, TripleTag,
    },
    parse::ast::{DataType, FullType, SpecType},
    typing::NodeEnv,
};

use super::get_usable_uses;

/// Gets the locations of all holes in topological order.
fn hole_topo(cfg: &Cfg) -> Vec<Loc> {
    let mut res = vec![];
    for block_id in &cfg.topo_order {
        for (local_id, instr) in cfg.blocks[block_id].stmts.iter().enumerate() {
            if let HirBody::Hole { .. } = instr {
                res.push(Loc(*block_id, local_id));
            }
        }
    }
    res
}

/// Gets the inital set of variables to initialize. Variables which are returned `dead`
/// from the function are not initialized.
fn get_inital_to_init_set(
    cfg: &Cfg,
    to_init: &HashSet<String>,
    output: &[FullType],
) -> HashSet<String> {
    let mut to_init: HashSet<_> = to_init.iter().cloned().collect();
    if let Terminator::FinalReturn(_, rets) = &cfg.blocks[&FINAL_BLOCK_ID].terminator {
        assert_eq!(rets.len(), output.len());
        for (ret, out) in rets.iter().zip(output) {
            for tag in &out.tags {
                if tag.quot_var.spec_type == SpecType::Value
                    && tag.flow == Some(crate::parse::ast::Flow::Dead)
                {
                    to_init.remove(ret);
                }
            }
        }
    }
    to_init
}

/// Builds an initializer set for each variable that needs to be initialized.
/// This algorithm will place initialization points at the last possible position.
/// If there exists an initializing set
pub fn build_init_set(
    cfg: &mut Cfg,
    to_init: &HashSet<String>,
    env: &NodeEnv,
    output: &[FullType],
    inputs: &[(String, TripleTag)],
    dtypes: &HashMap<String, DataType>,
    dinfo: &DomInfo,
) -> HashMap<String, HashSet<Loc>> {
    let mut res: HashMap<_, HashSet<_>> = HashMap::new();
    let mut to_init = get_inital_to_init_set(cfg, to_init, output);
    for v in &to_init {
        res.insert(v.clone(), HashSet::new());
    }
    let defs = get_defs(cfg, inputs);
    let uses = get_val_uses(cfg, env, dtypes, dinfo, &defs);
    for hole_loc in hole_topo(cfg) {
        let mut to_remove = vec![];
        for var in &to_init {
            if defs[var].dom(dinfo, &hole_loc) {
                let mut no_use_dom = true;
                for u in &uses[var] {
                    if u.dom(dinfo, &hole_loc) {
                        no_use_dom = false;
                        break;
                    }
                }
                if no_use_dom && can_reach_goal(cfg, &hole_loc, &res[var], &uses[var]) {
                    res.get_mut(var).unwrap().insert(hole_loc.clone());
                }
            }
            if res[var].cdom(cfg, &uses[var]) {
                to_remove.push(var.clone());
            }
        }
        for var in to_remove {
            to_init.remove(&var);
        }
    }
    res
}

/// Places initializations of each variable in the cfg holes' `initialized` set.
pub fn fill_initializers(cfg: &mut Cfg, init_sets: HashMap<String, HashSet<Loc>>) {
    for (var, inits) in init_sets {
        for init in inits {
            if let HirBody::Hole { initialized, .. } =
                &mut cfg.blocks.get_mut(&init.0).unwrap().stmts[init.1]
            {
                initialized.insert(var.clone());
            } else {
                unreachable!("Initialization location is not a hole!");
            }
        }
    }
}

/// Gets a map mapping each variable to the locations they are defined.
fn get_defs(cfg: &Cfg, inputs: &[(String, TripleTag)]) -> HashMap<String, Loc> {
    let mut res = HashMap::new();
    for (str, _) in inputs {
        res.insert(str.clone(), Loc(START_BLOCK_ID, 0));
    }
    for block in cfg.blocks.values() {
        for (local_id, instr) in block.stmts.iter().enumerate() {
            if let Some(defs) = instr.get_defs() {
                for def in defs {
                    assert!(!res.contains_key(&def));
                    res.insert(def, Loc(block.id, local_id));
                }
            }
        }
        if let Some(defs) = block.terminator.get_defs() {
            for def in defs {
                assert!(!res.contains_key(&def));
                res.insert(def, Loc(block.id, block.stmts.len()));
            }
        }
    }
    res
}

/// Returns a map mapping each variable to the locations their value is used.
fn get_val_uses(
    cfg: &mut Cfg,
    env: &NodeEnv,
    dtypes: &HashMap<String, DataType>,
    dinfo: &DomInfo,
    defs: &HashMap<String, Loc>,
) -> HashMap<String, HashSet<Loc>> {
    let mut res: HashMap<_, HashSet<_>> = HashMap::new();
    for block in cfg.blocks.values_mut() {
        // we consider a block that postdominates a use and is "in-scope" to be a use.
        for (var, uses) in &mut res {
            let block_start = Loc(block.id, 0);
            let mut to_add = false;
            for u in uses.iter() {
                if block_start.pdom(dinfo, u)
                    && defs
                        .get(var)
                        .map_or_else(|| false, |loc| loc.dom(dinfo, &block_start))
                {
                    to_add = true;
                    break;
                }
            }
            if to_add {
                uses.insert(block_start);
            }
        }
        for (local_id, instr) in block
            .stmts
            .iter_mut()
            .map(HirInstr::Stmt)
            .chain(std::iter::once(HirInstr::Tail(&mut block.terminator)))
            .enumerate()
        {
            let uses = get_usable_uses(&instr, env, dtypes, |_| {});
            for u in uses {
                res.entry(u).or_default().insert(Loc(block.id, local_id));
            }
        }
    }
    res
}
