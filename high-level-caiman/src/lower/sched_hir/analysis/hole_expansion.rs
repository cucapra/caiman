use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use crate::{
    error::{type_error, LocalError},
    lower::sched_hir::{
        cfg::{Cfg, START_BLOCK_ID},
        Hir, HirBody, HirInstr, TripleTag,
    },
    parse::ast::FullType,
};

use super::{
    analyze, bft_transform, dominators::DomTree, get_uses, Backwards, Fact, LiveVars, TransferData,
    UseMap,
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
    let to_init = analyze(cfg, &UninitVars::default());
    let uses = get_uses(cfg);
    let inputs: BTreeSet<_> = input_args.iter().map(|(x, _)| x.to_string()).collect();
    let lives = analyze(cfg, &LiveVars::top());
    let undef_vars = lives
        .get_in_fact(START_BLOCK_ID)
        .live_set
        .difference(&inputs)
        .map(|x| (x.clone(), uses.get(x).unwrap().clone()));
    let uninit_vars = to_init
        .get_in_fact(START_BLOCK_ID)
        .to_init_set
        .difference(&inputs)
        .map(|x| (x.clone(), uses.get(x).unwrap().clone()));

    let r = bft_transform(
        cfg,
        &FillHoleInits::top(uninit_vars.collect(), undef_vars.collect(), dom),
    );
    let start = r.get_in_fact(START_BLOCK_ID);
    if let Some(cannot_init) = start
        .undefined
        .keys()
        .chain(start.uninitialized.keys())
        .next()
    {
        let (block, local) = uses.get(cannot_init).unwrap().iter().next().unwrap();
        let info = cfg.blocks[block]
            .stmts
            .get(*local)
            .map_or(cfg.blocks[block].get_final_info(), Hir::get_info);
        return Err(type_error(
            info,
            &format!("There is no way for '{cannot_init}' to be initialized before its used"),
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct FillHoleInits {
    uninitialized: HashMap<String, HashSet<(usize, usize)>>,
    undefined: UseMap,
    dom: Rc<DomTree>,
}

impl PartialEq for FillHoleInits {
    fn eq(&self, other: &Self) -> bool {
        self.uninitialized.keys().eq(other.uninitialized.keys())
    }
}
impl FillHoleInits {
    fn top(mut uninitialized: UseMap, mut undefined: UseMap, doms: &DomTree) -> Self {
        for i in 0..3 {
            uninitialized.remove(&format!("_dim{i}"));
            undefined.remove(&format!("_dim{i}"));
        }
        Self {
            undefined,
            uninitialized,
            dom: Rc::new(doms.clone()),
        }
    }
    /// For each undefined or uninitialized variable, passes it to `init_var` if the current
    /// point in `data` dominates all uses.
    /// ## Args
    /// * `use_undef` - true for undefined variables, false for uninitialized
    /// * `data` - current program point
    /// * `init_var` - function to call if an undefined or uninitialized variable can
    /// be defined/initialized here
    fn process_vars(
        &mut self,
        use_undef: bool,
        data: &TransferData,
        mut init_var: impl FnMut(String),
    ) {
        let mut to_rem = vec![];
        let dominated = self.dom.dominated(data.block_id);
        for (var, uses) in if use_undef {
            &self.undefined
        } else {
            &self.uninitialized
        } {
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
            if use_undef {
                self.undefined.remove(&v);
            } else {
                self.uninitialized.remove(&v);
            }
        }
    }
}
impl Fact for FillHoleInits {
    fn meet(mut self, other: &Self) -> Self {
        let mut to_remove = vec![];
        for u in self.uninitialized.keys() {
            if !other.uninitialized.contains_key(u) {
                to_remove.push(u.clone());
            }
        }
        for rem in to_remove {
            self.uninitialized.remove(&rem);
        }
        self
    }

    fn transfer_instr(
        &mut self,
        stmt: crate::lower::sched_hir::HirInstr<'_>,
        data: super::TransferData,
    ) {
        if let HirInstr::Stmt(HirBody::Hole {
            dests, initialized, ..
        }) = stmt
        {
            self.process_vars(true, &data, |s| {
                if !dests.iter().any(|(d, _)| d == &s) {
                    dests.push((s, TripleTag::new_unspecified()));
                }
            });
            self.process_vars(false, &data, |s| initialized.push(s));
        }
    }

    type Dir = Backwards;
}

/// An analysis that identifies variables which are used but not initialized.
///
/// This is effectively live vars where writing to a ref constitutes a definition.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct UninitVars {
    /// variables that need to be made usable at this point
    pub to_init_set: BTreeSet<String>,
}

impl Fact for UninitVars {
    fn meet(mut self, other: &Self) -> Self {
        for var in &other.to_init_set {
            self.to_init_set.insert(var.clone());
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: TransferData) {
        if let Some(defs) = match stmt {
            HirInstr::Stmt(HirBody::Hole { .. } | HirBody::VarDecl { rhs: None, .. }) => None,
            HirInstr::Stmt(HirBody::BeginEncoding { encoder, .. }) => Some(vec![encoder.0.clone()]),
            _ => stmt.get_defs(),
        } {
            for var in defs {
                self.to_init_set.remove(&var);
            }
        }
        stmt.get_uses(&mut self.to_init_set);
        let mut s = HashSet::new();
        s.extend(self.to_init_set.iter().cloned());
        if let Some(defs) = stmt.get_write_uses() {
            for var in defs {
                self.to_init_set.remove(&var);
            }
        }
    }

    type Dir = Backwards;
}
