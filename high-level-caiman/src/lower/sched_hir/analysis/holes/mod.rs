use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use caiman::explication::expir::BufferFlags;
use init_synth::{build_init_set, fill_initializers};

use crate::{
    error::{hir_to_source_name, Info, LocalError},
    lower::{
        sched_hir::{
            cfg::{Cfg, FINAL_BLOCK_ID, START_BLOCK_ID},
            FuncletTypeInfo, Hir, HirBody, HirInstr, HirTerm, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{DataType, Flow, FullType},
    type_error,
    typing::{MetaVar, NodeEnv},
};

use super::{
    analyze, dominators::DomTree, get_uses, ssa::ssa_original_name, Backwards, DomInfo, Fact,
    Forwards, LiveVars, TransferData, UseMap,
};

mod init_synth;

/// Sets variables a hole should define. For a given variable that
/// is undefined, the hole that dominates all uses of the variable
/// which has the shortest path to the terminator block is chosen to define those variables.
pub fn set_hole_defs(
    cfg: &mut Cfg,
    input_args: &[(String, Option<FullType>)],
    dom: &DomTree,
) -> Result<(), LocalError> {
    let uses = get_uses(cfg);
    let inputs: BTreeSet<_> = input_args.iter().map(|(x, _)| x.to_string()).collect();
    let lives = analyze(cfg, LiveVars::top())?;
    let undef_vars = lives
        .get_in_fact(START_BLOCK_ID)
        .live_set
        .difference(&inputs)
        .map(|x| (x.clone(), uses.get(x).unwrap().clone()));

    let r = analyze(cfg, FillHoleDefs::top(undef_vars.collect(), dom))?;
    let start = r.get_in_fact(START_BLOCK_ID);
    if let Some(cannot_init) = start.undefined.keys().next() {
        let (block, local) = uses.get(cannot_init).unwrap().iter().next().unwrap();
        let info = cfg.blocks[block]
            .stmts
            .get(*local)
            .map_or(cfg.blocks[block].get_final_info(), Hir::get_info);
        return Err(type_error!(
            info,
            "There is no way for '{}' to be defined before it's used",
            hir_to_source_name(cannot_init)
        ));
    }
    Ok(())
}

/// An analysis that sets an undefined variable as a destination of the hole
/// closest to the exit block which dominates all uses
#[derive(Clone)]
struct FillHoleDefs<'a> {
    undefined: UseMap,
    dom: &'a DomTree,
}

impl<'a> PartialEq for FillHoleDefs<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.undefined.keys().eq(other.undefined.keys())
    }
}
impl<'a> FillHoleDefs<'a> {
    fn top(mut undefined: UseMap, doms: &'a DomTree) -> Self {
        for i in 0..3 {
            undefined.remove(&format!("_dim{i}"));
        }
        Self {
            undefined,
            dom: doms,
        }
    }
    /// For each undefined or uninitialized variable, passes it to `init_var` if the current
    /// point in `data` dominates all uses.
    /// ## Args
    /// * `data` - current program point
    /// * `init_var` - function to call if an undefined or uninitialized variable can
    /// be defined/initialized here
    fn process_vars(&mut self, data: &TransferData, mut init_var: impl FnMut(String)) {
        let mut to_rem = vec![];
        let dominated = self.dom.dominated(data.block_id);
        for (var, uses) in &self.undefined {
            let mut can_init = true;
            for (use_block, use_local) in uses {
                if !dominated.contains(use_block)
                    || *use_block == data.block_id && *use_local < data.local_instr_id
                {
                    can_init = false;
                    break;
                }
            }
            if can_init {
                init_var(var.clone());
                to_rem.push(var.clone());
            }
        }
        for v in to_rem {
            self.undefined.remove(&v);
        }
    }
}
impl<'a> Fact for FillHoleDefs<'a> {
    fn meet(mut self, other: &Self, _: Info) -> Result<Self, LocalError> {
        let mut to_remove = vec![];
        for u in self.undefined.keys() {
            if !other.undefined.contains_key(u) {
                to_remove.push(u.clone());
            }
        }
        for rem in to_remove {
            self.undefined.remove(&rem);
        }
        Ok(self)
    }

    fn transfer_instr(
        &mut self,
        stmt: HirInstr<'_>,
        data: super::TransferData,
    ) -> Result<(), LocalError> {
        if let HirInstr::Stmt(HirBody::Hole { dests, .. }) = stmt {
            self.process_vars(&data, |s| {
                if !dests.iter().any(|(d, _)| d == &s) {
                    dests.push((s, TripleTag::new_unspecified()));
                }
            });
        }
        Ok(())
    }

    type Dir = Backwards;
}

/// An analysis that identifies variables that need to be initialized (made usable).
#[derive(Clone)]
struct UsabilityAnalysis<'a> {
    /// variables that need to be made usable at this point
    /// The only way to consume a variable would be to pass it through a function,
    /// which creates a new definition of the variable. So once a variable becomes
    /// usable, we can treat it as usable for the rest of the function
    pub to_init: HashSet<String>,
    /// Map from node name to variables that depend on it
    val_deps: Rc<HashMap<String, Vec<String>>>,
    val_env: &'a NodeEnv,
    selects: &'a HashMap<usize, String>,
}

impl<'a> PartialEq for UsabilityAnalysis<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.to_init == other.to_init
    }
}

impl<'a> UsabilityAnalysis<'a> {
    pub fn top(
        env: &'a NodeEnv,
        data_types: &HashMap<String, DataType>,
        flags: &HashMap<String, BufferFlags>,
        selects: &'a HashMap<usize, String>,
    ) -> Self {
        let mut to_init = HashSet::new();
        let mut deps: HashMap<_, Vec<_>> = HashMap::new();

        for ssa_var in env.get_sched_vars() {
            let var = ssa_original_name(ssa_var);
            if let (Some(node_name), Some(typ)) = (env.get_node_name(ssa_var), data_types.get(&var))
            {
                if matches!(typ, DataType::Ref(_)) || flags.contains_key(&var) {
                    to_init.insert(ssa_var.clone());
                    for dep in env.dependencies(&MetaVar::new_class_name(&node_name)) {
                        deps.entry(dep).or_default().push(ssa_var.clone());
                    }
                }
            }
        }
        Self {
            to_init,
            val_env: env,
            val_deps: Rc::new(deps),
            selects,
        }
    }

    fn remove_dependents_of(&mut self, var_name: &str) {
        if let Some(class_name) = self.val_env.get_node_name(var_name) {
            self.remove_dependents_of_class(&class_name);
        }
    }

    fn remove_dependents_of_class(&mut self, class_name: &str) {
        let class_name = format!("${class_name}");
        if let Some(to_remove) = self.val_deps.get(&class_name) {
            for r in to_remove {
                self.to_init.remove(r);
            }
        }
    }
}

impl<'a> Fact for UsabilityAnalysis<'a> {
    fn meet(mut self, other: &Self, _: Info) -> Result<Self, LocalError> {
        self.to_init = self.to_init.intersection(&other.to_init).cloned().collect();
        Ok(self)
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: TransferData) -> Result<(), LocalError> {
        // TODO: we can always recalculate something so that even when something is used,
        // that doesn't mean that things that it depends on has to be initialized

        // for now, we only say that something can't be initialized if a control flow operation
        // depends on it and we have reached said control flow. This is bc a I'm assuming holes
        // only work within a basic block right now.
        // In other words:
        /* ```
        val foo()
            x :- a if c else b
            z :- x * 20

        fn foo_impl()
            ???;    // `x` cannot be initialized here since it depends on control flow
            if ??? {
                ???
            } else {
                ???
            }
            ???     // `x` can be initialized here
        ``` */
        match stmt {
            HirInstr::Tail(Terminator::CaptureCall { dests, .. }) => {
                let id = tuple_id(&dests.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>());
                self.remove_dependents_of(&id);
                for (d, _) in dests {
                    self.remove_dependents_of(d);
                }
            }
            HirInstr::Tail(Terminator::Return { dests, .. }) => {
                for (d, _) in dests {
                    self.remove_dependents_of(d);
                }
            }
            HirInstr::Tail(Terminator::Select { dests, .. }) => {
                for (d, _) in dests {
                    self.remove_dependents_of(d);
                }
                if let Some(select_node) = self.selects.get(&data.block_id) {
                    self.remove_dependents_of_class(select_node);
                }
            }
            // HirInstr::Stmt(HirBody::Hole { initialized, .. }) => {
            //     initialized.clone_from(&self.to_init);
            // }
            _ => {}
        };
        if let Some(defs) = match stmt {
            HirInstr::Stmt(HirBody::VarDecl { rhs: None, .. }) => None,
            HirInstr::Stmt(HirBody::BeginEncoding { encoder, .. }) => Some(vec![encoder.0.clone()]),
            _ => stmt.get_defs(),
        } {
            for var in defs {
                self.to_init.remove(&var);
            }
        }
        if let Some(defs) = stmt.get_write_uses() {
            for var in defs {
                self.to_init.remove(&var);
            }
        }
        Ok(())
    }

    type Dir = Backwards;
}
/// Pass that errors if a reference or GPU variable's value is used before it is
/// made usable. This pass is conservative in the sense that it will only error
/// if there is definitely a problem.
#[derive(Clone)]
struct UninitCheck<'a> {
    maybe_uninit: HashSet<String>,
    env: &'a NodeEnv,
    dtypes: &'a HashMap<String, DataType>,
}

impl<'a> UninitCheck<'a> {
    /// # Arguments
    /// * `maybe_uninit` - set of references and GPU variables to check to see if
    /// they're value is used before they're `usable`
    pub fn top(
        mut maybe_uninit: HashSet<String>,
        inputs: &[(String, TripleTag)],
        env: &'a NodeEnv,
        dtypes: &'a HashMap<String, DataType>,
    ) -> Self {
        maybe_uninit = maybe_uninit
            .into_iter()
            .map(|x| ssa_original_name(&x))
            .collect();
        for i in 0..4 {
            maybe_uninit.remove(&format!("_dim{i}"));
        }
        for (input, tag) in inputs {
            if tag.value.flow != Some(Flow::Dead) {
                maybe_uninit.remove(input);
            }
        }
        Self {
            maybe_uninit,
            env,
            dtypes,
        }
    }
}

impl<'a> PartialEq for UninitCheck<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.maybe_uninit == other.maybe_uninit
    }
}

/// Get's the uses of a statement that must be value `usable`
/// # Arguments
/// * `on_remove` - a callback invoked when a variable is removed from the use set during
/// computation of the `usable` use set
fn get_usable_uses(
    stmt: &HirInstr,
    env: &NodeEnv,
    dtypes: &HashMap<String, DataType>,
    mut on_remove: impl FnMut(&str),
) -> BTreeSet<String> {
    let mut uses = BTreeSet::new();
    stmt.get_uses(&mut uses);
    if let Some(writes) = stmt.get_write_uses() {
        for w in writes {
            uses.remove(&w);
            on_remove(&w);
        }
    }
    if matches!(
        stmt,
        HirInstr::Stmt(
            HirBody::VarDecl { rhs: Some(_), .. }
                | HirBody::ConstDecl {
                    rhs: HirTerm::Hole { .. },
                    ..
                }
        )
    ) {
        for d in stmt.get_defs().unwrap() {
            uses.remove(&d);
            on_remove(&d);
        }
    }
    match stmt {
        HirInstr::Tail(Terminator::CaptureCall { dests, .. }) => {
            // special handling for calls, which are currently the only way for a reference to be
            // used (in the traditional compilers sense) without consuming it
            let t = tuple_id(&dests.iter().map(|(nm, _)| nm.clone()).collect::<Vec<_>>());
            if let Some(class_name) = env.get_node_name(&t) {
                let deps = env.dependencies(&MetaVar::new_class_name(&class_name));
                uses.retain(|u| {
                    env.get_node_name(u)
                        .map_or(false, |node| deps.contains(&node))
                });
            } else {
                // if the call is unknown, assume it uses nothing
                uses.clear();
            }
            for (d, _) in dests {
                if env.get_node_name(d).is_some() {
                    uses.remove(d);
                    on_remove(d);
                }
            }
        }
        HirInstr::Stmt(HirBody::Submit { src, .. }) => {
            if let Some(DataType::Encoder(Some(rec))) = dtypes.get(&ssa_original_name(src)) {
                // all encoder members must be usable at the submit
                if let DataType::RemoteObj { all, .. } = &**rec {
                    for (mem, _) in all {
                        uses.insert(format!("{}::{mem}", ssa_original_name(src)));
                    }
                }
            }
        }
        _ => {}
    }
    uses
}

impl<'a> Fact for UninitCheck<'a> {
    fn meet(mut self, other: &Self, _: Info) -> Result<Self, LocalError> {
        self.maybe_uninit.extend(other.maybe_uninit.iter().cloned());
        Ok(self)
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: TransferData) -> Result<(), LocalError> {
        match stmt {
            HirInstr::Stmt(HirBody::Hole { initialized, .. }) => {
                for i in initialized.iter() {
                    self.maybe_uninit.remove(&ssa_original_name(i));
                }
            }
            HirInstr::Stmt(HirBody::InAnnotation(..) | HirBody::OutAnnotation(..))
            | HirInstr::Tail(Terminator::Next(..)) => {}
            HirInstr::Tail(Terminator::Return { dests, .. }) => {
                for (d, _) in dests {
                    self.maybe_uninit.remove(&ssa_original_name(d));
                }
            }
            x => {
                let uses = get_usable_uses(&x, self.env, self.dtypes, |x| {
                    self.maybe_uninit.remove(&ssa_original_name(x));
                });
                let uses: HashSet<_> = uses.into_iter().map(|x| ssa_original_name(&x)).collect();
                for u in &self.maybe_uninit {
                    if uses.contains(u) {
                        return Err(type_error!(
                            x.get_info(),
                            "'{}' is used before it can be initialized",
                            hir_to_source_name(u)
                        ));
                    }
                }
            }
        };
        Ok(())
    }

    type Dir = Forwards;
}

/// Sets the variables each hole will initialize. Errors if this cannot be done.
pub fn set_hole_initializations(
    cfg: &mut Cfg,
    val_env: &NodeEnv,
    type_info: &FuncletTypeInfo,
    selects: &HashMap<usize, String>,
    hir_inputs: &[(String, TripleTag)],
    outputs: &[FullType],
) -> Result<(), LocalError> {
    let res = analyze(
        cfg,
        UsabilityAnalysis::top(val_env, &type_info.data_types, &type_info.flags, selects),
    )?;
    // set of all references and GPU variables
    let maybe_uninit = res.get_out_fact(FINAL_BLOCK_ID).to_init.clone();
    let dinfo = DomInfo::new(cfg);
    let initializers = build_init_set(
        cfg,
        &maybe_uninit,
        val_env,
        outputs,
        hir_inputs,
        &type_info.data_types,
        &dinfo,
    );
    fill_initializers(cfg, initializers);
    analyze(
        cfg,
        UninitCheck::top(maybe_uninit, hir_inputs, val_env, &type_info.data_types),
    )?;
    Ok(())
}
