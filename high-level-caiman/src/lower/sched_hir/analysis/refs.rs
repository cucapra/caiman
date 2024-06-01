//! The deref transform pass converts uses of references to load HIR
//! instructions. Variable assignments become stores to references via
//! syntax-directed IR lowering of the AST. Variable uses become uses of loads
//! from references from this pass. We do this
//! transformation locally, and reuse results. So for example:
//!
//! ```text
//! var v;
//! // ...
//! let x = v * v;
//! ley y = v + x;
//! v = y;
//! let z = v * v;
//! ```
//!
//! becomes something like:
//!
//! ```text
//! var _v_ref;
//! // ...
//! let v = *_v_ref;
//! let x = v * v;
//! ley y = v + x;
//! _v_ref <- y;
//!
//! let v = *_v_ref;
//! let z = v * v;
//! ```
//! However, because this is a local transformation, we can't reuse results
//! across funclet/block boundaries.
//!
//! We also insert loads and stores for variables around function calls to
//! support capturing references across function calls. We do not do this
//! for references.

use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::{
    enum_cast,
    error::Info,
    lower::sched_hir::{
        cfg::{BasicBlock, Cfg},
        Hir, HirBody, HirInstr, HirOp, Terminator, TripleTag, UseType,
    },
    parse::ast::{DataType, SchedTerm, Uop},
};

use super::{analyze, Fact, Forwards, InOutFacts, LiveVars};

/// Transforms uses of references into uses of values by inserting
/// deref instructions (loads). A loaded value is reused within a block
/// until the reference is updated. This is a local pass, and does not reuse
/// loads across basic blocks.
///
/// Also allows capturing of variables by inserting loads and stores
/// before and after function calls.
///
/// This should be the first pass run (before live vars, etc.)
pub fn deref_transform_pass(
    cfg: &mut Cfg,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    insert_capture_copies(cfg, data_types, variables);
    for bb in cfg.blocks.values_mut() {
        deref_transform_block(bb, data_types, variables);
    }
    // we do a, maybe not the best thing, here and let analyze also
    // replaces uses of references with the use of the original variable

    // we must do the derefernce of variable uses first
    let _ = analyze(cfg, &RefPropagation::default());
    for bb in cfg.blocks.values_mut() {
        remove_refs_ops(bb);
    }
}

/// Inserts copies of variables that are live across calls to capture their values.
/// This will insert a read-ref of a variable before the call and a var decl
/// after the call to store the value of the variable back into a mutable location.
///
/// This only applies to variables, and NOT references. This was a conscious decision
/// as we don't want to do too much magic. It makes sense for variables since they
/// have value semantics.
///
/// # Example
/// ```text
/// var x = 10;
/// foo(10);
/// x
/// ```
///
/// becomes
///
/// ```text
/// var x = 10;
/// let _cap_x = x;
/// foo(10);
/// var x = _cap_x;
/// x
/// ```
fn insert_capture_copies(
    cfg: &mut Cfg,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    let live_vars = analyze(cfg, &LiveVars::top());
    let mut preceded_by_call = HashSet::new();
    let mut terminated_by_call = HashSet::new();
    for bb in cfg.blocks.values() {
        if matches!(
            &bb.terminator,
            Terminator::Call(..) | Terminator::CaptureCall { .. } | Terminator::Yield(..),
        ) {
            for next in cfg.graph[&bb.id].targets() {
                preceded_by_call.insert(next);
            }
            terminated_by_call.insert(bb.id);
        }
    }
    for bb in cfg.blocks.values_mut() {
        block_capture_vars(
            bb,
            &live_vars,
            &preceded_by_call,
            &terminated_by_call,
            variables,
            data_types,
        );
    }
}

/// Inserts copies of variables that are live across calls to capture their values.
/// This will insert a read-ref of a variable before the call and a var decl
/// after the call to store the value of the variable back into a mutable location.
fn block_capture_vars(
    bb: &mut BasicBlock,
    live_vars: &InOutFacts<LiveVars>,
    preceded_by_call: &HashSet<usize>,
    terminated_by_call: &HashSet<usize>,
    variables: &HashSet<String>,
    data_types: &mut HashMap<String, DataType>,
) {
    if preceded_by_call.contains(&bb.id) {
        let (last_in_annotation_idx, last_in_annotation_info) = bb
            .stmts
            .iter()
            .enumerate()
            .find_map(|(idx, stmt)| match stmt {
                HirBody::InAnnotation(..) => None,
                s => Some((idx, s.get_info())),
            })
            .unwrap_or((0, bb.src_loc));
        for name in &live_vars.get_in_fact(bb.id).live_set {
            if variables.contains(name) {
                bb.stmts.insert(
                    last_in_annotation_idx,
                    HirBody::VarDecl {
                        lhs: name.to_string(),
                        lhs_tag: TripleTag::new_unspecified(),
                        rhs: Some(SchedTerm::Var {
                            name: format!("_cap_{name}"),
                            info: last_in_annotation_info,
                            tag: None,
                        }),
                        info: last_in_annotation_info,
                    },
                );
            }
        }
    }
    if terminated_by_call.contains(&bb.id) {
        let info = bb.terminator.get_info();
        for name in &live_vars.get_out_fact(bb.id).live_set {
            if variables.contains(name) {
                bb.stmts.push(HirBody::ConstDecl {
                    lhs: format!("_cap_{name}"),
                    lhs_tag: TripleTag::new_unspecified(),
                    rhs: SchedTerm::Var {
                        name: name.clone(),
                        info,
                        tag: None,
                    },
                    info,
                });
                data_types.insert(format!("_cap_{name}"), unref_type(&data_types[name]));
            }
        }
    }
}

/// Reference propagation
/// Essentially performs constant propagation for references
/// by replacing all uses of
/// references with the original variable name (the reference of the variable,
/// not the value of the variable)
///
/// Assumes that the input cfg is in an SSA-ish form where all variables
/// have a single assignment except for reference writes. Also assumes that
/// the transformation inserting dereferences on variable uses has already
/// been run.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct RefPropagation {
    aliases: HashMap<String, String>,
}

impl Fact for RefPropagation {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.aliases {
            assert!(!self.aliases.contains_key(k) || self.aliases[k] == *v);
            self.aliases.insert(k.clone(), v.clone());
        }
        self
    }

    fn transfer_instr(&mut self, mut stmt: HirInstr<'_>, _: usize) {
        // assume single assignment
        stmt.rename_uses(&mut |name, _| {
            self.aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.to_string())
        });
        if let HirInstr::Stmt(HirBody::Op {
            op: HirOp::Unary(Uop::Ref),
            dests,
            args,
            ..
        }) = stmt
        {
            assert!(args.len() == 1);
            assert_eq!(dests.len(), 1);
            let src = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
            self.aliases.insert(dests[0].0.clone(), src.clone());
        }
    }

    type Dir = Forwards;
}

/// Removes all unary reference operators from a basic block
/// This should be run after reference propagation, which renders
/// reference operators useless.
fn remove_refs_ops(bb: &mut BasicBlock) {
    let mut to_remove = Vec::new();
    for (idx, instr) in bb.stmts.iter().enumerate() {
        if let HirBody::Op {
            op: HirOp::Unary(Uop::Ref),
            ..
        } = instr
        {
            to_remove.push(idx);
        }
    }
    for idx in to_remove.into_iter().rev() {
        bb.stmts.remove(idx);
    }
}

/// Transforms uses of references into uses of values by inserting deref instructions
/// (loads) for a single block.
fn deref_transform_block(
    bb: &mut BasicBlock,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    let mut insertions = Vec::new();
    let mut last_deref = HashMap::new();
    let mut names = HashMap::new();
    for (id, instr) in bb
        .stmts
        .iter_mut()
        .map(HirInstr::Stmt)
        .chain(std::iter::once(HirInstr::Tail(&mut bb.terminator)))
        .enumerate()
    {
        deref_transform_instr(
            id,
            instr,
            &mut names,
            &mut insertions,
            &mut last_deref,
            data_types,
            variables,
        );
    }

    for (insert_id, hir) in insertions.into_iter().rev() {
        bb.stmts.insert(insert_id, hir);
    }
}

/// Gets the current name for a non-reference version of a reference variable.
/// If `names` does not contain `name`, then `name` is inserted into `names` with
/// the current name version of 0
fn get_cur_name(name: &str, names: &mut HashMap<String, u16>) -> String {
    if let Some(i) = names.get(name) {
        return format!("_{i}{name}");
    }
    names.insert(name.to_string(), 0);
    format!("_0{name}")
}

/// Converts an asm `TypeId` that's a reference to the corresponding non-reference
/// ast `DataType`.
fn unref_type(typ: &DataType) -> DataType {
    match typ {
        DataType::Ref(t) => *t.clone(),
        x => x.clone(),
    }
}

/// Insert a deref instruction if needed. We need a deref instruction if the
/// reference has been updated since the last deref instruction.
/// # Arguments
/// * `last_deref` - the last derefed version for each variable
/// * `names` - the current name versions for each variable
/// * `types` - the types of each variable
/// * `insertions` - a list of insertions to make to the basic block. A list of
/// tuples of the insertion index (wrt the unmodified list of instructions)
/// and the instruction to insert.
/// * `id` - the basic block id
/// * `name` - the name of the variable to insert a deref instruction for
fn insert_deref_if_needed(
    last_deref: &mut HashMap<String, u16>,
    names: &mut HashMap<String, u16>,
    insertions: &mut Vec<(usize, HirBody)>,
    id: usize,
    name: &str,
    data_types: &mut HashMap<String, DataType>,
    info: Info,
) {
    if last_deref.get(name).is_none() || last_deref[name] != names[name] {
        let typ = unref_type(&data_types[name]);
        let dest = get_cur_name(name, names);
        data_types.insert(dest.clone(), typ.clone());
        insertions.push((
            id,
            HirBody::RefLoad {
                info,
                dest,
                src: format!("_{name}_ref"),
                typ,
            },
        ));
        last_deref.insert(name.to_string(), names[name]);
    }
}

/// Renames instructrs so that references have a `_ref` suffix and uses of the values
/// stored in the reference have unique node names.
/// # Arguments
/// * `id` - the basic block id
/// * `instr` - the instruction to transform
/// * `names` - the current name versions for each variable
/// * `types` - the types of each variable
/// * `insertions` - a list of insertions to make to the basic block. A list of
/// tuples of the insertion index (wrt the unmodified list of instructions)
/// and the instruction to insert.
/// * `last_deref` - the last derefed version for each variable
#[allow(clippy::too_many_arguments)]
fn deref_transform_instr(
    id: usize,
    instr: HirInstr,
    names: &mut HashMap<String, u16>,
    insertions: &mut Vec<(usize, HirBody)>,
    last_deref: &mut HashMap<String, u16>,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    match instr {
        // TODO: generalize terminator usage
        HirInstr::Tail(t) => {
            let info = t.get_info();
            t.rename_uses(&mut |u, ut| {
                if variables.contains(u) && ut == UseType::Read {
                    insert_deref_if_needed(last_deref, names, insertions, id, u, data_types, info);
                    get_cur_name(u, names)
                } else {
                    u.to_string()
                }
            });
        }
        HirInstr::Stmt(HirBody::RefLoad { src, .. }) => {
            if variables.contains(src) {
                *src = format!("_{src}_ref");
            }
        }
        HirInstr::Stmt(
            HirBody::InAnnotation(_, annotations) | HirBody::OutAnnotation(_, annotations),
        ) => {
            for (name, _) in annotations {
                if variables.contains(name) {
                    *name = format!("_{name}_ref");
                }
            }
        }
        HirInstr::Stmt(HirBody::Op {
            op: HirOp::Unary(Uop::Ref),
            args,
            ..
        }) => {
            for arg in args {
                if let SchedTerm::Var { name, .. } = arg {
                    if variables.contains(name) {
                        *name = format!("_{name}_ref");
                    }
                }
            }
        }
        HirInstr::Stmt(stmt) => {
            let info = stmt.get_info();
            stmt.rename_uses(&mut |name, ut| {
                if variables.contains(name) && ut == UseType::Read {
                    insert_deref_if_needed(
                        last_deref, names, insertions, id, name, data_types, info,
                    );
                    get_cur_name(name, names)
                } else {
                    name.to_string()
                }
            });
            if let HirBody::VarDecl { lhs, .. } | HirBody::RefStore { lhs, .. } = stmt {
                match names.entry(lhs.clone()) {
                    Entry::Occupied(mut entry) => {
                        *entry.get_mut() += 1;
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(0);
                    }
                }
            }
            match stmt {
                HirBody::VarDecl { lhs, .. } => {
                    let old_lhs = lhs.clone();
                    // rename the lhs to the reference version
                    *lhs = format!("_{lhs}_ref");
                    data_types.insert(lhs.clone(), data_types[&old_lhs].clone());
                }
                HirBody::RefStore { lhs, .. } => {
                    if variables.contains(lhs) {
                        *lhs = format!("_{lhs}_ref");
                    }
                }
                _ => (),
            }
        }
    }
}
