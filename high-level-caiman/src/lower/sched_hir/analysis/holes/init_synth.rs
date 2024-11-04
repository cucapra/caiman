//! The frontend initialization synthesizer that sort of optimally identifies where to
//! intialize mutable variables. Once we identify where best to initialize variables
//! we can guide the explicator to generate initializtion code where we want by
//! setting the value return flow to `usable` of the initializing hole's basic
//! block.
//!
//! We first place initializations points as late as possible, under the heuristic
//! that placing them earlier can result in initializations that need to generate
//! dependencies that the we or the user may have placed later in the program.
//!
//! If an initializing set exists for a variable, then this first step should
//! find some initializing set for that variable.
//!
//! Next, we iteratively hoist initializations to earlier program points that
//! only require a subset of the unavailable dependent nodes of the original
//! initialization point. Unavailable dependent nodes would need to be recreated by
//! the explicator. This is sort of like a partial redundancy elimination.
//!
//! If we iterate until convergence in the hoist step, we should get, for each variable,
//! an initializing set such that for any other initializing set, there exists a path
//! requiring generation of a set of dependent nodes that is not a subset of the
//! dependent nodes we need to generate. Colloqially, that is to say every other
//! initializing set is potentially worse on some path.

use std::collections::{HashMap, HashSet};

use crate::{
    lower::sched_hir::{
        analysis::DomInfo,
        cfg::{can_reach_goal, Cfg, CollectiveDom, Loc, START_BLOCK_ID},
        Hir, HirBody, HirInstr, TripleTag,
    },
    parse::ast::{DataType, FullType},
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

/// Builds an initializer set for each variable that needs to be initialized.
/// This algorithm will place initialization points at the last possible position.
/// If there exists an initializing set
pub fn build_init_set(
    cfg: &mut Cfg,
    to_init: &HashSet<String>,
    env: &NodeEnv,
    _output: &[FullType],
    inputs: &[(String, TripleTag)],
    dtypes: &HashMap<String, DataType>,
    dinfo: &DomInfo,
) -> HashMap<String, HashSet<Loc>> {
    let mut res: HashMap<_, HashSet<_>> = HashMap::new();
    let mut to_init = to_init.clone();
    for v in &to_init {
        res.insert(v.clone(), HashSet::new());
    }
    // dbg!(&to_init);
    let defs = get_defs(cfg, inputs, &to_init);
    let uses = get_val_uses(cfg, env, dtypes, dinfo, &defs, &to_init);
    // dbg!(&uses);
    let mut worklist = hole_topo(cfg);
    while let Some(hole_loc) = worklist.pop() {
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
                // println!("{var}: {:?} blocks {:?}", res[var], uses[var]);
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
pub fn fill_initializers(cfg: &mut Cfg, init_sets: HashMap<String, HashSet<Loc>>, env: &NodeEnv) {
    for (var, inits) in init_sets {
        for init in inits {
            if let HirBody::Hole { initialized, .. } =
                &mut cfg.blocks.get_mut(&init.0).unwrap().stmts[init.1]
            {
                let val_node = env.get_node_name(&var);
                assert!(!initialized.contains_key(&var) || initialized[&var] == val_node);
                initialized.insert(var.clone(), env.get_node_name(&var));
            } else {
                unreachable!("Initialization location is not a hole!");
            }
        }
    }
}

/// Gets a map mapping each variable to the locations they are defined.
/// Assumes we are in reference-SSA form, so writes to variables are considered defs.
fn get_defs(
    cfg: &Cfg,
    inputs: &[(String, TripleTag)],
    to_init: &HashSet<String>,
) -> HashMap<String, Loc> {
    let mut res = HashMap::new();
    for (str, _) in inputs {
        res.insert(str.clone(), Loc(START_BLOCK_ID, 0));
    }
    let mut add_defs = |defs: Option<Vec<_>>, block_id: usize, local_id: usize| {
        if let Some(defs) = defs {
            for def in defs.into_iter().filter(|x| to_init.contains(x)) {
                assert!(!res.contains_key(&def));
                res.insert(def, Loc(block_id, local_id));
            }
        }
    };
    for block in cfg.blocks.values() {
        for (local_id, instr) in block.stmts.iter().enumerate() {
            add_defs(instr.get_defs(), block.id, local_id);
            add_defs(instr.get_write_uses(), block.id, local_id);
        }
        add_defs(block.terminator.get_defs(), block.id, block.stmts.len());
        add_defs(
            block.terminator.get_write_uses(),
            block.id,
            block.stmts.len(),
        );
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
    to_init: &HashSet<String>,
) -> HashMap<String, HashSet<Loc>> {
    let mut res: HashMap<_, HashSet<_>> = HashMap::new();
    for block_id in &cfg.topo_order {
        // we consider a block that postdominates a use and is "in-scope" to be a use.
        for (var, uses) in &mut res {
            let block_start = Loc(*block_id, 0);
            let mut to_add = false;
            for u in uses.iter() {
                if block_start.pdom(dinfo, u)
                    && defs.get(var).map_or_else(
                        || panic!("Missing def of {var}"),
                        |loc| loc.dom(dinfo, &block_start),
                    )
                {
                    to_add = true;
                    break;
                }
            }
            if to_add {
                uses.insert(block_start);
            }
        }
        let block = cfg.blocks.get_mut(block_id).unwrap();
        for (local_id, instr) in block
            .stmts
            .iter_mut()
            .map(HirInstr::Stmt)
            .chain(std::iter::once(HirInstr::Tail(&mut block.terminator)))
            .enumerate()
        {
            // ignore the uses of a hole, which are everything that reaches it.
            if !matches!(instr, HirInstr::Stmt(HirBody::Hole { .. })) {
                // dbg!(&instr);
                let uses = get_usable_uses(&instr, env, dtypes, |_| {});
                for u in uses {
                    if to_init.contains(&u) {
                        res.entry(u).or_default().insert(Loc(*block_id, local_id));
                    }
                }
            }
        }
    }
    for (v, loc) in defs {
        let mut largest_postdom = Loc(loc.0, usize::MAX);
        for dominated in dinfo.dominated_by(loc.0) {
            let block_end = Loc(*dominated, usize::MAX);
            if block_end.pdom(dinfo, loc) && largest_postdom.lte(cfg, &block_end) {
                largest_postdom = block_end;
            }
        }
        // we consider the last point of the variables scope to be a use
        // TODO: do we want this?
        res.entry(v.clone()).or_default().insert(largest_postdom);
    }
    res
}
