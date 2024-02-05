use std::collections::{hash_map::Entry, HashMap, HashSet};

use caiman::assembly::ast as asm;
use caiman::assembly::ast::TypeId;

use crate::{
    enum_cast,
    lower::sched_hir::{
        cfg::{BasicBlock, Cfg},
        make_deref, Hir, HirBody, HirInstr, HirOp, UseType,
    },
    parse::ast::{DataType, IntSize, SchedTerm, Uop},
};

use super::{analyze, Fact, Forwards};

/// Transforms uses of variables into uses of values by inserting deref instructions.
/// After this pass, all deref and ref operators will be removed and replaced with
/// deref instructions when needed.
pub fn deref_transform_pass(
    cfg: &mut Cfg,
    types: &mut HashMap<String, TypeId>,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    for bb in cfg.blocks.values_mut() {
        deref_transform_block(bb, types, data_types, variables);
    }
    // we do a, maybe not the best thing, here and let analyze also
    // replaces uses of references with the use of the original variable

    // we must do the derefernce of variable uses first
    let _ = analyze(cfg, &RefPropagation::default());
    for bb in cfg.blocks.values_mut() {
        remove_refs_ops(bb);
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct RefPropagation {
    aliases: HashMap<String, String>,
}

impl Fact for RefPropagation {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.aliases {
            assert!(!self.aliases.contains_key(k));
            self.aliases.insert(k.clone(), v.clone());
        }
        self
    }

    fn transfer_instr(&mut self, mut stmt: HirInstr<'_>, _: usize) {
        // assume single assignment
        stmt.rename_uses(&mut |name, ut| {
            if ut == UseType::Read {
                self.aliases
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.to_string())
            } else {
                name.to_string()
            }
        });
        if let HirInstr::Stmt(HirBody::Op {
            op: HirOp::Unary(Uop::Ref),
            dest,
            args,
            ..
        }) = stmt
        {
            assert!(args.len() == 1);
            let src = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
            self.aliases.insert(dest.clone(), src.clone());
        }
    }

    type Dir = Forwards;
}

/// Removes all unary reference operators from a basic block
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
/// for a single block.
fn deref_transform_block(
    bb: &mut BasicBlock,
    types: &mut HashMap<String, TypeId>,
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
            types,
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
fn unref_type(typ: &asm::TypeId) -> DataType {
    match typ {
        asm::TypeId::Local(name) => match &name[1..] {
            "bool" => DataType::Bool,
            "i32" => DataType::Int(IntSize::I32),
            "i64" => DataType::Int(IntSize::I64),
            x => panic!("Unrecognized type: {x}"),
        },
        asm::TypeId::FFI(_) => todo!(),
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
    types: &mut HashMap<String, TypeId>,
    insertions: &mut Vec<(usize, HirBody)>,
    id: usize,
    name: &str,
    data_types: &mut HashMap<String, DataType>,
) {
    if last_deref.get(name).is_none() || last_deref[name] != names[name] {
        let typ = unref_type(&types[name]);
        let dest = get_cur_name(name, names);
        types.insert(dest.clone(), make_deref(&types[name]));
        data_types.insert(dest.clone(), typ.clone());
        insertions.push((
            id,
            HirBody::RefLoad {
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
    types: &mut HashMap<String, TypeId>,
    insertions: &mut Vec<(usize, HirBody)>,
    last_deref: &mut HashMap<String, u16>,
    data_types: &mut HashMap<String, DataType>,
    variables: &HashSet<String>,
) {
    match instr {
        // TODO: generalize terminator usage
        HirInstr::Tail(t) => {
            // TODO: return references
            t.rename_uses(&mut |u, ut| {
                if variables.contains(u) && ut == UseType::Read {
                    insert_deref_if_needed(last_deref, names, types, insertions, id, u, data_types);
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
            stmt.rename_uses(&mut |name, ut| {
                if variables.contains(name) && ut == UseType::Read {
                    insert_deref_if_needed(
                        last_deref, names, types, insertions, id, name, data_types,
                    );
                    get_cur_name(name, names)
                } else if ut == UseType::Write {
                    // writes can only occur to references, so we need to rename
                    // the variable to the reference version
                    format!("_{name}_ref")
                } else {
                    name.to_string()
                }
            });
            if let HirBody::VarDecl { lhs, .. } = stmt {
                let old_lhs = lhs.clone();
                // rename the lhs to the reference version
                *lhs = format!("_{lhs}_ref");
                types.insert(lhs.clone(), types[&old_lhs].clone());
                data_types.insert(lhs.clone(), data_types[&old_lhs].clone());
            }
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
        }
    }
}
