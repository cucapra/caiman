use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{
    enum_cast,
    error::{HasInfo, Info},
    parse::ast::{
        DataType, FlaggedType, FullType, SchedExpr, SchedFuncCall, SchedStmt, SchedTerm, SpecType,
        Tag, WGPUFlags, WGPUSettings,
    },
    typing::{Context, SchedOrExtern},
};

/// Pushes the fields of a record onto the end of a list of input or output arguments.
#[allow(clippy::too_many_arguments)]
fn insert_record_args(
    new_inputs: &mut Vec<(String, Option<FullType>)>,
    record_types: &BTreeMap<String, DataType>,
    read: &HashSet<String>,
    write: &HashSet<String>,
    flags: &BTreeSet<WGPUFlags>,
    settings: &BTreeSet<WGPUSettings>,
    tags: &[Tag],
    info: Info,
) {
    for (field_nm, field_typ) in record_types {
        let mut flags = flags.clone();
        if read.contains(field_nm) {
            flags.insert(WGPUFlags::MapRead);
        }
        if write.contains(field_nm) {
            flags.insert(WGPUFlags::CopyDst);
        }
        flags.insert(WGPUFlags::Storage);
        new_inputs.push((
            // TODO: renaming
            field_nm.clone(),
            Some(FullType {
                base: Some(FlaggedType {
                    base: field_typ.clone(),
                    flags,
                    info,
                    settings: settings.clone(),
                }),
                tags: tags
                    .iter()
                    .filter(|t| t.quot_var.spec_type == SpecType::Timeline)
                    .cloned()
                    .collect(),
            }),
        ));
    }
}

/// Replaces the input or output types of a function to include all fields of any
/// record crossing the function boundary.
fn replace_io<T: IntoIterator<Item = (String, Option<FullType>)>>(
    input: T,
) -> Vec<(String, Option<FullType>)> {
    // TODO: fences
    let mut new_inputs = Vec::new();
    for (input_nm, input_typ) in input {
        match input_typ {
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base:
                            ref b @ (DataType::Fence(Some(ref t)) | DataType::Encoder(Some(ref t))),
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => {
                if let DataType::RemoteObj { all, read, write } = &**t {
                    new_inputs.push((
                        input_nm,
                        Some(FullType {
                            base: Some(FlaggedType {
                                base: b.clone(),
                                flags: flags.clone(),
                                info,
                                settings: settings.clone(),
                            }),
                            tags: tags.clone(),
                        }),
                    ));
                    insert_record_args(
                        &mut new_inputs,
                        all,
                        read,
                        write,
                        &flags,
                        &settings,
                        &tags,
                        info,
                    );
                } else {
                    panic!("Unexpected inner type of fence/encoder");
                }
            }
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base: DataType::RemoteObj { all, read, write },
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => insert_record_args(
                &mut new_inputs,
                &all,
                &read,
                &write,
                &flags,
                &settings,
                &tags,
                info,
            ),
            _ => {
                new_inputs.push((input_nm, input_typ));
            }
        }
    }
    new_inputs
}

/// Replaces arguments of a function call to include all fields of records when
/// a record is passed directly or through an encoder or fence.
fn replace_fn_call(call: &mut SchedFuncCall, ctx: &Context) {
    let target_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, &*call.target)
    );
    if let SchedOrExtern::Sched(callee_info) = ctx.scheds.get(target_name).unwrap() {
        let mut new_args = Vec::new();
        let args = std::mem::take(&mut call.args);
        for (idx, arg) in args.into_iter().enumerate() {
            match &callee_info.dtype_sig.input[idx] {
                FlaggedType {
                    base: DataType::RemoteObj { all, .. },
                    ..
                } => {
                    let info = arg.info();
                    for field_name in all.keys() {
                        new_args.push(SchedExpr::Term(SchedTerm::Var {
                            name: field_name.clone(),
                            info,
                            tag: None,
                        }));
                    }
                }
                FlaggedType {
                    base: DataType::Fence(Some(t)) | DataType::Encoder(Some(t)),
                    info,
                    ..
                } => {
                    new_args.push(arg);
                    if let DataType::RemoteObj { all, .. } = &**t {
                        for field_name in all.keys() {
                            new_args.push(SchedExpr::Term(SchedTerm::Var {
                                name: field_name.clone(),
                                info: *info,
                                tag: None,
                            }));
                        }
                    } else {
                        panic!("Unexpected inner type of fence/encoder");
                    }
                }
                _ => new_args.push(arg),
            }
        }
        call.args = new_args;
    }
}

fn replace_stmts<'a, T: Iterator<Item = &'a mut SchedStmt>>(stmts: T, ctx: &Context) {
    for stmt in stmts {
        match stmt {
            SchedStmt::Decl {
                expr: Some(SchedExpr::Term(SchedTerm::Call(_, call))),
                ..
            }
            | SchedStmt::Call(_, call)
            | SchedStmt::Assign {
                rhs: SchedExpr::Term(SchedTerm::Call(_, call)),
                ..
            } => {
                replace_fn_call(call, ctx);
            }
            SchedStmt::Block(_, stmts) => {
                replace_stmts(stmts.iter_mut(), ctx);
            }
            SchedStmt::Seq { block, .. } => {
                replace_stmts(std::iter::once(&mut **block), ctx);
            }
            SchedStmt::If {
                true_block,
                false_block,
                ..
            } => {
                replace_stmts(true_block.iter_mut(), ctx);
                replace_stmts(false_block.iter_mut(), ctx);
            }
            _ => (),
        }
    }
}

/// Replaces uses of records with all of their fields.
pub fn expand_records(
    input: &mut Vec<(String, Option<FullType>)>,
    output: &mut Vec<FullType>,
    statements: &mut [SchedStmt],
    ctx: &Context,
) {
    *input = replace_io(std::mem::take(input));
    *output = replace_io(
        std::mem::take(output)
            .into_iter()
            .map(|t| (String::new(), Some(t))),
    )
    .into_iter()
    .map(|(_, t)| t.unwrap())
    .collect();
    replace_stmts(statements.iter_mut(), ctx);
}
