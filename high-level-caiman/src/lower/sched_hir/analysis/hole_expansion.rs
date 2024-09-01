use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use caiman::explication::expir::BufferFlags;

use crate::{
    error::{type_error, LocalError},
    lower::{
        sched_hir::{
            cfg::{Cfg, START_BLOCK_ID},
            Hir, HirBody, HirInstr, HirOp, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{DataType, FullType},
    typing::{MetaVar, NodeEnv},
};

use super::{
    analyze, dominators::DomTree, get_uses, ssa::ssa_original_name, Backwards, Fact, Forwards,
    LiveVars, TransferData, UseMap,
};

/// Sets variables a hole should define or initialize. For a given variable that
/// is undefined or uninitialized, the hole that dominates all uses of the variable
/// which has the shortest path to the terminator block is chosen to initialize or
/// define those variables.
///
/// In this context, initialize means to make usable. All definitions initialize
/// a variable except for reference variables.
pub fn fill_hole_initializers(
    cfg: &mut Cfg,
    input_args: &[(String, Option<FullType>)],
    dom: &DomTree,
) -> Result<(), LocalError> {
    let uses = get_uses(cfg);
    let inputs: BTreeSet<_> = input_args.iter().map(|(x, _)| x.to_string()).collect();
    let lives = analyze(cfg, LiveVars::top());
    let undef_vars = lives
        .get_in_fact(START_BLOCK_ID)
        .live_set
        .difference(&inputs)
        .map(|x| (x.clone(), uses.get(x).unwrap().clone()));

    let r = analyze(cfg, FillHoleInits::top(undef_vars.collect(), dom));
    let start = r.get_in_fact(START_BLOCK_ID);
    if let Some(cannot_init) = start.undefined.keys().next() {
        let (block, local) = uses.get(cannot_init).unwrap().iter().next().unwrap();
        let info = cfg.blocks[block]
            .stmts
            .get(*local)
            .map_or(cfg.blocks[block].get_final_info(), Hir::get_info);
        return Err(type_error(
            info,
            &format!("There is no way for '{cannot_init}' to be defined before it's used"),
        ));
    }
    Ok(())
}

/// An analysis that sets an undefined variable as a destination of the hole
/// closest to the exit block which dominates all uses
#[derive(Clone)]
struct FillHoleInits<'a> {
    undefined: UseMap,
    dom: &'a DomTree,
}

impl<'a> PartialEq for FillHoleInits<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.undefined.keys().eq(other.undefined.keys())
    }
}
impl<'a> FillHoleInits<'a> {
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
impl<'a> Fact for FillHoleInits<'a> {
    fn meet(mut self, other: &Self) -> Self {
        let mut to_remove = vec![];
        for u in self.undefined.keys() {
            if !other.undefined.contains_key(u) {
                to_remove.push(u.clone());
            }
        }
        for rem in to_remove {
            self.undefined.remove(&rem);
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: super::TransferData) {
        if let HirInstr::Stmt(HirBody::Hole { dests, .. }) = stmt {
            self.process_vars(&data, |s| {
                if !dests.iter().any(|(d, _)| d == &s) {
                    dests.push((s, TripleTag::new_unspecified()));
                }
            });
        }
    }

    type Dir = Backwards;
}

/// An analysis that identifies variables which are used but not initialized.
///
/// This is effectively live vars where writing to a ref constitutes a definition.
#[derive(Clone)]
pub struct UsabilityAnalysis<'a> {
    /// variables that need to be made usable at this point
    /// The only way to consume a variable would be to pass it through a function,
    /// which creates a new definition of the variable. So once a variable becomes
    /// usable, we can treat it as usable for the rest of the function
    pub to_init: HashSet<String>,
    /// Map from node name to variables that depend on it
    deps: Rc<HashMap<String, Vec<String>>>,
    env: &'a NodeEnv,
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
            env,
            deps: Rc::new(deps),
            selects,
        }
    }

    fn remove_dependents_of(&mut self, var_name: &str) {
        if let Some(class_name) = self.env.get_node_name(var_name) {
            self.remove_dependents_of_class(&class_name);
        }
    }

    fn remove_dependents_of_class(&mut self, class_name: &str) {
        let class_name = format!("${class_name}");
        if let Some(to_remove) = self.deps.get(&class_name) {
            for r in to_remove {
                self.to_init.remove(r);
            }
        }
    }
}

impl<'a> Fact for UsabilityAnalysis<'a> {
    fn meet(mut self, other: &Self) -> Self {
        self.to_init = self.to_init.intersection(&other.to_init).cloned().collect();
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: TransferData) {
        if let Some(defs) = match stmt {
            HirInstr::Stmt(HirBody::Hole { .. } | HirBody::VarDecl { rhs: None, .. }) => None,
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
        match stmt {
            HirInstr::Stmt(HirBody::EncodeDo { dests, .. })
            | HirInstr::Tail(Terminator::CaptureCall { dests, .. }) => {
                let id = tuple_id(&dests.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>());
                self.remove_dependents_of(&id);
                for (d, _) in dests {
                    self.remove_dependents_of(d);
                }
            }
            HirInstr::Stmt(
                HirBody::Phi { dest, .. }
                | HirBody::ConstDecl { lhs: dest, .. }
                | HirBody::VarDecl {
                    lhs: dest,
                    rhs: Some(_),
                    ..
                }
                | HirBody::DeviceCopy { dest, .. }
                | HirBody::RefStore { lhs: dest, .. }
                | HirBody::RefLoad { dest, .. },
            ) => {
                self.remove_dependents_of(dest);
            }
            HirInstr::Stmt(HirBody::Sync { dests, .. }) => {
                for (dest, _) in dests.processed() {
                    self.remove_dependents_of(dest);
                }
            }
            HirInstr::Stmt(HirBody::Op { dests, op, .. }) => {
                let id = match op {
                    HirOp::External(_) => {
                        tuple_id(&dests.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>())
                    }
                    HirOp::Binary(_) | HirOp::Unary(_) => dests[0].0.clone(),
                };
                self.remove_dependents_of(&id);
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
            HirInstr::Stmt(HirBody::Hole { initialized, .. }) => {
                initialized.clone_from(&self.to_init);
            }
            HirInstr::Stmt(
                HirBody::InAnnotation(..)
                | HirBody::OutAnnotation(..)
                | HirBody::Submit { .. }
                | HirBody::BeginEncoding { .. }
                | HirBody::VarDecl { rhs: None, .. },
            )
            | HirInstr::Tail(
                Terminator::None(..)
                | Terminator::FinalReturn(..)
                | Terminator::Yield(..)
                | Terminator::Next(..)
                | Terminator::Call(..),
            ) => {}
        }
    }

    type Dir = Backwards;
}

#[derive(Clone)]
pub struct UninitCheck {
    maybe_uninit: HashSet<String>,
    pub error: Option<LocalError>,
}

/// Follow up pass to usability analysis that errors if something can be used before it is initialized
impl UninitCheck {
    pub fn top<'a>(
        mut maybe_uninit: HashSet<String>,
        inputs: impl Iterator<Item = &'a String>,
    ) -> Self {
        for i in 0..4 {
            maybe_uninit.remove(&format!("_dim{i}"));
        }
        for input in inputs {
            maybe_uninit.remove(input);
        }
        Self {
            maybe_uninit,
            error: None,
        }
    }
}

impl PartialEq for UninitCheck {
    fn eq(&self, other: &Self) -> bool {
        self.maybe_uninit == other.maybe_uninit
    }
}

impl Fact for UninitCheck {
    fn meet(mut self, other: &Self) -> Self {
        self.maybe_uninit.extend(other.maybe_uninit.iter().cloned());
        if self.error.is_none() && other.error.is_some() {
            self.error.clone_from(&other.error);
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: TransferData) {
        match stmt {
            HirInstr::Stmt(HirBody::Hole { initialized, .. }) => {
                for i in initialized.iter() {
                    self.maybe_uninit.remove(i);
                }
            }
            HirInstr::Stmt(HirBody::InAnnotation(..) | HirBody::OutAnnotation(..)) => {}
            x if self.error.is_none() => {
                let mut uses = BTreeSet::new();
                x.get_uses(&mut uses);
                if let Some(writes) = x.get_write_uses() {
                    for w in writes {
                        uses.remove(&w);
                    }
                }
                for u in &self.maybe_uninit {
                    if uses.contains(u) {
                        self.error = Some(type_error(
                            x.get_info(),
                            &format!(
                                "'{}' is used before it can be initialized",
                                ssa_original_name(u)
                            ),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    type Dir = Forwards;
}
