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

use std::collections::{hash_map::Entry, HashMap};

use caiman::assembly::ast as asm;
use caiman::assembly::ast::TypeId;

use crate::{
    lower::sched_hir::{
        cfg::{BasicBlock, Cfg},
        make_deref, Hir, HirBody, HirInstr, UseType,
    },
    parse::ast::{DataType, IntSize},
};

/// Determines if the given variable has a reference type.
fn is_ref_type(name: &str, types: &HashMap<String, TypeId>) -> bool {
    use crate::lower::sched_hir::is_ref;
    types.get(name).map_or(false, is_ref)
}

/// Transforms uses of references into uses of values by inserting
/// deref instructions (loads). A loaded value is reused within a block
/// until the reference is updated. This is a local pass, and does not reuse
/// loads across basic blocks.
///
/// This should be the first pass run (before live vars, etc.)
pub fn deref_transform_pass(
    cfg: &mut Cfg,
    types: &mut HashMap<String, TypeId>,
    data_types: &mut HashMap<String, DataType>,
) {
    for bb in cfg.blocks.values_mut() {
        deref_transform_block(bb, types, data_types);
    }
}

/// Transforms uses of references into uses of values by inserting deref instructions
/// (loads) for a single block.
fn deref_transform_block(
    bb: &mut BasicBlock,
    types: &mut HashMap<String, TypeId>,
    data_types: &mut HashMap<String, DataType>,
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
fn deref_transform_instr(
    id: usize,
    instr: HirInstr,
    names: &mut HashMap<String, u16>,
    types: &mut HashMap<String, TypeId>,
    insertions: &mut Vec<(usize, HirBody)>,
    last_deref: &mut HashMap<String, u16>,
    data_types: &mut HashMap<String, DataType>,
) {
    match instr {
        // TODO: generalize terminator usage
        HirInstr::Tail(t) => {
            // TODO: return references
            t.rename_uses(&mut |u, ut| {
                if is_ref_type(u, types) && ut == UseType::Read {
                    insert_deref_if_needed(last_deref, names, types, insertions, id, u, data_types);
                    get_cur_name(u, names)
                } else {
                    u.to_string()
                }
            });
        }
        HirInstr::Stmt(HirBody::RefLoad { .. }) => panic!("Already inserted deref"),
        HirInstr::Stmt(
            HirBody::InAnnotation(_, annotations) | HirBody::OutAnnotation(_, annotations),
        ) => {
            for (name, _) in annotations {
                if is_ref_type(name, types) {
                    *name = format!("_{name}_ref");
                }
            }
        }
        HirInstr::Stmt(stmt) => {
            stmt.rename_uses(&mut |name, ut| {
                if is_ref_type(name, types) && ut == UseType::Read {
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
