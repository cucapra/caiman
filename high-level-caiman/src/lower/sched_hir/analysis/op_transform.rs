use std::collections::HashMap;

use caiman::assembly::ast as asm;
use caiman::assembly::ast::TypeId;

use crate::{
    enum_cast,
    lower::{
        binop_to_str,
        sched_hir::{
            cfg::{BasicBlock, Cfg},
            HirBody, HirInstr, HirOp,
        },
    },
    parse::ast::SchedTerm,
};

/// Transforms binary operations into external FFI calls
#[allow(clippy::module_name_repetitions)]
pub fn op_transform_pass(cfg: &mut Cfg, types: &HashMap<String, TypeId>) {
    for bb in cfg.blocks.values_mut() {
        op_transform_block(bb, types);
    }
}

/// Transforms binary operations into external FFI calls
fn op_transform_block(bb: &mut BasicBlock, types: &HashMap<String, TypeId>) {
    for (_, mut instr) in bb
        .stmts
        .iter_mut()
        .map(HirInstr::Stmt)
        .chain(std::iter::once(HirInstr::Tail(&mut bb.terminator)))
        .enumerate()
    {
        op_transform_instr(&mut instr, types);
    }
}

/// Converts a type id to a string
fn type_to_str(t: &TypeId) -> String {
    match t {
        asm::TypeId::Local(s) => s.clone(),
        asm::TypeId::FFI(ffi) => match ffi {
            asm::FFIType::I32 => String::from("i32"),
            asm::FFIType::I64 => String::from("i64"),
            asm::FFIType::F32 => String::from("f32"),
            asm::FFIType::F64 => String::from("f64"),
            _ => todo!(),
        },
    }
}

/// Transforms an instruction by replacing binary operations with external FFI calls
fn op_transform_instr(instr: &mut HirInstr, types: &HashMap<String, asm::TypeId>) {
    if let HirInstr::Stmt(HirBody::Op { op, args, .. }) = instr {
        if let HirOp::Binary(bin) = op {
            assert_eq!(args.len(), 2);
            let arg_l = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
            let arg_r = enum_cast!(SchedTerm::Var { name, .. }, name, &args[1]);
            *op = HirOp::FFI(binop_to_str(
                *bin,
                &type_to_str(&types[arg_l.as_str()]),
                &type_to_str(&types[arg_r.as_str()]),
            ));
        }
    }
}
