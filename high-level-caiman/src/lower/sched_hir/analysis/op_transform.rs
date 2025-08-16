use std::collections::HashMap;

use crate::{
    enum_cast,
    lower::{
        binop_name,
        sched_hir::{
            cfg::{BasicBlock, Cfg},
            FillIn, HirBody, HirOp, HirTerm,
        },
        uop_name,
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
            op: HirOp::Unary(FillIn::Initial(Uop::Deref)),
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
                bin.process(|bop| {
                    (
                        String::from(binop_name(*bop)),
                        vec![
                            arg_l
                                .opt()
                                .and_then(|x| data_types.get(x).map(|dt| format!("{dt}"))),
                            arg_r
                                .opt()
                                .and_then(|x| data_types.get(x).map(|dt| format!("{dt}"))),
                        ],
                    )
                });
            }
            HirOp::Unary(unary @ FillIn::Initial(Uop::Neg | Uop::Not | Uop::LNot)) => {
                assert_eq!(args.len(), 1);
                let arg = args[0].hole_or_var().unwrap();
                unary.process(|uop| {
                    (
                        String::from(uop_name(*uop)),
                        arg.opt()
                            .and_then(|x| data_types.get(x).map(|dt| format!("{dt}"))),
                    )
                });
            }
            HirOp::Unary(FillIn::Initial(Uop::Ref)) | HirOp::External(_) => (),
            HirOp::Unary(FillIn::Initial(Uop::Deref)) => panic!("Unexpected deref op"),
            HirOp::Unary(FillIn::Processed(_)) => {
                panic!("Unexpected transformed op")
            }
        },
        _ => {}
    }
}
