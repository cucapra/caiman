use std::collections::HashMap;

use crate::{
    enum_cast,
    lower::{
        binop_to_str,
        sched_hir::{
            cfg::{BasicBlock, Cfg},
            HirBody, HirOp, OpType,
        },
        uop_to_str,
    },
    parse::ast::{DataType, SchedTerm, Uop},
};

/// Transforms binary and unary operations into external FFI calls.
/// Also replaces dereferences with `ref_load` instructions.
/// After this pass, all binary and unary operators, except referenceS
/// will be replaced with external FFI calls or loads.
#[allow(clippy::module_name_repetitions)]
pub fn op_transform_pass(cfg: &mut Cfg, data_types: &HashMap<String, DataType>) {
    for bb in cfg.blocks.values_mut() {
        op_transform_block(bb, data_types);
    }
}

/// Transforms binary operations into external FFI calls and dereferences into
/// `ref_load` instructions.
fn op_transform_block(bb: &mut BasicBlock, data_types: &HashMap<String, DataType>) {
    for instr in &mut bb.stmts {
        op_transform_instr(instr, data_types);
    }
}

/// Dereferences a data type. If the data type is not a reference,
/// it is returned as is.
fn deref_data_type(dt: DataType) -> DataType {
    match dt {
        DataType::Ref(t) => *t,
        x => x,
    }
}

/// Transforms an instruction by replacing binary operations with external FFI calls.
/// Also replaces dereferences with `ref_load` instructions.
fn op_transform_instr(instr: &mut HirBody, data_types: &HashMap<String, DataType>) {
    match instr {
        HirBody::Op {
            op: HirOp::Unary(Uop::Deref),
            dest,
            args,
            ..
        } => {
            assert_eq!(args.len(), 1);
            let src = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
            *instr = HirBody::RefLoad {
                dest: dest.clone(),
                src: src.clone(),
                typ: deref_data_type(data_types[src].clone()),
            }
        }
        HirBody::Op { op, args, .. } => match op {
            HirOp::Binary(bin) => {
                assert_eq!(args.len(), 2);
                let arg_l = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
                let arg_r = enum_cast!(SchedTerm::Var { name, .. }, name, &args[1]);
                *op = HirOp::FFI(
                    binop_to_str(
                        *bin,
                        &format!("{}", data_types[arg_l]),
                        &format!("{}", data_types[arg_r]),
                    ),
                    OpType::Binary,
                );
            }
            HirOp::Unary(unary @ (Uop::Neg | Uop::Not | Uop::LNot)) => {
                assert_eq!(args.len(), 1);
                let arg = enum_cast!(SchedTerm::Var { name, .. }, name, &args[0]);
                *op = HirOp::FFI(
                    uop_to_str(*unary, &format!("{}", data_types[arg])),
                    OpType::Unary,
                );
            }
            HirOp::Unary(Uop::Ref) | HirOp::FFI(_, OpType::External) => (),
            HirOp::Unary(Uop::Deref) => panic!("Unexpected deref op"),
            HirOp::FFI(_, _) => panic!("Unexpected transformed op"),
        },
        _ => {}
    }
}
