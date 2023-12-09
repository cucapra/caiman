use std::collections::{hash_map::Entry, BTreeSet, HashMap};

use caiman::assembly::ast as asm;
use caiman::assembly::ast::TypeId;

use crate::{
    lower::sched_hir::{
        cfg::{BasicBlock, Cfg},
        make_deref, term_get_uses, term_rename_uses, Hir, HirInstr, Terminator,
    },
    parse::ast::{DataType, NumberType},
};

/// Determines if the given variable has a reference type.
fn is_ref_type(name: &str, types: &HashMap<String, TypeId>) -> bool {
    use crate::lower::sched_hir::is_ref;
    types.get(name).map_or(false, is_ref)
}

/// Transforms uses of references into uses of values by inserting deref instructions.
/// This should be the first pass run (before live vars, etc.)
pub fn deref_transform_pass(cfg: &mut Cfg, types: &mut HashMap<String, TypeId>) {
    for bb in cfg.blocks.values_mut() {
        deref_transform_block(bb, types);
    }
}

/// Transforms uses of references into uses of values by inserting deref instructions
/// for a single block.
fn deref_transform_block(bb: &mut BasicBlock, types: &mut HashMap<String, TypeId>) {
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
            "i32" => DataType::Num(NumberType::I32),
            "i64" => DataType::Num(NumberType::I64),
            x => panic!("Unrecognized type: {x}"),
        },
        asm::TypeId::FFI(_) => todo!(),
    }
}

/// Insert a deref instruction if needed. We need a deref instruction if the
/// reference has been updated since the last deref instruction.
fn insert_deref_if_needed(
    last_deref: &mut HashMap<String, u16>,
    names: &mut HashMap<String, u16>,
    types: &mut HashMap<String, TypeId>,
    insertions: &mut Vec<(usize, Hir)>,
    id: usize,
    name: &str,
) {
    if last_deref.get(name).is_none() || last_deref[name] != names[name] {
        let typ = unref_type(&types[name]);
        let dest = get_cur_name(name, names);
        types.insert(dest.clone(), make_deref(&types[name]));
        insertions.push((
            id,
            Hir::RefLoad {
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
fn deref_transform_instr(
    id: usize,
    instr: HirInstr,
    names: &mut HashMap<String, u16>,
    types: &mut HashMap<String, TypeId>,
    insertions: &mut Vec<(usize, Hir)>,
    last_deref: &mut HashMap<String, u16>,
) {
    match instr {
        HirInstr::Tail(Terminator::Return(Some(name)) | Terminator::Select(name, _))
            if is_ref_type(name, types) =>
        {
            // TODO: returning references
            insert_deref_if_needed(last_deref, names, types, insertions, id, name);
            *name = get_cur_name(name, names);
        }
        HirInstr::Stmt(Hir::RefStore { lhs, rhs, .. }) => {
            let mut uses = BTreeSet::new();
            term_get_uses(rhs, &mut uses);
            for u in uses {
                if is_ref_type(&u, types) {
                    insert_deref_if_needed(last_deref, names, types, insertions, id, &u);
                }
            }
            term_rename_uses(rhs, &mut |name| {
                if is_ref_type(name, types) {
                    get_cur_name(name, names)
                } else {
                    name.to_string()
                }
            });
            let old_lhs = lhs.clone();
            *lhs = format!("_{lhs}_ref");
            types.insert(lhs.clone(), types[&old_lhs].clone());
            match names.entry(lhs.clone()) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() += 1;
                }
                Entry::Vacant(entry) => {
                    entry.insert(0);
                }
            }
        }
        HirInstr::Stmt(Hir::RefLoad { .. }) => panic!("Already inserted deref"),
        HirInstr::Stmt(Hir::InAnnotation(_, annotations) | Hir::OutAnnotation(_, annotations)) => {
            for (name, _) in annotations {
                if is_ref_type(name, types) {
                    *name = format!("_{name}_ref");
                }
            }
        }
        HirInstr::Stmt(stmt) => {
            let mut uses = BTreeSet::new();
            stmt.get_uses(&mut uses);
            for u in uses {
                if is_ref_type(&u, types) {
                    insert_deref_if_needed(last_deref, names, types, insertions, id, &u);
                }
            }
            stmt.rename_uses(&mut |name| {
                if is_ref_type(name, types) {
                    get_cur_name(name, names)
                } else {
                    name.to_string()
                }
            });
            if let Hir::VarDecl { lhs, .. } = stmt {
                *lhs = format!("_{lhs}_ref");
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
        HirInstr::Tail(Terminator::Call(..)) => todo!(),
        HirInstr::Tail(_) => (),
    }
}
