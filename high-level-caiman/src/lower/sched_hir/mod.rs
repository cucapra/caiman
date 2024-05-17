pub mod cfg;
#[allow(clippy::module_inception)]
mod hir;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

pub use hir::*;

use crate::{
    lower::IN_STEM,
    parse::ast::{DataType, SchedulingFunc},
    typing::{Context, Mutability, SchedInfo, ENCODE_DST_FLAGS, ENCODE_SRC_FLAGS},
};
use caiman::assembly::ast::{self as asm};
use caiman::explication::Hole;
use caiman::ir;

use self::{
    analysis::{
        analyze, deduce_val_quots, deref_transform_pass, op_transform_pass, transform_out_ssa,
        transform_to_ssa, ActiveFences, InOutFacts, LiveVars, TagAnalysis,
    },
    cfg::{BasicBlock, Cfg, Edge, FINAL_BLOCK_ID, START_BLOCK_ID},
};
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

/// Information about a high level caiman function
struct FuncInfo {
    name: String,
    input: Vec<(String, TripleTag)>,
    output: Vec<TripleTag>,
}

/// The funclets of a scheduling function.
/// Combines the scheduling function's CFG with its analysis information
pub struct Funclets {
    cfg: Cfg,
    live_vars: analysis::InOutFacts<LiveVars>,
    type_info: analysis::InOutFacts<TagAnalysis>,
    /// Mapping from variable names to their local type. The local type of a variable
    /// is a reference
    // types: HashMap<String, asm::TypeId>,
    /// Mapping from variable names to their data type
    data_types: HashMap<String, DataType>,
    finfo: FuncInfo,
    specs: Rc<Specs>,
    /// Map from block id to the set of output variables captured by a
    /// function call
    captured_out: HashMap<usize, BTreeSet<String>>,
    /// Set of value quotients which are literals in the value specification
    literal_value_classes: HashSet<String>,
    /// Set of variables used in the schedule
    variables: HashSet<String>,
    /// Mapping from device variable to its buffer flags
    flags: HashMap<String, ir::BufferFlags>,
}

/// A specific funclet in a scheduling function.
/// Combines the funclet's basic block with is analysis information.
pub struct Funclet<'a> {
    parent: &'a Funclets,
    block: &'a BasicBlock,
}

impl<'a> Funclet<'a> {
    /// Gets the next blocks in the cfg as `FuncletIds`
    pub fn next_blocks(&self) -> Vec<Hole<asm::FuncletId>> {
        match &self.block.terminator {
            Terminator::FinalReturn(_) => vec![],
            Terminator::Select { .. } => {
                let mut e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| asm::FuncletId(self.parent.funclet_name(id)));
                let mut res = vec![];
                if let Some(true_block) = e.next() {
                    res.push(Hole::Filled(true_block));
                }
                if let Some(false_block) = e.next() {
                    res.push(Hole::Filled(false_block));
                }
                assert_eq!(res.len(), 2);
                res
            }
            Terminator::None
            | Terminator::Return { .. }
            | Terminator::Next(_)
            | Terminator::Call(..)
            | Terminator::CaptureCall { .. }
            | Terminator::Yield(_) => {
                let e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| Hole::Filled(asm::FuncletId(self.parent.funclet_name(id))));
                let res: Vec<_> = e.collect();
                assert!(res.len() <= 1);
                res
            }
        }
    }

    /// Gets the input arguments of this funclet based on the union of the live
    /// variables of all predecessor funclets.
    ///
    /// # Returns
    /// A vector where captured variables will come before non-captured variables.
    /// The returned vector of strings do not contain duplicates and each part of the
    /// result (the captures and non-captures) is sorted alphabetically.
    fn input_vars(&self) -> Vec<String> {
        if self.id() == START_BLOCK_ID {
            self.parent
                .finfo
                .input
                .iter()
                .map(|(name, _)| name.clone())
                .collect()
        } else {
            let preds = self.parent.cfg.predecessors(self.id());
            self.parent.exiting_vars(&preds)
        }
    }

    /// Gets the output arguments of this funclet based on the live out variables
    /// of this block. The returned vector of strings do not contain duplicates and
    /// contains captures **and** non-captures. The entire result is sorted
    #[allow(clippy::option_if_let_else)]
    fn output_vars(&self) -> Vec<(String, TripleTag)> {
        // Schedule call and select occurs "before" funclet ends.
        // Their returns (return of continuation)
        // must match the returns of the funclet.

        if self.id() == FINAL_BLOCK_ID {
            return self
                .parent
                .finfo
                .output
                .iter()
                .enumerate()
                .map(|(idx, _)| {
                    let s = format!("{RET_VAR}{idx}");
                    let t = self.get_out_tag(&s).unwrap().clone();
                    (s, t)
                })
                .collect();
        }

        match self.block.terminator {
            Terminator::Call(..)
            | Terminator::CaptureCall { .. }
            | Terminator::Select { .. }
            | Terminator::Yield(_) => {
                let continuation = self.block.ret_block.unwrap();
                self.parent.get_funclet(continuation).output_vars()
            }
            Terminator::Return { .. } if self.is_final_return() => {
                // final return is a jump to final basic block
                let continuation = self.block.ret_block.unwrap();
                self.parent.get_funclet(continuation).output_vars()
            }
            Terminator::FinalReturn(_)
            | Terminator::None
            | Terminator::Next(_)
            | Terminator::Return { .. } => self
                .parent
                .exiting_vars(&[self.id()])
                .into_iter()
                .map(|v| {
                    (
                        v.clone(),
                        self.parent
                            .type_info
                            .get_out_fact(self.id())
                            .get_tag(&v)
                            .unwrap_or_else(|| {
                                panic!(
                                    "{}: An output tag must be specified for {v}",
                                    self.block.src_loc
                                )
                            })
                            .clone(),
                    )
                })
                .collect(),
        }
    }

    /// Gets the input arguments for each block based on the block's live in variables
    pub fn inputs(&self) -> Vec<asm::FuncletArgument> {
        #[allow(clippy::map_unwrap_or)]
        if self.id() == cfg::START_BLOCK_ID {
            self.parent
                .finfo
                .input
                .iter()
                .map(|(name, _)| asm::FuncletArgument {
                    name: Some(asm::NodeId(name.clone())),
                    typ: self.get_asm_type(name).unwrap(),
                    tags: self
                        .get_input_tag(&format!("{IN_STEM}{name}"))
                        .unwrap()
                        .tags_vec(),
                })
                .collect()
        } else if self.id() == FINAL_BLOCK_ID {
            // final block is just for type conversion
            // final block input and output are the same as the function
            self.parent
                .finfo
                .output
                .iter()
                .enumerate()
                .map(|(idx, _)| {
                    let name = format!("{RET_VAR}{idx}");
                    asm::FuncletArgument {
                        typ: self.get_asm_type(&name).unwrap(),
                        tags: self.get_input_tag(&name).unwrap().tags_vec(),
                        name: Some(asm::NodeId(name)),
                    }
                })
                .collect()
        } else {
            self.input_vars()
                .iter()
                .map(|var| asm::FuncletArgument {
                    name: Some(asm::NodeId(var.clone())),
                    typ: self.get_asm_type(var).unwrap(),
                    tags: self
                        .get_input_tag(var)
                        .unwrap_or_else(|| {
                            panic!(
                                "{}: An input tag must be specified for {var}",
                                self.block.src_loc
                            )
                        })
                        .tags_vec(),
                })
                .collect()
        }
    }

    /// Gets the input tag of the specified variable, handling input overrides
    pub fn get_input_tag(&self, var: &str) -> Option<TripleTag> {
        let ovr = self
            .parent
            .type_info
            .get_out_fact(self.id())
            .get_input_override(var)
            .cloned();
        match (
            self.parent
                .type_info
                .get_in_fact(self.id())
                .get_tag(var)
                .cloned(),
            ovr,
        ) {
            (orig, None) => orig,
            (None, Some(ovr)) => {
                // for auto generated sequence-ifs, we handle this by creating
                // an in annotation for the variable
                Some(ovr)
            }
            (Some(mut orig), Some(ovr)) => {
                orig.set_specified_info(ovr);
                Some(orig.clone())
            }
        }
    }

    /// Gets the return arguments of a funclet based on the block's live out variables
    pub fn outputs(&self) -> Vec<asm::FuncletArgument> {
        if self.id() == cfg::FINAL_BLOCK_ID {
            self.parent
                .finfo
                .output
                .iter()
                .enumerate()
                .map(|(idx, _)| {
                    let name = format!("{RET_VAR}{idx}");
                    asm::FuncletArgument {
                        typ: self.get_asm_type(&name).unwrap(),
                        tags: self.get_out_tag(&name).unwrap().tags_vec(),
                        name: Some(asm::NodeId(name)),
                    }
                })
                .collect()
        } else {
            // TODO: re-evaluate if this is correct for the general case
            self.output_vars()
                .into_iter()
                .map(|(var, tag)| asm::FuncletArgument {
                    name: None,
                    typ: self.get_asm_type(&var).unwrap(),
                    tags: tag.tags_vec(),
                })
                .collect()
        }
    }

    /// Gets the nodes that exit this funclet
    pub fn output_args(&self) -> Vec<Hole<asm::NodeId>> {
        self.parent
            .live_vars
            .get_out_fact(self.block.id)
            .live_set()
            .iter()
            .cloned()
            .map(asm::NodeId)
            .map(Hole::Filled)
            .collect()
    }

    #[inline]
    pub fn stmts(&self) -> &[hir::HirBody] {
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
    pub const fn specs(&self) -> &Rc<Specs> {
        &self.parent.specs
    }

    /// Gets the funclet name of the join point. The join point is the
    /// first successor funclet shared by all immediate successors of this funclet.
    #[inline]
    pub fn join_funclet(&self) -> asm::FuncletId {
        let id = match self.parent.cfg.graph.get(&self.id()).unwrap() {
            Edge::Select { .. } | Edge::Next(_) => self.block.ret_block.unwrap_or(FINAL_BLOCK_ID),
            Edge::None => FINAL_BLOCK_ID,
        };
        asm::FuncletId(self.parent.funclet_name(id))
    }

    /// Returns true if this funclet is the sole predecessor to the final block.
    /// In other words, this funclet is a top level funclet with a `return`
    /// terminator.
    pub fn is_final_return(&self) -> bool {
        self.parent.cfg.predecessors(FINAL_BLOCK_ID).len() == 1
            && self.parent.cfg.predecessors(FINAL_BLOCK_ID).first() == Some(&self.id())
    }

    /// Gets the tag of the specified variable at the end of the funclet
    #[inline]
    pub fn get_out_tag(&self, var: &str) -> Option<&TripleTag> {
        self.parent.type_info.get_out_fact(self.id()).get_tag(var)
    }

    /// Gets the data type of the specified variable. Note that
    /// the data type of a variable will be the data type of the value,
    /// not a reference data type
    #[inline]
    fn get_dtype(&self, var: &str) -> Option<&DataType> {
        self.parent.data_types.get(var)
    }

    #[inline]
    pub fn get_storage_type(&self, var: &str) -> Option<asm::FFIType> {
        self.get_dtype(var).map(DataType::storage_type)
    }

    /// Gets the assembly type for a variable, considering the place of the
    /// variable
    fn get_asm_type(&self, var: &str) -> Result<asm::TypeId, String> {
        if let Some(flags) = self.parent.flags.get(var) {
            let suffix = if *flags == ENCODE_SRC_FLAGS {
                "::gs"
            } else if *flags == ENCODE_DST_FLAGS {
                "::gd"
            } else {
                return Err(format!("{}: Invalid flags for {var}", self.block.src_loc));
            };
            Ok(asm::TypeId(format!(
                "{}{}",
                self.get_dtype(var).unwrap().asm_type().0,
                suffix
            )))
        } else if let Some(dt) = self.get_dtype(var) {
            Ok(dt.asm_type())
        } else {
            Err(format!("{}: Missing type for {var}", self.block.src_loc))
        }
    }

    /// Returns true if the specified tag is a literal node in the value specification
    pub fn is_literal_value(&self, t: &asm::RemoteNodeId) -> bool {
        t.node.as_ref().map_or(false, |n| {
            n.as_ref()
                .opt()
                .map_or(false, |r| self.parent.literal_value_classes.contains(&r.0))
        })
    }

    /// Returns true if the specified variable is a mutable reference or a mutable variable
    pub fn is_var_or_ref(&self, v: &str) -> bool {
        self.parent.variables.contains(v) || matches!(self.get_dtype(v), Some(DataType::Ref(_)))
    }

    /// Gets a map of device variables to their buffer flags
    pub const fn get_flags(&self) -> &'a HashMap<String, ir::BufferFlags> {
        self.parent.get_flags()
    }
}

impl Funclets {
    /// Updates terminators by replacing temporary terminators with their respective
    /// versions which contain more information computed by analyses.
    ///
    /// Replaces `Terminator::None` with `Terminator::Next` which is required for
    /// lowering. `Terminator::Next` contains information about which variables
    /// escape the basic block while `Terminator::None` does not. We use
    /// `Terminator::None` as a temporary until CFG analyses can be performed.
    ///
    /// Also replaces `Terminator::Call` with `Terminator::CaptureCall` which
    /// contains information about which variables are captured by the call.
    /// # Returns
    /// A map from block id to the set of output variables captured by a
    /// function call and a map from block id to the set of output variables
    /// returned by the block (i.e. not captured by a function call)
    fn terminator_transform_pass(
        cfg: &mut Cfg,
        live_vars: &InOutFacts<LiveVars>,
    ) -> HashMap<usize, BTreeSet<String>> {
        let mut captured_out = HashMap::new();
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
            } else if let Terminator::Call(dest, call) = bb.terminator.clone() {
                let mut captures = BTreeSet::new();
                for v in &live_vars.get_out_fact(*id).live_set {
                    let mut handled = false;
                    for (returned, _) in &dest {
                        if v == returned {
                            handled = true;
                            break;
                        }
                    }
                    if !handled {
                        captures.insert(v.clone());
                    }
                }
                captured_out.insert(*id, captures.clone());
                bb.terminator = Terminator::CaptureCall {
                    dests: dest,
                    call,
                    captures,
                };
            } else if let Terminator::Return {
                dests, passthrough, ..
            } = &mut bb.terminator
            {
                let live_out = live_vars.get_out_fact(*id).live_set();
                for v in live_out {
                    if !dests.iter().any(|(d, _)| d == v) {
                        passthrough.push(v.clone());
                    }
                }
            } else if let Terminator::Yield(captures) = &mut bb.terminator {
                let lives = live_vars.get_out_fact(*id).live_set();
                *captures = lives.iter().cloned().collect();
                captured_out.insert(*id, lives.clone());
            }
        }
        captured_out
    }

    /// Creates a new `Funclets` from a scheduling function by performing analyses
    /// and transforming the scheduling func into a canonical CFG of lowered HIR.
    pub fn new(f: SchedulingFunc, specs: &Specs, ctx: &Context) -> Self {
        let mut cfg = Cfg::new(f.statements, &f.output, ctx);
        let (mut data_types, variables, flags) =
            Self::collect_types(ctx.scheds.get(&f.name).unwrap().unwrap_sched());

        deref_transform_pass(&mut cfg, &mut data_types, &variables);
        op_transform_pass(&mut cfg, &data_types);
        let live_vars = analyze(&mut cfg, &LiveVars::top());
        let captured_out = Self::terminator_transform_pass(&mut cfg, &live_vars);
        cfg = transform_to_ssa(cfg, &live_vars);
        let specs_rc = Rc::new(specs.clone());
        let mut hir_inputs: Vec<_> = f
            .input
            .iter()
            .map(|(name, typ)| (name.clone(), TripleTag::from_fulltype_opt(typ)))
            .collect();
        let mut hir_outputs: Vec<_> = f.output.iter().map(TripleTag::from_fulltype).collect();

        deduce_val_quots(
            &mut hir_inputs,
            &mut hir_outputs,
            &mut cfg,
            &ctx.specs[&specs.value.0],
            ctx,
        )
        .unwrap();
        cfg = transform_out_ssa(cfg);
        let type_info = analyze(
            &mut cfg,
            &TagAnalysis::top(&hir_inputs, &hir_outputs, &data_types),
        );
        // TODO: populate active fences from function inputs
        let _ = analyze(&mut cfg, &ActiveFences::top(&[]));
        let finfo = FuncInfo {
            name: f.name,
            input: hir_inputs,
            output: hir_outputs,
        };
        Self {
            cfg,
            live_vars,
            type_info,
            data_types,
            finfo,
            specs: specs_rc,
            captured_out,
            literal_value_classes: ctx.specs[&specs.value.0].nodes.literal_classes(),
            variables,
            flags,
        }
    }

    /// Collects a map of variable names to their base types as local types,
    /// including the output variables (ex. `_out0`)
    /// The base type of a variable is the reference type if the variable.
    /// # Arguments
    /// * `f` - The scheduling function information to collect types from
    /// # Returns
    /// A tuple of the map of variable names to their local types and the map of
    /// variable names to their data types, and a set of mutable variables
    #[allow(clippy::type_complexity)]
    fn collect_types(
        f: &SchedInfo,
    ) -> (
        HashMap<String, DataType>,
        HashSet<String>,
        HashMap<String, ir::BufferFlags>,
    ) {
        let mut data_types = f.types.clone();
        let mut variables = HashSet::new();
        for (var, typ) in &f.types {
            if f.defined_names.get(var) == Some(&Mutability::Mut) {
                data_types.insert(var.to_string(), DataType::Ref(Box::new(typ.clone())));
                variables.insert(var.to_string());
            }
        }
        for (id, out_ty) in f.dtype_sig.output.iter().enumerate() {
            data_types.insert(format!("{RET_VAR}{id}"), out_ty.clone());
        }
        (data_types, variables, f.flags.clone())
    }

    /// Gets the funclet with the given id
    #[inline]
    pub fn get_funclet(&self, id: usize) -> Funclet {
        Funclet {
            parent: self,
            block: self.cfg.blocks.get(&id).unwrap(),
        }
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

    /// Get's the terminator for a basic block with the given ID
    #[inline]
    fn terminator(&self, block_id: usize) -> &Terminator {
        &self.cfg.blocks[&block_id].terminator
    }

    /// Gets the definitions of a terminator for a basic block
    fn terminator_dests(&self, block_id: usize) -> Vec<String> {
        self.terminator(block_id).get_defs().unwrap_or_default()
    }

    /// Gets the list of variables that exit a block.
    /// The returned list of variable names have 3 sections. The first section is
    /// the captured variables in alphabetical order. The second section is the
    /// terminator destinations ordered how they are passed around in the program,
    /// and the final section are other returns, ordered alphabetically.
    fn exiting_vars(&self, block_ids: &[usize]) -> Vec<String> {
        let captures: BTreeSet<_> = block_ids
            .iter()
            .filter_map(|id| self.captured_out.get(id))
            .flatten()
            .cloned()
            .collect();
        assert!(captures.is_empty() || block_ids.len() == 1);
        let term_dests: Vec<_> = block_ids
            .iter()
            .map(|id| self.terminator_dests(*id))
            .collect();
        assert!(term_dests.windows(2).all(|wnd| wnd[0] == wnd[1]));
        let term_dests = term_dests.into_iter().next().unwrap_or_default();
        let returns: BTreeSet<_> = block_ids
            .iter()
            .flat_map(|id| self.live_vars.get_out_fact(*id).live_set().iter())
            .filter(|v| !captures.contains(*v) && !term_dests.contains(*v))
            .cloned()
            .collect();
        captures
            .into_iter()
            .chain(term_dests.into_iter())
            .chain(returns)
            .collect()
    }

    /// Get's a map of device variables to their buffer flags
    pub const fn get_flags(&self) -> &HashMap<String, ir::BufferFlags> {
        &self.flags
    }
}
