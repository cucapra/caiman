use std::collections::{BTreeSet, HashMap};

mod continuations;
mod dominators;
mod op_transform;
mod quot_typing;
mod refs;
mod ssa;
mod tags;

use super::{
    cfg::{Cfg, Edge, FINAL_BLOCK_ID, START_BLOCK_ID},
    hir::HirInstr,
};

pub use continuations::compute_continuations;
pub use op_transform::op_transform_pass;
pub use quot_typing::deduce_val_quots;
pub use refs::deref_transform_pass;
pub use ssa::transform_out_ssa;
pub use ssa::transform_to_ssa;
#[allow(clippy::module_name_repetitions)]
pub use tags::{TagAnalysis, TagInfo};

/// A dataflow analysis fact
pub trait Fact: PartialEq + Clone {
    /// Performs a meet operation on two facts
    #[must_use]
    fn meet(self, other: &Self) -> Self;

    /// Updates the basic block's fact after propagating the fact through the given
    /// statement or terminator.
    fn transfer_instr(&mut self, stmt: HirInstr<'_>, block_id: usize);

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
        it: &mut dyn std::iter::DoubleEndedIterator<Item = HirInstr<'a>>,
        func: &mut dyn FnMut(HirInstr<'a>),
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
    T::Dir::local_iter(
        &mut block
            .stmts
            .iter_mut()
            .map(HirInstr::Stmt)
            .chain(std::iter::once(HirInstr::Tail(&mut block.terminator))),
        &mut |instr| {
            fact.transfer_instr(instr, block_id);
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

/// Performs a dataflow analysis using the worklist algorithm
#[must_use]
pub fn analyze<T: Fact>(cfg: &mut Cfg, top: &T) -> InOutFacts<T> {
    let mut in_facts: HashMap<usize, T> = HashMap::new();
    let mut out_facts: HashMap<usize, T> = HashMap::new();
    let mut worklist: Vec<usize> = Vec::new();
    let adj_lst = T::Dir::get_adj_list(cfg);
    in_facts.extend(cfg.graph.keys().map(|k| (*k, top.clone())));
    worklist.push(T::Dir::root_id());

    while let Some(block) = worklist.pop() {
        let in_fact = in_facts.get(&block).unwrap();
        let out_fact = analyze_basic_block(cfg, block, in_fact);
        let add_neighbors = out_facts.get(&block) != Some(&out_fact);
        if add_neighbors {
            in_facts = broadcast_out_facts(&[&out_fact], in_facts, &adj_lst, block);
            worklist.extend(adj_lst.get(&block).unwrap());
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
                in_facts.get(neighbor).cloned().unwrap().meet(out_fact[0]),
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
        it: &mut dyn std::iter::DoubleEndedIterator<Item = HirInstr<'a>>,
        func: &mut dyn FnMut(HirInstr<'a>),
    ) {
        for instr in it.rev() {
            func(instr);
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
        it: &mut dyn std::iter::DoubleEndedIterator<Item = HirInstr<'a>>,
        func: &mut dyn FnMut(HirInstr<'a>),
    ) {
        for instr in it {
            func(instr);
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: usize) {
        if let Some(defs) = stmt.get_defs() {
            for var in defs {
                self.live_set.remove(&var);
            }
        }
        stmt.get_uses(&mut self.live_set);
    }

    type Dir = Backwards;
}
