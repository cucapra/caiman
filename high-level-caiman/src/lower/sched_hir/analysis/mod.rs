use std::collections::{hash_map::Entry, BTreeSet, HashMap, HashSet};

mod continuations;
mod dominators;
mod hole_expansion;
mod op_transform;
mod quot;
mod record_expansion;
mod refs;
mod ssa;
mod tags;

use crate::parse::ast::DataType;

use super::{
    cfg::{Cfg, Edge, FINAL_BLOCK_ID, START_BLOCK_ID},
    hir::HirInstr,
    HirBody, Terminator,
};

use caiman::explication::Hole;
pub use continuations::{compute_continuations, Succs};
pub use dominators::compute_dominators;
pub use hole_expansion::set_hole_defs;
pub use op_transform::op_transform_pass;
pub use quot::deduce_tmln_quots;
pub use quot::deduce_val_quots;
pub use record_expansion::transform_encode_pass;
pub use refs::deref_transform_pass;
pub use ssa::transform_out_ssa;
pub use ssa::transform_to_ssa;
#[allow(clippy::module_name_repetitions)]
pub use tags::FlowAnalysis;

#[derive(Default, PartialEq, Eq, Clone)]
#[allow(clippy::struct_field_names)]
pub struct TransferData {
    pub block_id: usize,
    pub cont_id: Option<usize>,
    pub local_instr_id: usize,
}

/// A dataflow analysis fact
pub trait Fact: PartialEq + Clone {
    /// Performs a meet operation on two facts
    #[must_use]
    fn meet(self, other: &Self) -> Self;

    /// Updates the basic block's fact after propagating the fact through the given
    /// statement or terminator.
    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: TransferData);

    type Dir: Direction;
}

/// The direction of an analysis. Provides hooks to change the behavior of the
/// worklist algorithm depending on the analysis direction.
pub trait Direction {
    /// Gets the adj list for the direction
    fn get_adj_list(cfg: &Cfg) -> HashMap<usize, Vec<usize>>;

    /// Iterates over the instructions in the block in the direction
    /// # Arguments
    /// * `it` - The iterator over the instructions
    /// * `func` - The function to call on each instruction
    ///    The function is called on instructions in the order of the direction
    ///    of the analysis
    fn local_iter<'a>(
        it: &mut dyn std::iter::DoubleEndedIterator<Item = (usize, HirInstr<'a>)>,
        func: &mut dyn FnMut(HirInstr<'a>, usize),
    );

    /// Gets the starting point for the analysis in this direction
    fn root_id() -> usize;

    /// Gets the input facts for the analysis, with respect to the direction
    /// # Arguments
    /// * `in_facts` - The input facts for each block, relative to the direction
    /// * `out_facts` - The output facts for each block, relative to the direction
    /// # Returns
    /// * The input facts for the analysis as if the analysis was a forward analysis
    fn get_in_facts<'a, T: Fact>(
        in_facts: &'a HashMap<usize, T>,
        out_facts: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T>;

    /// Gets the output facts for the analysis, with respect to the direction
    /// # Arguments
    /// * `in_facts` - The input facts for each block, relative to the direction
    /// * `out_facts` - The output facts for each block, relative to the direction
    /// # Returns
    /// * The output facts for the analysis as if the analysis was a forward analysis
    fn get_out_facts<'a, T: Fact>(
        in_facts: &'a HashMap<usize, T>,
        out_facts: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T>;
}

/// Analyzes a basic block
/// # Arguments
/// * `cfg` - The CFG
/// * `block` - The block to analyze
/// * `res_in_facts` - The input facts for each instruction
/// * `in_fact` - The input fact for the block
/// # Returns
/// * Tuple of input facts for each instruction and the output fact for the block
fn analyze_basic_block<T: Fact>(cfg: &mut Cfg, block_id: usize, in_fact: &T) -> T {
    let mut fact = in_fact.clone();
    let block = cfg.blocks.get_mut(&block_id).unwrap();
    let cont_id = block.ret_block;
    let sz = block.stmts.len();
    T::Dir::local_iter(
        &mut block
            .stmts
            .iter_mut()
            .enumerate()
            .map(|(id, x)| (id, HirInstr::Stmt(x)))
            .chain(std::iter::once((sz, HirInstr::Tail(&mut block.terminator)))),
        &mut |instr, local_instr_id| {
            fact.transfer_instr(
                instr,
                TransferData {
                    block_id,
                    cont_id,
                    local_instr_id,
                },
            );
        },
    );
    fact
}

/// The result of an analysis pass
pub struct InOutFacts<T: Fact> {
    /// Mapping from blocks to facts coming from the predecessors
    in_facts: HashMap<usize, T>,
    /// Mapping from blocks to facts going to the successors
    out_facts: HashMap<usize, T>,
}

impl<T: Fact> InOutFacts<T> {
    /// Gets the input fact for the given block
    /// # Panics
    /// If the block does not exist
    pub fn get_in_fact(&self, block: usize) -> &T {
        T::Dir::get_in_facts(&self.in_facts, &self.out_facts)
            .get(&block)
            .unwrap()
    }

    /// Gets the output fact for the given block
    /// # Panics
    /// If the block does not exist
    pub fn get_out_fact(&self, block: usize) -> &T {
        T::Dir::get_out_facts(&self.in_facts, &self.out_facts)
            .get(&block)
            .unwrap()
    }
}

/// Reverse topological order
fn topo_order_rev<T>(adj_lst: &HashMap<usize, T>, start_id: usize) -> Vec<usize>
where
    for<'a> &'a T: IntoIterator<Item = &'a usize>,
{
    enum Node {
        Start(usize),
        Finished(usize),
    }
    use Node::{Finished, Start};

    let mut reversed_order = vec![];
    let mut is_done = HashSet::new();
    let mut stack = vec![];
    stack.push(Start(start_id));
    while let Some(n) = stack.pop() {
        match n {
            Start(n) => {
                if is_done.contains(&n) {
                    continue;
                }
                stack.push(Finished(n));
                for next in &adj_lst[&n] {
                    stack.push(Start(*next));
                }
            }
            Finished(n) => {
                reversed_order.push(n);
                is_done.insert(n);
            }
        }
    }
    reversed_order
}

/// Performs a data flow analysis. Requires that `cfg` has a topological order
/// and does not contain loops.
///
/// Furthermore, we assume that any `x` met with Top results in `x`.
pub fn analyze<T: Fact>(cfg: &mut Cfg, top: T) -> InOutFacts<T> {
    let mut in_facts: HashMap<usize, T> = HashMap::new();
    let mut out_facts: HashMap<usize, T> = HashMap::new();
    let adj_lst = T::Dir::get_adj_list(cfg);
    in_facts.insert(T::Dir::root_id(), top);
    let mut worklist = topo_order_rev(&adj_lst, T::Dir::root_id());

    while let Some(block) = worklist.pop() {
        let in_fact = in_facts.get(&block).unwrap();
        let out_fact = analyze_basic_block(cfg, block, in_fact);
        let add_neighbors = out_facts.get(&block) != Some(&out_fact);
        if add_neighbors {
            in_facts = broadcast_out_facts(&[&out_fact], in_facts, &adj_lst, block);
            // use topo order, so don't add to worklist
        }
        out_facts.insert(block, out_fact);
    }
    InOutFacts {
        in_facts,
        out_facts,
    }
}

/// Broadcasts the output facts to the neighbors
/// If there is one output fact, it is broadcasted to all neighbors
/// Otherwise, the number of output facts must be equal to the number of neighbors
/// # Arguments
/// * `out_fact` - The output fact of the current block
/// * `in_facts` - The input facts for each block
/// * `adj_lst` - The adjacency list of the CFG
/// * `block` - The current block
/// # Returns
/// * The input facts for each block
fn broadcast_out_facts<T: Fact>(
    out_fact: &[&T],
    mut in_facts: HashMap<usize, T>,
    adj_lst: &HashMap<usize, Vec<usize>>,
    block: usize,
) -> HashMap<usize, T> {
    if out_fact.is_empty() {
        // do nothing (meet w/ top)
    } else {
        // This must be changed if we support conditional dataflow
        // analyses
        assert_eq!(out_fact.len(), 1);
        for neighbor in adj_lst.get(&block).unwrap() {
            in_facts.insert(
                *neighbor,
                in_facts
                    .get(neighbor)
                    .cloned()
                    .map_or_else(|| out_fact[0].clone(), |x| x.meet(out_fact[0])),
            );
        }
    }
    in_facts
}

/// A backwards analysis
pub struct Backwards {}
impl Direction for Backwards {
    fn get_adj_list(cfg: &Cfg) -> HashMap<usize, Vec<usize>> {
        // gets the reverse adjacency list
        let mut res = HashMap::new();
        for (k, v) in &cfg.graph {
            res.entry(*k).or_insert_with(Vec::default);
            match v {
                Edge::Next(n) => {
                    res.entry(*n).or_insert_with(Vec::new).push(*k);
                }
                Edge::Select {
                    true_branch,
                    false_branch,
                } => {
                    res.entry(*true_branch).or_insert_with(Vec::new).push(*k);
                    res.entry(*false_branch).or_insert_with(Vec::new).push(*k);
                }
                Edge::None => {}
            };
        }
        res
    }

    fn local_iter<'a>(
        it: &mut dyn std::iter::DoubleEndedIterator<Item = (usize, HirInstr<'a>)>,
        func: &mut dyn FnMut(HirInstr<'a>, usize),
    ) {
        for (id, instr) in it.rev() {
            func(instr, id);
        }
    }

    fn root_id() -> usize {
        FINAL_BLOCK_ID
    }

    fn get_in_facts<'a, T: Fact>(
        _: &'a HashMap<usize, T>,
        out_facts: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T> {
        out_facts
    }

    fn get_out_facts<'a, T: Fact>(
        in_facts: &'a HashMap<usize, T>,
        _: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T> {
        in_facts
    }
}

/// A forwards analysis
pub struct Forwards {}
impl Direction for Forwards {
    fn get_adj_list(cfg: &Cfg) -> HashMap<usize, Vec<usize>> {
        // gets the reverse adjacency list
        let mut res = HashMap::new();
        for (k, v) in &cfg.graph {
            match v {
                Edge::Next(n) => {
                    res.entry(*k).or_insert_with(Vec::new).push(*n);
                    // ensure next node has an entry
                    res.entry(*n).or_insert_with(Vec::default);
                }
                Edge::Select {
                    true_branch,
                    false_branch,
                } => {
                    res.entry(*k).or_insert_with(Vec::new).push(*true_branch);
                    res.entry(*k).or_insert_with(Vec::new).push(*false_branch);
                    // ensure nodes have entries
                    res.entry(*false_branch).or_insert_with(Vec::default);
                    res.entry(*true_branch).or_insert_with(Vec::default);
                }
                Edge::None => {}
            };
        }
        res
    }

    fn local_iter<'a>(
        it: &mut dyn std::iter::DoubleEndedIterator<Item = (usize, HirInstr<'a>)>,
        func: &mut dyn FnMut(HirInstr<'a>, usize),
    ) {
        for (id, instr) in it {
            func(instr, id);
        }
    }

    fn root_id() -> usize {
        START_BLOCK_ID
    }

    fn get_in_facts<'a, T: Fact>(
        in_facts: &'a HashMap<usize, T>,
        _: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T> {
        in_facts
    }

    fn get_out_facts<'a, T: Fact>(
        _: &'a HashMap<usize, T>,
        out_facts: &'a HashMap<usize, T>,
    ) -> &'a HashMap<usize, T> {
        out_facts
    }
}

/// Computes reaching definitions and uses the information to fill the uses
/// for holes
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReachingDefs<'a> {
    available_set: HashSet<String>,
    kill_set: HashSet<String>,
    data_types: &'a HashMap<String, DataType>,
    variables: &'a HashSet<String>,
    /// Map from block id to list of defs that become available at that block
    becomes_available: HashMap<usize, Vec<String>>,
}

impl<'a> ReachingDefs<'a> {
    pub fn top<'b>(
        inputs: impl Iterator<Item = &'b String>,
        dt: &'a HashMap<String, DataType>,
        variables: &'a HashSet<String>,
    ) -> Self {
        Self {
            available_set: inputs.cloned().collect(),
            kill_set: HashSet::new(),
            data_types: dt,
            variables,
            becomes_available: HashMap::new(),
        }
    }

    pub fn reaching_defs(&self) -> impl Iterator<Item = &String> {
        self.available_set.difference(&self.kill_set)
    }

    /// Set the uses of a hole to be all reaching definitions of its current program point
    fn set_hole_uses(&self, stmt: &mut HirInstr<'_>) {
        match stmt {
            HirInstr::Stmt(
                HirBody::ConstDecl { rhs, .. }
                | HirBody::VarDecl { rhs: Some(rhs), .. }
                | HirBody::RefStore { rhs, .. }
                | HirBody::DeviceCopy { src: rhs, .. },
            ) => rhs.fill_uses(|| self.reaching_defs().cloned().collect()),
            HirInstr::Stmt(HirBody::Hole { uses, .. }) => {
                assert!(uses.is_initial());
                uses.process(|()| self.reaching_defs().cloned().collect());
            }
            HirInstr::Stmt(HirBody::EncodeDo { func, .. })
            | HirInstr::Tail(Terminator::Call(_, func)) => {
                if let Some(x) = func.extra_uses.as_mut() {
                    assert!(x.is_initial());
                    x.process(|()| self.reaching_defs().cloned().collect());
                }
            }
            HirInstr::Stmt(HirBody::Op { args, .. }) => {
                for arg in args {
                    arg.fill_uses(|| self.reaching_defs().cloned().collect());
                }
            }
            HirInstr::Tail(Terminator::CaptureCall { .. }) => panic!("Pass out of order"),
            HirInstr::Tail(_)
            | HirInstr::Stmt(
                HirBody::RefLoad { .. }
                | HirBody::InAnnotation(..)
                | HirBody::OutAnnotation(..)
                | HirBody::BeginEncoding { .. }
                | HirBody::Submit { .. }
                | HirBody::Sync { .. }
                | HirBody::Phi { .. }
                | HirBody::VarDecl { rhs: None, .. },
            ) => {}
        }
    }
}

impl<'a> Fact for ReachingDefs<'a> {
    fn meet(mut self, other: &Self) -> Self {
        self.kill_set.extend(other.kill_set.iter().cloned());
        for item in &self.available_set {
            if !other.available_set.contains(item) {
                self.kill_set.insert(item.clone());
            }
        }
        for item in &other.available_set {
            if !self.available_set.contains(item) {
                self.kill_set.insert(item.clone());
            }
        }
        self.available_set
            .extend(other.available_set.iter().cloned());
        self
    }

    fn transfer_instr(&mut self, mut stmt: HirInstr<'_>, data: TransferData) {
        self.set_hole_uses(&mut stmt);
        if let Some(newly_available) = self.becomes_available.get(&data.block_id) {
            self.available_set.extend(newly_available.iter().cloned());
        }
        match stmt {
            HirInstr::Stmt(
                HirBody::ConstDecl { lhs: dest, .. }
                | HirBody::Phi { dest, .. }
                | HirBody::RefLoad { dest, .. }
                | HirBody::VarDecl { lhs: dest, .. }
                | HirBody::DeviceCopy { dest, .. },
            ) => {
                self.available_set.insert(dest.clone());
            }
            HirInstr::Stmt(HirBody::Submit { dest, src, .. }) => {
                self.available_set.insert(dest.clone());
                assert!(!self.variables.contains(src));
                self.kill_set.insert(src.clone());
            }
            HirInstr::Stmt(HirBody::Sync { dests, srcs, .. }) => {
                self.available_set
                    .extend(dests.processed().iter().map(|(x, _)| x.clone()));
                assert!(!srcs.processed().iter().any(|x| self.variables.contains(x)));
                self.kill_set.extend(srcs.processed().iter().cloned());
            }
            HirInstr::Stmt(
                HirBody::EncodeDo { dests, .. }
                | HirBody::Op { dests, .. }
                | HirBody::Hole { dests, .. },
            ) => {
                // ops are pure, so srcs aren't killed
                // srcs of encode-do are references, so they aren't killed
                self.available_set
                    .extend(dests.iter().map(|(x, _)| x.clone()));
            }
            HirInstr::Stmt(HirBody::BeginEncoding {
                encoder,
                device_vars,
                ..
            }) => {
                self.available_set.insert(encoder.0.clone());
                self.available_set
                    .extend(device_vars.iter().map(|(x, _)| x.clone()));
            }
            HirInstr::Stmt(
                HirBody::InAnnotation(..) | HirBody::OutAnnotation(..) | HirBody::RefStore { .. },
            )
            | HirInstr::Tail(
                Terminator::Next(..) | Terminator::None(_) | Terminator::FinalReturn(..),
            ) => {}
            HirInstr::Tail(x @ (Terminator::Call(..) | Terminator::Yield(..))) => {
                for avail in &self.available_set {
                    // we can't capture references, so they are no longer reachable over a call
                    if matches!(self.data_types.get(avail), Some(DataType::Ref(_))) {
                        self.kill_set.insert(avail.clone());
                    }
                }
                if let Terminator::Call(dests, func) = x {
                    for arg in &func.args {
                        if let Hole::Filled(arg) = arg {
                            // arguments that are consumed are also no longer reachable
                            if !self.variables.contains(arg) {
                                self.kill_set.insert(arg.clone());
                            }
                        } else {
                            // If we use `?` in a function argument, we have to kill everything because
                            // we don't know what might be consumed here.
                            self.kill_set.extend(self.available_set.iter().cloned());
                        }
                    }
                    for (dest, _) in dests {
                        self.available_set.insert(dest.clone());
                    }
                }
            }
            HirInstr::Tail(Terminator::Return { dests, rets, .. }) => {
                self.available_set
                    .extend(dests.iter().map(|(x, _)| x.clone()));
                self.kill_set
                    .extend(rets.iter().filter_map(|x| x.as_ref().opt().cloned()));
            }
            HirInstr::Tail(Terminator::Select { dests, .. }) => {
                let cont_id = data.cont_id.expect("Select should have a continuation");
                // dests of a select become available at the continuation
                match self.becomes_available.entry(cont_id) {
                    Entry::Vacant(e) => {
                        e.insert(dests.iter().map(|(x, _)| x.clone()).collect());
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().extend(dests.iter().map(|(x, _)| x.clone()));
                    }
                }
            }
            HirInstr::Tail(Terminator::CaptureCall { .. }) => panic!("Passes out of order"),
        }
    }

    type Dir = Forwards;
}

#[derive(Clone, Eq, Debug)]
pub struct LiveVars {
    pub(super) live_set: BTreeSet<String>,
}

impl LiveVars {
    pub const fn top() -> Self {
        Self {
            live_set: BTreeSet::new(),
        }
    }

    pub const fn live_set(&self) -> &BTreeSet<String> {
        &self.live_set
    }
}

impl PartialEq for LiveVars {
    fn eq(&self, other: &Self) -> bool {
        self.live_set == other.live_set
    }
}

/// The stem of the special variables used for return values. Each return variable
/// will have a number appended to this stem.
pub const RET_VAR: &str = "_out";

impl Fact for LiveVars {
    fn meet(mut self, other: &Self) -> Self {
        for var in &other.live_set {
            self.live_set.insert(var.clone());
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: TransferData) {
        if let Some(defs) = stmt.get_defs() {
            for var in defs {
                self.live_set.remove(&var);
            }
        }
        stmt.get_uses(&mut self.live_set);
    }

    type Dir = Backwards;
}

/// For each `begin-encode` determines the fences that are active at that point
/// and mutates the `begin-encode` to include the active fences.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ActiveFences<'a> {
    active_fences: HashSet<String>,
    data_types: &'a HashMap<String, DataType>,
}

impl<'a> ActiveFences<'a> {
    pub fn top<'b, T: Iterator<Item = &'b String>>(
        fences: T,
        data_types: &'a HashMap<String, DataType>,
    ) -> Self {
        Self {
            active_fences: fences.cloned().collect(),
            data_types,
        }
    }
}

impl<'a> Fact for ActiveFences<'a> {
    fn meet(mut self, other: &Self) -> Self {
        self.active_fences = self
            .active_fences
            .intersection(&other.active_fences)
            .cloned()
            .collect();
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: TransferData) {
        match stmt {
            HirInstr::Stmt(HirBody::BeginEncoding { active_fences, .. }) => {
                *active_fences = self.active_fences.iter().cloned().collect();
            }
            stmt => {
                // fences cannot be stored in references or variables, so we only have to worry about defs
                if let Some(defs) = stmt.get_defs() {
                    for d in defs {
                        if matches!(self.data_types.get(&d), Some(DataType::Fence(_))) {
                            self.active_fences.insert(d);
                        }
                    }
                }
                let mut use_set = BTreeSet::new();
                stmt.get_uses(&mut use_set);
                for u in use_set {
                    if matches!(self.data_types.get(&u), Some(DataType::Fence(_))) {
                        self.active_fences.remove(&u);
                    }
                }
            }
        }
    }

    type Dir = Forwards;
}

pub type UseMap = HashMap<String, HashSet<(usize, usize)>>;

/// Gets a map from variable name to a set of `(block_idx, block_local_instr_idx)`
/// for every use of that variable.
pub fn get_uses(cfg: &mut Cfg) -> UseMap {
    let mut res = UseMap::new();
    for block in cfg.blocks.values_mut() {
        for (loc_id, instr) in block
            .stmts
            .iter_mut()
            .map(HirInstr::Stmt)
            .chain(std::iter::once(HirInstr::Tail(&mut block.terminator)))
            .enumerate()
        {
            let mut uses = BTreeSet::new();
            instr.get_uses(&mut uses);
            for u in uses {
                res.entry(u).or_default().insert((block.id, loc_id));
            }
        }
    }
    res
}
