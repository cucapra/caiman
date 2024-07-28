use std::collections::HashMap;

use caiman::explication::Hole;

use crate::{
    enum_cast,
    lower::{
        binop_to_str,
        sched_hir::{
            cfg::{BasicBlock, Cfg},
            HirBody, HirOp, HirTerm, OpType,
        },
        uop_to_str,
    },
    parse::ast::{DataType, Uop},
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
            dests,
            args,
            info,
            ..
        } => {
            assert_eq!(args.len(), 1);
            assert_eq!(dests.len(), 1);
            let src = enum_cast!(HirTerm::Var { name, .. }, name, &args[0]);
            *instr = HirBody::RefLoad {
                info: *info,
                dest: dests[0].0.clone(),
                src: src.clone(),
                typ: deref_data_type(data_types[src].clone()),
            }
        }
        HirBody::Op { op, args, .. } => match op {
            HirOp::Binary(bin) => {
                assert_eq!(args.len(), 2);
                let arg_l = args[0].hole_or_var().unwrap();
                let arg_r = args[1].hole_or_var().unwrap();
                if let (Hole::Filled(arg_l), Hole::Filled(arg_r)) = (arg_l, arg_r) {
                    *op = HirOp::FFI(
                        Hole::Filled(binop_to_str(
                            *bin,
                            &format!("{}", data_types[arg_l]),
                            &format!("{}", data_types[arg_r]),
                        )),
                        OpType::Binary,
                    );
                } else {
                    *op = HirOp::FFI(Hole::Empty, OpType::Binary);
                }
            }
            HirOp::Unary(unary @ (Uop::Neg | Uop::Not | Uop::LNot)) => {
                assert_eq!(args.len(), 1);
                let arg = args[0].hole_or_var().unwrap();
                if let Hole::Filled(arg) = arg {
                    *op = HirOp::FFI(
                        Hole::Filled(uop_to_str(*unary, &format!("{}", data_types[arg]))),
                        OpType::Unary,
                    );
                } else {
                    *op = HirOp::FFI(Hole::Empty, OpType::Unary);
                }
            }
            HirOp::Unary(Uop::Ref) | HirOp::FFI(_, OpType::External) => (),
            HirOp::Unary(Uop::Deref) => panic!("Unexpected deref op"),
            HirOp::FFI(_, _) => panic!("Unexpected transformed op"),
        },
        _ => {}
    }
}
