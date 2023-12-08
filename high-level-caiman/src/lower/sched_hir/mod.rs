pub mod cfg;
#[allow(clippy::module_inception)]
mod hir;

use std::collections::{BTreeSet, HashMap};

pub use hir::*;

use crate::{
    lower::data_type_to_local_type,
    parse::ast::{Arg, FullType, SchedulingFunc},
};
use caiman::assembly::ast as asm;

use self::{
    analysis::{analyze, deref_transform_pass, InOutFacts, LiveVars, TagAnalysis},
    cfg::{BasicBlock, Cfg, Edge, FINAL_BLOCK_ID},
};

use super::{global_context::SpecType, lower_schedule::hlc_arg_to_asm_arg};
pub use analysis::TagInfo;
mod analysis;
#[cfg(test)]
mod test;

pub use analysis::RET_VAR;

/// Scheduling funclet specs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Specs {
    pub value: asm::FuncletId,
    pub timeline: asm::FuncletId,
    pub spatial: asm::FuncletId,
}

impl Specs {
    /// Gets the type of spec the funclet with `spec_name` is or `None`
    /// if it is not a spec funclet.
    pub fn get_spec_type(&self, spec_name: &str) -> Option<SpecType> {
        if spec_name == self.value.0 {
            Some(SpecType::Value)
        } else if spec_name == self.spatial.0 {
            Some(SpecType::Spatial)
        } else if spec_name == self.timeline.0 {
            Some(SpecType::Timeline)
        } else {
            None
        }
    }
}

/// Information about a high level caiman function
struct FuncInfo {
    name: String,
    input: Vec<Arg<FullType>>,
    output: Arg<FullType>,
}

/// The funclets of a scheduling function.
/// Combines the scheduling function's CFG with its analysis information
pub struct Funclets {
    cfg: Cfg,
    live_vars: analysis::InOutFacts<LiveVars>,
    type_info: analysis::InOutFacts<TagAnalysis>,
    types: HashMap<String, asm::TypeId>,
    finfo: FuncInfo,
    specs: Specs,
}

/// A specific funclet in a scheduling function.
/// Combines the funclet's basic block with is analysis information.
pub struct Funclet<'a> {
    parent: &'a Funclets,
    block: &'a BasicBlock,
}

impl<'a> Funclet<'a> {
    /// Gets the next blocks in the cfg as `FuncletIds`
    pub fn next_blocks(&self) -> Vec<asm::Hole<asm::FuncletId>> {
        match &self.block.terminator {
            Terminator::FinalReturn => vec![],
            Terminator::Select(..) => {
                let mut e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| asm::FuncletId(self.parent.funclet_name(id)));
                let mut res = vec![];
                if let Some(true_block) = e.next() {
                    res.push(Some(true_block));
                }
                if let Some(false_block) = e.next() {
                    res.push(Some(false_block));
                }
                assert_eq!(res.len(), 2);
                res
            }
            Terminator::Call(..) => todo!(),
            Terminator::None | Terminator::Return(..) | Terminator::Next(_) => {
                let e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| Some(asm::FuncletId(self.parent.funclet_name(id))));
                let res: Vec<_> = e.collect();
                assert!(res.len() <= 1);
                res
            }
        }
    }

    /// Gets the input arguments of this funclet based on the union of the live
    /// variables of all predecessor funclets.
    fn input_vars(&self) -> BTreeSet<&String> {
        // TODO: re-evaluate if this is correct for the general case
        self.parent
            .cfg
            .predecessors(self.id())
            .iter()
            .flat_map(|id| self.parent.live_vars.get_out_fact(*id).live_set().iter())
            .collect()
    }

    /// Gets the output arguments of this funclet based on the live out variables
    /// of this block. The returned vector of strings do not contain duplicates.
    fn output_vars(&self) -> Vec<&String> {
        self.parent
            .live_vars
            .get_out_fact(self.block.cont_block.unwrap_or_else(|| self.id()))
            .live_set()
            .iter()
            .collect()
    }

    /// Gets the input arguments for each block based on the block's live in variables
    pub fn inputs(&self) -> Vec<asm::FuncletArgument> {
        #[allow(clippy::map_unwrap_or)]
        if self.id() == cfg::START_BLOCK_ID {
            self.parent
                .finfo
                .input
                .iter()
                .map(hlc_arg_to_asm_arg)
                .collect()
        } else {
            self.input_vars()
                .iter()
                .map(|&var| asm::FuncletArgument {
                    name: Some(asm::NodeId(var.clone())),
                    typ: self
                        .parent
                        .types
                        .get(var)
                        .unwrap_or_else(|| panic!("Missing type for {var}"))
                        .clone(),
                    tags: self
                        .get_input_tag(var)
                        .unwrap_or_else(|| panic!("A tag must be specified for {var}"))
                        .tags_vec_default(),
                })
                .collect()
        }
    }

    /// Gets the input tag of the specified variable, handling input overrides
    fn get_input_tag(&self, var: &str) -> Option<TagInfo> {
        let ovr = self
            .parent
            .type_info
            .get_out_fact(self.id())
            .get_input_override(var);
        match (
            self.parent.type_info.get_in_fact(self.id()).get_tag(var),
            ovr,
        ) {
            (orig, None) => orig,
            (None, Some(ovr)) => Some(TagInfo::from_tags(&ovr, self.specs())),
            (Some(mut orig), Some(ovr)) => {
                orig.update(self.specs(), &ovr);
                Some(orig)
            }
        }
    }

    /// Gets the return arguments of a funclet based on the block's live out variables
    pub fn outputs(&self) -> Vec<asm::FuncletArgument> {
        if self.id() == cfg::START_BLOCK_ID || self.id() == cfg::FINAL_BLOCK_ID {
            vec![hlc_arg_to_asm_arg(&self.parent.finfo.output)]
        } else {
            // TODO: re-evaluate if this is correct for the general case
            self.output_vars()
                .iter()
                .map(|&var| asm::FuncletArgument {
                    name: None,
                    typ: self
                        .parent
                        .types
                        .get(var)
                        .unwrap_or_else(|| panic!("Missing base type for {var}"))
                        .clone(),
                    tags: self
                        .parent
                        .type_info
                        .get_out_fact(self.block.cont_block.unwrap_or_else(|| self.id()))
                        .get_tag(var)
                        // no need for input overrides since this is the output
                        .unwrap_or_else(|| panic!("A tag must be specified for {var}"))
                        .tags_vec_default(),
                })
                .collect()
        }
    }

    /// Gets the nodes that exit this funclet
    pub fn output_args(&self) -> Vec<asm::Hole<asm::NodeId>> {
        self.parent
            .live_vars
            .get_out_fact(self.block.id)
            .live_set()
            .iter()
            .cloned()
            .map(asm::NodeId)
            .map(Some)
            .collect()
    }

    #[inline]
    pub fn stmts(&self) -> &[hir::Hir] {
        &self.block.stmts
    }

    #[inline]
    pub const fn terminator(&self) -> &hir::Terminator {
        &self.block.terminator
    }

    /// Numeric id of the funclet, which is how it's identified at the HIR level
    #[inline]
    pub const fn id(&self) -> usize {
        self.block.id
    }

    /// Gets the name of the funclet, which is how it's identified at the
    /// assembly level.
    #[inline]
    pub fn name(&self) -> String {
        self.parent.funclet_name(self.id())
    }

    /// Gets the specs of this funclet
    #[inline]
    pub const fn specs(&self) -> &Specs {
        &self.parent.specs
    }

    /// Gets the funclet name of the join point. The join point is the
    /// first successor funclet shared by all immediate successors of this funclet.
    #[inline]
    pub fn join_funclet(&self) -> asm::FuncletId {
        // TODO: re-evaluate if this is correct for the general case
        let id = match self.parent.cfg.graph.get(&self.id()).unwrap() {
            Edge::Select {
                true_branch,
                false_branch,
            } => {
                let s = self
                    .parent
                    .cfg
                    .blocks
                    .get(true_branch)
                    .unwrap()
                    .join_block
                    .unwrap_or(FINAL_BLOCK_ID);
                assert_eq!(
                    s,
                    self.parent
                        .cfg
                        .blocks
                        .get(false_branch)
                        .unwrap()
                        .join_block
                        .unwrap_or(FINAL_BLOCK_ID)
                );
                s
            }
            Edge::Next(id) => self
                .parent
                .cfg
                .blocks
                .get(id)
                .unwrap()
                .join_block
                .unwrap_or(FINAL_BLOCK_ID),
            Edge::None => FINAL_BLOCK_ID,
        };
        asm::FuncletId(self.parent.funclet_name(id))
    }

    /// Gets the local type of the specified variable.
    #[inline]
    #[allow(dead_code)]
    pub fn get_local_type(&self, var: &str) -> Option<asm::TypeId> {
        self.parent.types.get(var).cloned()
    }

    /// Gets the tag of the specified variable at the end of the funclet
    #[inline]
    pub fn get_out_tag(&self, var: &str) -> Option<TagInfo> {
        self.parent.type_info.get_out_fact(self.id()).get_tag(var)
    }
}

impl Funclets {
    /// Replaces `Terminator::None` with `Terminator::Next` which is required for
    /// lowering. `Terminator::Next` contains information about which variables
    /// escape the basic block while `Terminator::None` does not. We use
    /// `Terminator::None` as a temporary until CFG analyses can be performed.
    fn add_terminators(cfg: &mut Cfg, live_vars: &InOutFacts<LiveVars>) {
        for (id, bb) in &mut cfg.blocks {
            if matches!(bb.terminator, Terminator::None)
                && cfg
                    .graph
                    .get(id)
                    .map_or(false, |e| !matches!(e, Edge::None))
            {
                bb.terminator = Terminator::Next(
                    live_vars
                        .get_out_fact(*id)
                        .live_set
                        .iter()
                        .cloned()
                        .collect(),
                );
            }
        }
    }
    pub fn new(f: SchedulingFunc, specs: Specs) -> Self {
        let mut cfg = Cfg::new(f.statements);
        let mut types = Self::collect_types(&cfg, &f.input, &f.output);
        deref_transform_pass(&mut cfg, &mut types);
        let live_vars = analyze(&mut cfg, &LiveVars::top());
        let type_info = analyze(&mut cfg, &TagAnalysis::top(&specs, &f.output));
        Self::add_terminators(&mut cfg, &live_vars);
        let finfo = FuncInfo {
            name: f.name,
            input: f.input,
            output: (
                String::new(),
                f.output.expect("Functions must return values for now"),
            ),
        };
        Self {
            cfg,
            live_vars,
            type_info,
            types,
            finfo,
            specs,
        }
    }

    /// Collects a map of variable names to their base types.
    /// # Arguments
    /// * `cfg` - The canonical CFG of the scheduling function
    /// * `f_in` - The input arguments of the scheduling function
    /// * `f_out` - The output argument of the scheduling function
    fn collect_types(
        cfg: &Cfg,
        f_in: &[(String, FullType)],
        f_out: &Option<FullType>,
    ) -> HashMap<String, asm::TypeId> {
        use std::collections::hash_map::Entry;
        let mut types = HashMap::new();
        types.insert(
            String::from(RET_VAR),
            data_type_to_local_type(&f_out.as_ref().unwrap().base.base),
        );
        for (var, typ) in f_in {
            types.insert(var.to_string(), data_type_to_local_type(&typ.base.base));
        }
        for bb in cfg.blocks.values() {
            for stmt in &bb.stmts {
                if let (Some(def), Some(typ)) = (stmt.get_def(), stmt.get_def_local_type()) {
                    match types.entry(def) {
                        Entry::Occupied(t) => assert_eq!(t.get(), &typ),
                        Entry::Vacant(v) => {
                            v.insert(typ);
                        }
                    }
                }
            }
        }
        types
    }

    /// Get's the list of funclets in this scheduling function
    pub fn funclets(&self) -> Vec<Funclet<'_>> {
        let mut v: Vec<_> = self
            .cfg
            .blocks
            .values()
            .map(|blk| Funclet {
                parent: self,
                block: blk,
            })
            .collect();
        // sort for determinism in assigning of funclet ids at the IR level
        v.sort_by_key(Funclet::id);
        v
    }

    /// Gets the name of the scheduling funclet for a given block
    fn funclet_name(&self, block_id: usize) -> String {
        if block_id == cfg::START_BLOCK_ID {
            self.finfo.name.clone()
        } else {
            format!("_{}{block_id}", self.finfo.name)
        }
    }
}

/// Returns true if the type is a reference
pub fn is_ref(typ: &asm::TypeId) -> bool {
    matches!(typ, asm::TypeId::Local(s) if s.starts_with('&'))
        || matches!(
            typ,
            asm::TypeId::FFI(asm::FFIType::ConstRef(_) | asm::FFIType::MutRef(_))
        )
}
