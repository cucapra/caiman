use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{
    enum_cast,
    error::{HasInfo, Info},
    parse::ast::{
        Binop, DataType, EncodedCommand, FlaggedType, FullType, SchedExpr, SchedFuncCall,
        SchedLiteral, SchedStmt, SchedTerm, SpecType, Tag, TimelineOperation, WGPUFlags,
        WGPUSettings,
    },
    typing::{Context, SchedOrExtern},
};

/// Pushes the fields of a record onto the end of a list of input or output arguments.
/// Prefixes the field name with `arg_prefix::`.
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
    arg_prefix: &str,
    remote: bool,
) {
    for (field_nm, field_typ) in record_types {
        let mut flags = flags.clone();
        if remote {
            if read.contains(field_nm) {
                flags.insert(WGPUFlags::MapRead);
            }
            if write.contains(field_nm) {
                flags.insert(WGPUFlags::CopyDst);
            }
            flags.insert(WGPUFlags::Storage);
        }
        new_inputs.push((
            format!("{arg_prefix}::{field_nm}"),
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
///
/// Renames the fields of the records to be `record_name::field`.
fn replace_io<T: IntoIterator<Item = (String, Option<FullType>)>>(
    input: T,
) -> Vec<(String, Option<FullType>)> {
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
                        input_nm.clone(),
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
                        &input_nm,
                        true,
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
                &input_nm,
                true,
            ),
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base: DataType::Record(types),
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => insert_record_args(
                &mut new_inputs,
                &types,
                &HashSet::new(),
                &HashSet::new(),
                &flags,
                &settings,
                &tags,
                info,
                &input_nm,
                false,
            ),
            _ => {
                new_inputs.push((input_nm, input_typ));
            }
        }
    }
    new_inputs
}

/// Expands out records in the argument list and returns a new vector
/// where each record is replaced with all of its fields.
///
/// Requires that `args` contain all variables.
fn replace_arg_rets<
    'a,
    T: Iterator<Item = (SchedExpr, Option<FullType>)>,
    I: Iterator<Item = &'a FlaggedType>,
>(
    args: T,
    sig: I,
    name_map: &HashMap<String, String>,
) -> Vec<(SchedExpr, Option<FullType>)> {
    let mut new_args = Vec::new();
    for ((arg, arg_t), sig) in args.zip(sig) {
        let arg_name = enum_cast!(
            SchedTerm::Var { name, .. },
            name,
            enum_cast!(SchedExpr::Term, &arg)
        )
        .clone();
        match sig {
            FlaggedType {
                base: DataType::RemoteObj { all, .. } | DataType::Record(all),
                ..
            } => {
                let info = arg.info();
                for field_name in all.keys() {
                    new_args.push((
                        SchedExpr::Term(SchedTerm::Var {
                            name: format!(
                                "{}::{field_name}",
                                name_map.get(&arg_name).unwrap_or(&arg_name)
                            ),
                            info,
                            tag: None,
                        }),
                        None,
                    ));
                }
            }
            FlaggedType {
                base: DataType::Fence(Some(t)) | DataType::Encoder(Some(t)),
                info,
                ..
            } => {
                new_args.push((arg, arg_t));
                if let DataType::RemoteObj { all, .. } = &**t {
                    for field_name in all.keys() {
                        new_args.push((
                            SchedExpr::Term(SchedTerm::Var {
                                name: format!(
                                    "{}::{field_name}",
                                    name_map.get(&arg_name).unwrap_or(&arg_name)
                                ),
                                info: *info,
                                tag: None,
                            }),
                            None,
                        ));
                    }
                } else {
                    panic!("Unexpected inner type of fence/encoder");
                }
            }
            _ => new_args.push((arg, arg_t)),
        }
    }
    new_args
}

/// Replaces arguments of a function call to pass all fields of records when
/// a record is passed directly or through an encoder or fence.
fn replace_fn_call(call: &mut SchedFuncCall, ctx: &Context, name_map: &HashMap<String, String>) {
    let target_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, &*call.target)
    );
    if let SchedOrExtern::Sched(callee_info) = ctx.scheds.get(target_name).unwrap() {
        let args = std::mem::take(&mut call.args);
        call.args = replace_arg_rets(
            args.into_iter().map(|x| (x, None)),
            callee_info.dtype_sig.input.iter(),
            name_map,
        )
        .into_iter()
        .map(|(a, _)| a)
        .collect();
    }
}

/// Replaces the lhs assignments of a call with the expanded return values of
/// the function.
fn replace_fn_rets(
    lhs: &mut Vec<(String, Option<FullType>)>,
    call: &SchedFuncCall,
    ctx: &Context,
    name_map: &HashMap<String, String>,
) {
    let target_name = enum_cast!(
        SchedTerm::Var { name, .. },
        name,
        enum_cast!(SchedExpr::Term, &*call.target)
    );
    if let SchedOrExtern::Sched(callee_info) = ctx.scheds.get(target_name).unwrap() {
        let new_lhs = std::mem::take(lhs);
        lhs.extend(
            replace_arg_rets(
                new_lhs.into_iter().map(|(x, t)| {
                    (
                        SchedExpr::Term(SchedTerm::Var {
                            name: x,
                            info: call.info,
                            tag: None,
                        }),
                        t,
                    )
                }),
                callee_info.dtype_sig.output.iter(),
                name_map,
            )
            .into_iter()
            .map(|(a, b)| {
                (
                    enum_cast!(
                        SchedTerm::Var { name, .. },
                        name,
                        enum_cast!(SchedExpr::Term, a)
                    ),
                    b,
                )
            }),
        );
    }
}

/// Replaces all statements in the iterators such that all calls to functions which
/// pass records are replaced with calls to the function with all fields of the record
/// passed as arguments.
///
/// Also, we rename all record fields to be `record_name::field` for all fields.
/// # Arguments
/// * `stmts` - The statements to replace calls in.
/// * `ctx` - The context containing the function signatures.
/// * `name_map` - A map from record/fence/encoder names to the canonical name of
/// the fence/encoder/record that holds the variables. A mapping `(r, r')` in this
/// map means that instead of the renaming prefix `r`, use the prefix `r'`.
#[allow(clippy::too_many_lines)]
fn replace_stmts<'a, T: Iterator<Item = &'a mut SchedStmt>>(
    stmts: T,
    ctx: &Context,
    name_map: &mut HashMap<String, String>,
    data_types: &HashMap<String, DataType>,
) {
    for stmt in stmts {
        match stmt {
            SchedStmt::Call(_, call)
            | SchedStmt::Assign {
                rhs: SchedExpr::Term(SchedTerm::Call(_, call)),
                ..
            } => {
                replace_fn_call(call, ctx, name_map);
            }
            SchedStmt::Decl {
                expr: Some(SchedExpr::Term(SchedTerm::Call(_, call))),
                lhs,
                ..
            } => {
                replace_fn_call(call, ctx, name_map);
                replace_fn_rets(lhs, call, ctx, name_map);
            }
            SchedStmt::Block(_, stmts) => {
                replace_stmts(stmts.iter_mut(), ctx, name_map, data_types);
            }
            SchedStmt::Seq {
                block, dests, info, ..
            } => {
                let new_lhs = std::mem::take(dests);
                let types: Vec<_> = new_lhs
                    .iter()
                    .map(|(n, _)| data_types.get(n).unwrap().clone().into())
                    .collect();
                *dests = replace_arg_rets(
                    new_lhs.into_iter().map(|(x, t)| {
                        (
                            SchedExpr::Term(SchedTerm::Var {
                                name: x,
                                info: *info,
                                tag: None,
                            }),
                            t,
                        )
                    }),
                    types.iter(),
                    name_map,
                )
                .into_iter()
                .map(|(a, b)| {
                    (
                        enum_cast!(
                            SchedTerm::Var { name, .. },
                            name,
                            enum_cast!(SchedExpr::Term, a)
                        ),
                        b,
                    )
                })
                .collect();
                replace_stmts(std::iter::once(&mut **block), ctx, name_map, data_types);
            }
            SchedStmt::If {
                true_block,
                false_block,
                ..
            } => {
                replace_stmts(true_block.iter_mut(), ctx, name_map, data_types);
                replace_stmts(false_block.iter_mut(), ctx, name_map, data_types);
            }
            SchedStmt::Encode {
                encoder, stmt, cmd, ..
            } => {
                for (dest, _) in &mut stmt.lhs {
                    *dest = format!("{encoder}::{dest}");
                }
                if *cmd == EncodedCommand::Invoke {
                    if let SchedExpr::Term(SchedTerm::Call(_, call)) = &mut stmt.rhs {
                        for arg in &mut call.args {
                            let arg_name = enum_cast!(
                                SchedTerm::Var { name, .. },
                                name,
                                enum_cast!(SchedExpr::Term, arg)
                            );
                            *arg_name = format!("{encoder}::{arg_name}");
                        }
                    } else {
                        panic!("RHS of encode call is not a call");
                    }
                }
            }
            SchedStmt::Decl {
                lhs,
                expr:
                    Some(SchedExpr::Term(SchedTerm::TimelineOperation {
                        op: TimelineOperation::Submit,
                        arg,
                        ..
                    })),
                ..
            } => {
                let arg_name = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, &**arg)
                );
                // fence points to the encoder
                name_map.insert(lhs[0].0.clone(), arg_name.clone());
            }
            SchedStmt::Decl {
                expr:
                    Some(SchedExpr::Binop {
                        op: Binop::Dot,
                        lhs,
                        rhs,
                        ..
                    }),
                ..
            } => {
                let lhs_name = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, &**lhs)
                );
                let rhs_name = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, &mut **rhs)
                );
                *rhs_name = format!("{lhs_name}::{rhs_name}");
            }
            SchedStmt::InEdgeAnnotation { tags, .. }
            | SchedStmt::OutEdgeAnnotation { tags, .. } => {
                for (name, _) in tags {
                    if name.contains("::") {
                        let (prefix, suffix) = name.split_at(name.find("::").unwrap());
                        *name = format!(
                            "{}{suffix}",
                            name_map.get(prefix).unwrap_or(&prefix.to_string())
                        );
                    }
                }
            }
            SchedStmt::Return(info, rets) => {
                let mut new_ret = None;
                match rets {
                    SchedExpr::Term(SchedTerm::Lit {
                        lit: SchedLiteral::Tuple(rets),
                        ..
                    }) => {
                        let types: Vec<_> = rets
                            .iter()
                            .map(|x| {
                                data_types[enum_cast!(
                                    SchedTerm::Var { name, .. },
                                    name,
                                    enum_cast!(SchedExpr::Term, x)
                                )]
                                .clone()
                                .into()
                            })
                            .collect();
                        let r = std::mem::take(rets);
                        *rets = replace_arg_rets(
                            r.iter().map(|x| (x.clone(), None)),
                            types.iter(),
                            name_map,
                        )
                        .into_iter()
                        .map(|(a, _)| a)
                        .collect();
                    }
                    ref r @ SchedExpr::Term(SchedTerm::Var {
                        ref name, ref tag, ..
                    }) => {
                        if matches!(
                            data_types.get(name),
                            Some(DataType::Record(_) | DataType::Encoder(_) | DataType::Fence(_))
                        ) {
                            let ft = &data_types[name].clone().into();
                            let expanded = replace_arg_rets(
                                std::iter::once(((*r).clone(), None)),
                                std::iter::once(ft),
                                name_map,
                            );
                            new_ret = Some(SchedExpr::Term(SchedTerm::Lit {
                                lit: SchedLiteral::Tuple(
                                    expanded.into_iter().map(|(a, _)| a).collect(),
                                ),
                                tag: tag.clone(),
                                info: *info,
                            }));
                        }
                    }
                    _ => (),
                }
                if let Some(new_ret) = new_ret {
                    *rets = new_ret;
                }
            }
            _ => (),
        }
    }
}

/// Replaces uses of records with all of their fields.
pub fn expand_record_io(
    input: &mut Vec<(String, Option<FullType>)>,
    output: &mut Vec<FullType>,
    statements: &mut [SchedStmt],
    data_types: &HashMap<String, DataType>,
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
    let mut rename_map = HashMap::new();
    replace_stmts(statements.iter_mut(), ctx, &mut rename_map, data_types);
}
