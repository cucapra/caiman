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
        uop_to_str,
    },
    parse::ast::{SchedTerm, Uop},
};

/// Transforms binary and unary operations into external FFI calls.
/// After this pass, all binary and unary operators, except references and
/// dereferences, will be replaced with external FFI calls.
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
        match op {
            HirOp::Binary(bin) => {
                assert_eq!(args.len(), 2);
                let arg_l = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
                let arg_r = enum_cast!(SchedTerm::Var { name, .. }, name, &args[1]);
                *op = HirOp::FFI(binop_to_str(
                    *bin,
                    &type_to_str(&types[arg_l]),
                    &type_to_str(&types[arg_r]),
                ));
            }
            HirOp::Unary(unary @ (Uop::Neg | Uop::Not | Uop::LNot)) => {
                assert_eq!(args.len(), 1);
                let arg = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
                *op = HirOp::FFI(uop_to_str(*unary, &type_to_str(&types[arg])));
            }
            HirOp::Unary(Uop::Deref | Uop::Ref) => (),
            HirOp::FFI(_) => panic!("Unexpected FFI op"),
        }
    }
}
