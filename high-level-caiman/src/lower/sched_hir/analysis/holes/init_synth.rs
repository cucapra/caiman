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
//! initializing set is potentially worse on some path. Since this is a statement
//! with regards to a single variable, another way to think of this is that we
//! find a local maxima without ever making things worse, even if temporarily doing
//! so would result in a better solution overall.

use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap, HashSet},
    rc::Rc,
};

use crate::{
    error::{Info, LocalError},
    lower::sched_hir::{
        analysis::{analyze, Backwards, DomInfo, Fact, Forwards, TransferData},
        cfg::{can_reach_goal, Cfg, CollectiveDom, Loc, FINAL_BLOCK_ID, START_BLOCK_ID},
        Hir, HirBody, HirInstr, Terminator, TripleTag,
    },
    parse::ast::{self, DataType, FullType},
    typing::{MetaVar, NodeEnv},
};

use super::{get_usable_uses, invert_map};
use smallvec::{smallvec, SmallVec};

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

/// A dataflow analysis that minimizes an initializing set by identifying
/// initialization points which are collectively dominated by other initialization points.
#[derive(Debug, Clone)]
struct MinimizeInitSet<'a> {
    redundant_locs: Vec<(String, Loc)>,
    seen_inits: HashSet<String>,
    inverted_orig_inits: &'a HashMap<Loc, HashSet<String>>,
}

impl<'a> PartialEq for MinimizeInitSet<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.seen_inits == other.seen_inits
    }
}

impl<'a> Fact for MinimizeInitSet<'a> {
    fn meet(
        mut self,
        other: &Self,
        _: crate::error::Info,
    ) -> Result<Self, crate::error::LocalError> {
        self.redundant_locs
            .extend(other.redundant_locs.iter().cloned());
        Ok(Self {
            seen_inits: self
                .seen_inits
                .intersection(&other.seen_inits)
                .cloned()
                .collect(),
            inverted_orig_inits: self.inverted_orig_inits,
            redundant_locs: self.redundant_locs,
        })
    }

    fn transfer_instr(
        &mut self,
        _: HirInstr<'_>,
        data: crate::lower::sched_hir::analysis::TransferData,
    ) -> Result<(), crate::error::LocalError> {
        let cur_loc = Loc(data.block_id, data.local_instr_id);
        if let Some(inited_vars) = self.inverted_orig_inits.get(&cur_loc) {
            for v in inited_vars {
                if self.seen_inits.contains(v) {
                    self.redundant_locs.push((v.clone(), cur_loc.clone()));
                } else {
                    self.seen_inits.insert(v.clone());
                }
            }
        }
        Ok(())
    }

    type Dir = Forwards;
}

impl<'a> MinimizeInitSet<'a> {
    fn new(inverted_orig_inits: &'a HashMap<Loc, HashSet<String>>) -> Self {
        Self {
            inverted_orig_inits,
            seen_inits: HashSet::new(),
            redundant_locs: vec![],
        }
    }
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
    let inv_res = invert_map(&res);
    // remove unnecessary members from the init set, which enables the hoist optimization.
    let min_init = analyze(cfg, MinimizeInitSet::new(&inv_res)).unwrap();
    for (var, loc) in &min_init.get_out_fact(FINAL_BLOCK_ID).redundant_locs {
        res.get_mut(var).unwrap().remove(loc);
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
                // The node will be filled in by the second quotient deduction pass
                initialized.insert(var.clone(), None);
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
            // allow annotations to tell us where something must be initialized
            // We don't add this to `get_usable_uses` more generally bc I don't think we
            // need to and it doesn't fit with handling in-annotations
            let mut add_annot_use = |tags: &[(String, TripleTag)], loc: Loc| {
                for (var, tag) in tags {
                    if to_init.contains(var)
                        && !matches!(tag.value.quot, Some(ast::Quotient::None))
                        && matches!(tag.value.flow, Some(ast::Flow::Usable))
                    {
                        res.entry(var.clone()).or_default().insert(loc.clone());
                    }
                }
            };
            if let HirInstr::Stmt(HirBody::OutAnnotation(_, tags)) = instr {
                add_annot_use(tags, Loc(*block_id, local_id));
            }
            if let HirInstr::Stmt(HirBody::InAnnotation(_, tags)) = instr {
                add_annot_use(tags, Loc(*block_id, 0));
                for pred in &cfg.succs.preds[block_id] {
                    add_annot_use(tags, Loc(*pred, usize::MAX));
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

/// A map from location to a map from variable name to nodes that need to be initialized
/// if the given variable were to be initialized at the given location.
///
/// Ie. `f(l, v) =` set of classes (with leading '$') that need to be
/// initialized if `v` were to be initialized at `l`
type UnavailableNodes = HashMap<Loc, HashMap<String, HashSet<String>>>;

/// For each location, gets a set of class names (with the leading `$`) that
/// would need to be generated given the initialization set of `inits`.
fn unavailable_nodes<'a, T: Iterator<Item = &'a String> + Clone>(
    cfg: &Cfg,
    env: &NodeEnv,
    to_init: &T,
    uninit: &HashSet<String>,
) -> UnavailableNodes {
    let mut res = HashMap::new();
    for block in cfg.blocks.values() {
        for (local_id, s) in block.stmts.iter().enumerate() {
            let mut unavailable = HashMap::new();
            if let HirBody::Hole { uses, .. } = s {
                for var in to_init.clone() {
                    if let Some(node) = env.get_node_name(var) {
                        let node_name = MetaVar::new_class_name(&node);
                        unavailable.insert(
                            var.clone(),
                            env.unavailable_nodes(
                                &node_name,
                                // we ignore the variable we're trying to initialize
                                // TODO: we can count them if their initializations dominate this location...
                                uses.processed().iter().filter(|&u| !uninit.contains(u)),
                            ),
                        );
                    }
                }
            }
            res.insert(Loc(block.id, local_id), unavailable);
        }
    }
    res
}

/// Determines which holes anticipate the initialization of a variable. As in,
/// for which holes do all paths to the exit intersect with an initialization of
/// a certain variable.
///
/// This is akin to anticipated expressions in standard partial redundancy
/// elimination.
#[derive(Clone)]
struct AnticipatedInits<'a> {
    anticipated_var_inits: HashSet<String>,
    inits: &'a HashMap<Loc, HashSet<String>>,
    anticipated_vars: Rc<RefCell<HashMap<Loc, HashSet<String>>>>,
}

impl<'a> AnticipatedInits<'a> {
    /// # Args
    /// * `inits` - map from hole location to set of variables initialized there
    /// * `output` - result map which will store map of hole locations to variables whose
    ///     initialization is anticipated.
    pub fn top(
        inits: &'a HashMap<Loc, HashSet<String>>,
        output: Rc<RefCell<HashMap<Loc, HashSet<String>>>>,
    ) -> Self {
        Self {
            anticipated_var_inits: HashSet::new(),
            inits,
            anticipated_vars: output,
        }
    }
}

impl<'a> PartialEq for AnticipatedInits<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.anticipated_var_inits == other.anticipated_var_inits
    }
}

impl<'a> Fact for AnticipatedInits<'a> {
    fn meet(self, other: &Self, _: Info) -> Result<Self, LocalError> {
        Ok(Self {
            anticipated_var_inits: self
                .anticipated_var_inits
                .intersection(&other.anticipated_var_inits)
                .cloned()
                .collect(),
            inits: self.inits,
            anticipated_vars: self.anticipated_vars,
        })
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: TransferData) -> Result<(), LocalError> {
        if matches!(stmt, HirInstr::Stmt(HirBody::Hole { .. })) {
            let cur_loc = Loc(data.block_id, data.local_instr_id);
            self.anticipated_vars
                .borrow_mut()
                .insert(cur_loc.clone(), self.anticipated_var_inits.clone());
            if let Some(initialized) = self.inits.get(&cur_loc) {
                self.anticipated_var_inits
                    .extend(initialized.iter().cloned());
            }
        }
        Ok(())
    }

    type Dir = Backwards;
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Copy)]
struct Rational(u32, u32);
impl Rational {
    const fn gcd(mut a: u32, mut b: u32) -> u32 {
        const fn diff(a: u32, b: u32) -> (u32, u32) {
            if a > b {
                (a - b, b)
            } else {
                (b - a, a)
            }
        }
        loop {
            (a, b) = diff(a, b);
            if a == 0 || b == 0 {
                return b;
            }
        }
    }
    pub const fn new(num: u32, den: u32) -> Self {
        let gcd = Self::gcd(num, den);
        Self(num / gcd, den / gcd)
    }

    pub const fn one() -> Self {
        Self(1, 1)
    }

    pub const fn num(self) -> u32 {
        self.0
    }

    pub const fn den(self) -> u32 {
        self.1
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Rational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 * other.1).cmp(&(other.0 * self.1))
    }
}

#[cfg(test)]
#[test]
fn test_rational() {
    assert_eq!(Rational::gcd(22, 104), 2);
    assert_eq!(Rational::gcd(96, 144), 48);
    assert_eq!(Rational::new(3, 9), Rational::new(1, 3));
    assert_eq!(Rational::new(1, 1), Rational::new(7, 7));
    assert_eq!(Rational::new(22, 121), Rational::new(2, 11));
    assert_ne!(Rational::new(1, 2), Rational::new(2, 2));
    assert!(Rational::new(3, 4) > Rational::new(5, 7));
    assert!(Rational::new(1, 2) < Rational::new(2, 1));
}

#[derive(Default, Clone, Debug)]
struct FractionalMultiSet<T: std::hash::Hash + Eq> {
    map: HashMap<T, Rational>,
}

impl<T: std::hash::Hash + Eq> FractionalMultiSet<T> {
    fn subset(&self, other: &Self) -> bool {
        let mut better = false;
        for (entry, amount) in &self.map {
            if let Some(other_amount) = other.map.get(entry) {
                match amount.cmp(other_amount) {
                    Ordering::Greater => return false,
                    Ordering::Less => better = true,
                    Ordering::Equal => (),
                }
            } else {
                return false;
            }
        }
        // at this point, we know that self \subseteq other
        if better {
            return true;
        }
        for entry in other.map.keys() {
            if !self.map.contains_key(entry) {
                return true;
            }
        }
        // self = other
        false
    }

    fn intersect(self, other: &Self) -> Self {
        let mut new_map = HashMap::new();
        for (entry, amount) in self.map {
            if let Some(other_amount) = other.map.get(&entry) {
                new_map.insert(entry, amount.min(*other_amount));
            }
        }
        Self { map: new_map }
    }

    #[allow(unused)]
    fn insert(&mut self, e: T) {
        if let Some(val) = self.map.get_mut(&e) {
            *val = Rational::new(val.num() + 1, val.den());
        } else {
            self.map.insert(e, Rational::one());
        }
    }

    /// Insert `e` into the multi-set, increasing the denominator if it already is present
    fn insert_frac(&mut self, e: T) {
        if let Some(val) = self.map.get_mut(&e) {
            *val = Rational::new(val.num(), val.den() + 1);
        } else {
            self.map.insert(e, Rational::one());
        }
    }

    fn contains(&self, e: &T) -> bool {
        self.map.contains_key(e)
    }

    #[allow(unused)]
    fn remove_all(&mut self, e: &T) {
        self.map.remove(e);
    }
}

#[derive(Clone)]
struct DependencyCalc<'a> {
    env: &'a NodeEnv,
    /// map from location to mutables we are currently initializing at this location
    inv_inits: &'a HashMap<Loc, HashSet<String>>,
    unavailable_nodes: &'a UnavailableNodes,
    cache: HashMap<Loc, Rc<FractionalMultiSet<String>>>,
}

impl<'a> DependencyCalc<'a> {
    /// Computes the nodes that must be initialized if `var` were to be made usable at `loc`.
    /// Takes into account sharing of dependencies by making the contribution
    /// of a dependency count as `1/n` where `n` is the number of variables
    /// being initialized at `loc` that require an initialization of the dependency.
    fn calc(&mut self, var: &str, loc: &Loc) -> Rc<FractionalMultiSet<String>> {
        if let Some(res) = self.cache.get(loc) {
            return res.clone();
        }
        let var_class =
            MetaVar::new_class_name(&self.env.get_node_name(var).unwrap()).into_string();
        let needed = &self.unavailable_nodes[loc][var];
        let mut res = FractionalMultiSet::default();
        for (v, deps) in &self.unavailable_nodes[loc] {
            // for all variables (including ourselves) being initialized here,
            // take into account dependencies they are initializing at this
            // hole location if we also need them.
            if self.inv_inits.get(loc).is_some_and(|s| s.contains(v)) || var == v {
                for dep in deps {
                    if needed.contains(dep) || dep == &var_class {
                        res.insert_frac(dep.clone());
                    }
                }
            }
        }
        // take ourselves into account
        res.insert_frac(var_class);
        if let Some(being_inited) = self.inv_inits.get(loc) {
            for init_mutable in being_inited {
                // take other variables (not ourself) being initialized at this hole into account
                // if we depend on them.
                if init_mutable != var {
                    if let Some(var_node_name) = self.env.get_node_name(init_mutable) {
                        let init_var_node_class =
                            MetaVar::new_class_name(&var_node_name).into_string();
                        if res.contains(&init_var_node_class) {
                            // take into account things we're already initializing
                            res.insert_frac(init_var_node_class);
                        }
                    }
                }
            }
        }
        let res = Rc::new(res);
        self.cache.insert(loc.clone(), res.clone());
        res
    }
}

/// A backwards pass that hoists variable initializations up in the cfg
/// to locations that would require initializing fewer variables.
#[derive(Clone)]
struct Hoist<'a> {
    cfg: &'a Cfg,
    prev_var_inits: &'a HashSet<Loc>,
    var_inits: HashSet<Loc>,
    /// The variable to handle
    var: &'a String,
    /// Location of definition of mutable variable.
    var_def: Loc,
    /// Map from mark location to a tuple of `(origin location, set of initializations)`
    mark: HashMap<Loc, (Loc, HashSet<Loc>)>,
    anticipated_inits: &'a HashMap<Loc, HashSet<String>>,
    doms: &'a DomInfo,
    dep_calc: DependencyCalc<'a>,
    /// no location that precedes any of these can be hoisted to
    freezes: &'a HashSet<Loc>,
}

impl<'a> Hoist<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        env: &'a NodeEnv,
        cfg: &'a Cfg,
        var: &'a String,
        var_def: Loc,
        anticipated_inits: &'a HashMap<Loc, HashSet<String>>,
        doms: &'a DomInfo,
        inits: &'a HashMap<String, HashSet<Loc>>,
        inv_inits: &'a HashMap<Loc, HashSet<String>>,
        unavailable_nodes: &'a UnavailableNodes,
        freezes: &'a HashSet<Loc>,
    ) -> Self {
        Self {
            cfg,
            var,
            prev_var_inits: &inits[var],
            mark: HashMap::new(),
            anticipated_inits,
            doms,
            var_inits: inits[var].clone(),
            var_def,
            freezes,
            dep_calc: DependencyCalc {
                env,
                inv_inits,
                unavailable_nodes,
                cache: HashMap::new(),
            },
        }
    }
}

impl<'a> Hoist<'a> {
    /// Marks the current location as an initialization point if the multi-set of
    /// nodes we have to calculate at the current location is a subset of the nodes
    /// of `self.mark[&loc]`
    ///
    /// Requires that `loc` is a hole and `self.mark[&loc]` is set.
    fn mark_cur_loc_if_better(&mut self, loc: Loc) {
        if matches!(self.anticipated_inits.get(&loc), Some(set) if set.contains(self.var))
            && self.var_def.dom(self.doms, &loc)
        {
            for freeze_loc in self.freezes {
                if loc.lte(self.cfg, freeze_loc) {
                    // user says variable is dead here
                    return;
                }
            }
            let calc_here = self.dep_calc.calc(self.var, &loc);
            if let Some(prev_calcs) = self.mark[&loc]
                .1
                .iter()
                .map(|loc| self.dep_calc.calc(self.var, loc))
                .next()
            {
                let mut prev_calcs = (*prev_calcs).clone();
                for marked_loc in self.mark[&loc]
                    .1
                    .iter()
                    .skip(1)
                    .map(|loc| self.dep_calc.calc(self.var, loc))
                {
                    prev_calcs = prev_calcs.intersect(&marked_loc);
                }
                if calc_here.subset(&prev_calcs) {
                    self.mark.insert(loc.clone(), {
                        let mut hs = HashSet::new();
                        hs.insert(loc.clone());
                        (loc.clone(), hs)
                    });
                    let mut to_remove: SmallVec<[_; 8]> = smallvec![];
                    for init_loc in &self.var_inits {
                        if loc.dom(self.doms, init_loc) {
                            to_remove.push(init_loc.clone());
                        }
                    }
                    let should_insert = !to_remove.is_empty();
                    while let Some(to_rem) = to_remove.pop() {
                        self.var_inits.remove(&to_rem);
                    }
                    if should_insert {
                        self.var_inits.insert(loc);
                    }
                }
            }
        }
    }
}

impl<'a> PassState for Hoist<'a> {
    fn handle_terminator(&mut self, _: &Terminator, loc: Loc) {
        let next = loc.succs(self.cfg);
        match next.len() {
            0 => {
                self.mark.insert(loc.clone(), (loc, HashSet::new()));
            }
            1 => {
                self.mark.insert(loc, self.mark[&next[0]].clone());
            }
            2 => {
                let (loc_a, mark_a) = &self.mark[&next[0]];
                let (loc_b, mark_b) = &self.mark[&next[1]];
                if loc_a.pdom(self.doms, loc_b) {
                    self.mark
                        .insert(loc.clone(), (loc_a.clone(), mark_a.clone()));
                } else if loc_b.pdom(self.doms, loc_a) {
                    self.mark
                        .insert(loc.clone(), (loc_b.clone(), mark_b.clone()));
                } else {
                    self.mark.insert(
                        loc.clone(),
                        (loc.clone(), mark_a.union(mark_b).cloned().collect()),
                    );
                    let mark_cur = &self.mark[&loc].1;
                    let mut to_remove: SmallVec<[_; 4]> = smallvec![];
                    for loc in &self.var_inits {
                        if !mark_cur.contains(loc)
                            && mark_cur.cdom(self.cfg, &{
                                let mut s = HashSet::new();
                                s.insert(loc.clone());
                                s
                            })
                        {
                            to_remove.push(loc.clone());
                        }
                    }
                    let should_insert = !to_remove.is_empty();
                    while let Some(loc) = to_remove.pop() {
                        self.var_inits.remove(&loc);
                    }
                    if should_insert {
                        self.var_inits.extend(mark_cur.iter().cloned());
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_body(&mut self, t: &HirBody, loc: Loc) {
        let is_hole = matches!(t, HirBody::Hole { .. });
        if self.prev_var_inits.contains(&loc) {
            assert!(is_hole);
            self.mark.insert(
                loc.clone(),
                (loc.clone(), {
                    let mut set = HashSet::new();
                    set.insert(loc);
                    set
                }),
            );
        } else {
            let next = loc.succs(self.cfg);
            assert_eq!(next.len(), 1);
            self.mark.insert(loc.clone(), self.mark[&next[0]].clone());
            if is_hole {
                self.mark_cur_loc_if_better(loc);
            }
        }
    }
}
impl<'a> BackwardPass for Hoist<'a> {}

/// A pass that iterates over the entire CFG, producing a single final result instead
/// of input and output results for each block.
trait PassState {
    fn handle_terminator(&mut self, t: &Terminator, loc: Loc);
    fn handle_body(&mut self, t: &HirBody, loc: Loc);
}

/// A pass that is meant to be run backwards.
trait BackwardPass: PassState {
    fn backward_pass(&mut self, cfg: &Cfg) -> &mut Self {
        let mut order = cfg.topo_order.clone();
        while let Some(block) = order.pop() {
            let block = &cfg.blocks[&block];
            self.handle_terminator(&block.terminator, Loc(block.id, block.stmts.len()));
            for (id, stmt) in block.stmts.iter().enumerate().rev() {
                self.handle_body(stmt, Loc(block.id, id));
            }
        }
        self
    }
}

/// A pass that is meant to be run forwards.
trait ForwardPass: PassState {
    fn forward_pass(&mut self, cfg: &Cfg) -> &mut Self {
        let mut order = cfg.topo_order_rev.clone();
        while let Some(block) = order.pop() {
            let block = &cfg.blocks[&block];
            for (id, stmt) in block.stmts.iter().enumerate() {
                self.handle_body(stmt, Loc(block.id, id));
            }
            self.handle_terminator(&block.terminator, Loc(block.id, block.stmts.len()));
        }
        self
    }
}

/// A pass that finds the definition locations of all variables we need to initialize.
struct Defs<'a> {
    to_init: &'a HashSet<String>,
    def_locs: HashMap<String, Loc>,
}

impl<'a> Defs<'a> {
    fn handle(&mut self, t: &dyn Hir, loc: &Loc) {
        if let Some(defs) = t.get_defs() {
            for def in defs {
                if self.to_init.contains(&def) {
                    match self.def_locs.entry(def) {
                        Entry::Occupied(..) => unreachable!(),
                        Entry::Vacant(e) => {
                            e.insert(loc.clone());
                        }
                    }
                }
            }
        }
    }

    fn new(to_init: &'a HashSet<String>) -> Self {
        Self {
            to_init,
            def_locs: HashMap::new(),
        }
    }
}

impl<'a> PassState for Defs<'a> {
    fn handle_terminator(&mut self, t: &Terminator, loc: Loc) {
        self.handle(t, &loc);
    }

    fn handle_body(&mut self, t: &HirBody, loc: Loc) {
        self.handle(t, &loc);
    }
}

impl<'a> ForwardPass for Defs<'a> {}

/// Gets a map from variable to definition location
/// for variables that are defined by a hole and have a node in the environment.
struct HoleDefs<'a> {
    env: &'a NodeEnv,
    defs: HashMap<String, Loc>,
}

impl<'a> HoleDefs<'a> {
    fn new(env: &'a NodeEnv) -> Self {
        Self {
            env,
            defs: HashMap::new(),
        }
    }
}

impl<'a> PassState for HoleDefs<'a> {
    fn handle_terminator(&mut self, _: &Terminator, _: Loc) {}

    fn handle_body(&mut self, t: &HirBody, loc: Loc) {
        if let HirBody::Hole { dests, .. } = t {
            for dest in dests.iter().map(|(dest, _)| dest) {
                if self.env.get_node_name(dest).is_some() {
                    assert!(!self.defs.contains_key(dest));
                    self.defs.insert(dest.clone(), loc.clone());
                }
            }
        }
    }
}

impl<'a> ForwardPass for HoleDefs<'a> {}

/// Determines all the definitions by holes and the locations those definitions occur
/// at, returning a mapping from hole-defined var to def location.
///
/// Also appends the initialization locations of these variables to `init_sets`
fn append_static_hole_defs(
    init_sets: &mut HashMap<String, HashSet<Loc>>,
    cfg: &Cfg,
    env: &NodeEnv,
) -> HashMap<String, Loc> {
    // take into account definitions of a hole.
    let mut hole_defs = HoleDefs::new(env);
    hole_defs.forward_pass(cfg);
    let hole_defs = hole_defs.defs;
    for (var, loc) in &hole_defs {
        assert!(
            !init_sets.contains_key(var)
                || init_sets.get(var).map_or(false, |set| set.is_empty()
                    || (set.len() == 1 && set.contains(loc)))
        );
        init_sets.insert(var.clone(), {
            let mut hs = HashSet::new();
            hs.insert(loc.clone());
            hs
        });
    }
    hole_defs
}

/// Optimizes the placement of holes in `init_sets` to reduce the amount of nodes
/// we will need to recalculate.
///
/// # Returns
/// The updated initializing set
pub fn hoist_optimization(
    mut init_sets: HashMap<String, HashSet<Loc>>,
    to_init: &HashSet<String>,
    cfg: &mut Cfg,
    env: &NodeEnv,
    doms: &DomInfo,
) -> HashMap<String, HashSet<Loc>> {
    if init_sets.is_empty() {
        return init_sets;
    }
    let mut changed = false;
    let anticipated_inits = Rc::new(RefCell::new(HashMap::new()));
    let _ = analyze(
        cfg,
        AnticipatedInits::top(&invert_map(&init_sets), anticipated_inits.clone()),
    );
    let anticipated_inits = anticipated_inits.take();
    let hole_defs = append_static_hole_defs(&mut init_sets, cfg, env);
    let unavailable_nodes =
        unavailable_nodes(cfg, env, &to_init.iter().chain(hole_defs.keys()), to_init);
    let mut calc_defs = Defs::new(to_init);
    calc_defs.forward_pass(cfg);
    let defs = calc_defs.def_locs;
    let mut freezes = HoistFreezes::default();
    freezes.forward_pass(cfg);
    let freezes = freezes.freezes;
    let empty = HashSet::new();

    // bound the number of iterations to converge arbitrarily
    for _ in 0..5 {
        for v in to_init {
            if init_sets[v].is_empty() || !defs.contains_key(v) {
                continue;
            }
            let inv_inits = invert_map(&init_sets);
            let the_def = &defs[v];
            let mut hoist_v = Hoist::new(
                env,
                cfg,
                v,
                the_def.clone(),
                &anticipated_inits,
                doms,
                &init_sets,
                &inv_inits,
                &unavailable_nodes,
                freezes.get(v).unwrap_or(&empty),
            );
            hoist_v.backward_pass(cfg);
            if hoist_v.var_inits != init_sets[v] {
                changed = true;
                assert!(hoist_v.var_inits.cdom(cfg, &init_sets[v]));
                init_sets.insert(v.clone(), hoist_v.var_inits);
            }
        }
        if !changed {
            break;
        }
    }
    init_sets
}

/// Calculates the locations where we cannot hoist to any predecessor. This is
/// caused by a user manually adding a `dead` type annotation.
#[derive(Default)]
struct HoistFreezes {
    freezes: HashMap<String, HashSet<Loc>>,
}

impl PassState for HoistFreezes {
    fn handle_terminator(&mut self, _: &Terminator, _: Loc) {}

    fn handle_body(&mut self, t: &HirBody, loc: Loc) {
        let mut add_to_freezes = |tags: &[(String, TripleTag)], loc: Loc| {
            for (var, tag) in tags {
                if (tag.value.quot.is_none() || tag.value.quot == Some(ast::Quotient::None))
                    && tag.value.flow == Some(ast::Flow::Dead)
                {
                    self.freezes
                        .entry(var.clone())
                        .or_default()
                        .insert(loc.clone());
                }
            }
        };
        match t {
            HirBody::InAnnotation(_, tags) => add_to_freezes(tags, Loc(loc.0, 0)),
            HirBody::OutAnnotation(_, tags) => add_to_freezes(tags, Loc(loc.0, usize::MAX)),
            _ => (),
        }
    }
}

impl ForwardPass for HoistFreezes {}
