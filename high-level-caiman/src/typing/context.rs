use std::collections::{HashMap, HashSet};

use crate::error::{type_error, LocalError};
use crate::lower::{binop_to_str, data_type_to_ffi, data_type_to_ffi_type};
use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{ClassMembers, DataType, TopLevel},
};
use caiman::assembly::ast as asm;
use caiman::ir;

use super::sched::{collect_sched_names, collect_schedule};
use super::specs::collect_spec;
use super::types::DTypeConstraint;
use super::{
    sig_match, Context, DTypeEnv, NamedSignature, SchedInfo, SchedOrExtern, Signature, SpecInfo,
    SpecType, TypedBinop,
};

/// Gets a list of type declarations for the base types used in the program.
fn gen_type_decls(_tl: &[TopLevel]) -> Vec<asm::Declaration> {
    // collect used types
    vec![
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("BufferSpace"),
            data: asm::LocalTypeInfo::BufferSpace,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("Event"),
            data: asm::LocalTypeInfo::Event,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(asm::FFIType::I64)),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(BOOL_FFI_TYPE)),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("bool"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(BOOL_FFI_TYPE),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&bool"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::TypeId::FFI(BOOL_FFI_TYPE),
                storage_place: ir::Place::Local,
                buffer_flags: ir::BufferFlags::new(),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(asm::FFIType::I64),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i64"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::TypeId::FFI(asm::FFIType::I64),
                storage_place: ir::Place::Local,
                buffer_flags: ir::BufferFlags::new(),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(asm::FFIType::I32),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i32"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::TypeId::FFI(asm::FFIType::I32),
                storage_place: ir::Place::Local,
                buffer_flags: ir::BufferFlags::new(),
            },
        })),
    ]
}

/// Collects a context for top level declarations.
/// Generates a list of extern declarations needed for a given program and type
/// checks the specs.
/// # Arguments
/// * `tl` - the top-level declarations to scan
/// * `ctx` - the context to update with node definitions for each spec
/// # Returns
/// * A list of extern declarations needed for a given program.
/// * A map from spec names to a map from variable names to their types.
fn type_check_spec(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    let mut existing_externs = HashSet::new();
    for decl in tl {
        if let TopLevel::FunctionClass { members, .. } = decl {
            for m in members {
                if let ClassMembers::ValueFunclet {
                    statements,
                    input,
                    name,
                    ..
                } = m
                {
                    let spec = ctx.specs.get_mut(name).unwrap();
                    for (name, typ) in input {
                        spec.types.insert(name.clone(), typ.clone());
                    }
                    // for (name, _) in input.iter() {
                    //     spec.nodes
                    //         .insert(name.clone(), SpecNode::Input(name.clone()));
                    // }
                    // for name in output.iter().filter_map(|x| x.0.as_ref()) {
                    //     spec.nodes
                    //         .insert(name.clone(), SpecNode::Output(name.clone()));
                    // }
                    existing_externs.extend(collect_spec(statements, spec, &ctx.signatures)?);
                }
            }
        }
    }
    ctx.type_decls
        .append(&mut get_extern_decls(&existing_externs));
    Ok(add_ext_ops(&existing_externs, ctx))
}

/// Adds all base types to the scheduling info struct for all defined names that
/// can be resolved. There is no error of a name cannot be resolved. This is
/// bc the schedule may contain holes.
fn resolve_types(
    env: &DTypeEnv,
    types: &mut HashMap<String, DataType>,
    names: &HashMap<String, bool>,
) {
    for name in names.keys() {
        if let Some(dt) = env.env.get_type(name) {
            if let Ok(dt) = DTypeConstraint::try_from(dt) {
                if let Ok(dt) = DataType::try_from(dt) {
                    types.insert(name.clone(), dt);
                }
            }
        }
    }
}

/// Collects type constraints for scheduling functions.
/// Should be called after spec signatures are collected.
/// # Returns
/// * Updated context
fn type_check_schedules(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    for decl in tl {
        if let TopLevel::SchedulingFunc {
            name,
            input,
            output,
            statements,
            info,
            ..
        } = decl
        {
            let mut env = DTypeEnv::new();
            let spec_name = &ctx.scheds[name].unwrap_sched().value;
            let val_sig = &ctx.specs[spec_name].sig;
            if input.len() != val_sig.input.len() {
                return Err(type_error(
                    *info,
                    &format!("Function inputs of {name} do not match the spec {spec_name}"),
                ));
            }
            for ((decl_name, decl_typ), (_, spec_typ)) in input.iter().zip(val_sig.input.iter()) {
                if let Some(dt) = decl_typ.base.as_ref() {
                    if dt.base != *spec_typ {
                        return Err(type_error(
                            *info,
                            &format!(
                                "Function input {decl_name} of {name} does not match the spec {spec_typ}",
                            ),
                        ));
                    }
                }
                env.add_dtype_constraint(decl_name, spec_typ.clone(), *info)?;
            }
            let outs = val_sig.output.clone();
            collect_schedule(&ctx, &mut env, statements, output, &outs, *info, name)?;
            let sched_info = ctx.scheds.get_mut(name).unwrap().unwrap_sched_mut();
            collect_sched_names(statements.iter(), &mut sched_info.defined_names)?;
            resolve_types(&env, &mut sched_info.types, &sched_info.defined_names);
        }
    }
    Ok(ctx)
}

/// Adds extern info for a given set of typed operators.
fn add_ext_ops(externs: &HashSet<TypedBinop>, mut ctx: Context) -> Context {
    for TypedBinop {
        op,
        op_l,
        op_r,
        ret,
    } in externs
    {
        let op_name = binop_to_str(*op, &format!("{op_l:#}"), &format!("{op_r:#}")).to_string();
        let sig = Signature {
            input: vec![op_l.clone(), op_r.clone()],
            output: vec![ret.clone()],
        };
        ctx.signatures.insert(op_name.clone(), sig.clone());
        ctx.scheds.insert(op_name, SchedOrExtern::Extern(sig));
    }
    ctx
}

/// Returns a list of extern declarations needed for a given set of typed operators.
fn get_extern_decls(existing_externs: &HashSet<TypedBinop>) -> Vec<asm::Declaration> {
    let mut res = vec![];
    for TypedBinop {
        op,
        op_l,
        op_r,
        ret,
    } in existing_externs
    {
        let op_name = binop_to_str(*op, &format!("{op_l:#}"), &format!("{op_r:#}")).to_string();
        res.extend(
            [
                asm::Declaration::FunctionClass(asm::FunctionClass {
                    name: asm::FunctionClassId(op_name.clone()),
                    input_types: vec![data_type_to_ffi_type(op_l), data_type_to_ffi_type(op_r)],
                    output_types: vec![data_type_to_ffi_type(ret)],
                }),
                asm::Declaration::ExternalFunction(asm::ExternalFunction {
                    name: op_name.clone(),
                    kind: asm::ExternalFunctionKind::CPUPure,
                    value_function_binding: asm::FunctionClassBinding {
                        default: false,
                        function_class: asm::FunctionClassId(op_name.clone()),
                    },
                    input_args: vec![
                        asm::ExternalArgument {
                            name: None,
                            ffi_type: data_type_to_ffi(op_l).unwrap(),
                        },
                        asm::ExternalArgument {
                            name: None,
                            ffi_type: data_type_to_ffi(op_r).unwrap(),
                        },
                    ],
                    output_types: vec![asm::ExternalArgument {
                        name: None,
                        ffi_type: data_type_to_ffi(ret).unwrap(),
                    }],
                }),
            ]
            .into_iter(),
        );
    }
    res
}

fn collect_class_signatures(
    members: &[ClassMembers],
    mut ctx: Context,
    class_name: &str,
) -> Result<Context, LocalError> {
    let mut member_sig = None;
    for m in members {
        match m {
            ClassMembers::ValueFunclet {
                name,
                input,
                output,
                info,
                ..
            } => {
                let sig = NamedSignature {
                    input: input.clone(),
                    output: output.iter().map(|x| x.1.clone()).collect::<Vec<_>>(),
                };
                if let Some(member_sig) = &member_sig {
                    if !sig_match(member_sig, &sig) {
                        return Err(type_error(
                            *info,
                            &format!(
                                "Function class {class_name} has inconsistent signatures for member {name}",
                            ),
                        ));
                    }
                } else {
                    member_sig = Some(From::from(&sig));
                }
                ctx.specs
                    .insert(name.to_string(), SpecInfo::new(SpecType::Value, sig, *info));
            }
            ClassMembers::Extern {
                name,
                input,
                output,
                info,
                ..
            } => {
                let sig = Signature {
                    input: input.iter().map(|x| x.1.clone()).collect::<Vec<_>>(),
                    output: output.iter().map(|x| x.1.clone()).collect::<Vec<_>>(),
                };
                if let Some(member_sig) = &member_sig {
                    if member_sig != &sig {
                        return Err(type_error(
                            *info,
                            &format!(
                                "Function class {class_name} has inconsistent signatures for member {name}",
                            ),
                        ));
                    }
                } else {
                    member_sig = Some(sig.clone());
                }
                ctx.signatures.insert(name.to_string(), sig);
            }
        }
    }
    ctx.signatures.insert(
        class_name.to_string(),
        member_sig
            .unwrap_or_else(|| panic!("Function class {class_name} must have at least one member")),
    );
    Ok(ctx)
}

fn collect_type_signatures(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    for decl in tl {
        match decl {
            TopLevel::SpatialFunclet { name, info, .. } => {
                ctx.specs.insert(
                    name.to_string(),
                    super::SpecInfo::new(
                        SpecType::Spatial,
                        NamedSignature {
                            input: vec![(String::from("bs"), DataType::BufferSpace)],
                            output: vec![DataType::BufferSpace],
                        },
                        *info,
                    ),
                );
            }
            TopLevel::TimelineFunclet { name, info, .. } => {
                ctx.specs.insert(
                    name.to_string(),
                    super::SpecInfo::new(
                        SpecType::Timeline,
                        NamedSignature {
                            input: vec![(String::from("e"), DataType::Event)],
                            output: vec![DataType::Event],
                        },
                        *info,
                    ),
                );
            }
            TopLevel::FunctionClass {
                name: class_name,
                members,
                ..
            } => {
                ctx = collect_class_signatures(members, ctx, class_name)?;
            }
            _ => (),
        }
    }
    Ok(ctx)
}

/// Collects spec info for scheduling functions.
fn collect_sched_signatures(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    for s in tl {
        if let TopLevel::SchedulingFunc {
            name,
            input,
            specs,
            info,
            ..
        } = s
        {
            let mut types = HashMap::new();
            for (name, typ) in input {
                types.insert(name.clone(), typ.base.as_ref().unwrap().base.clone());
            }
            ctx.scheds.insert(
                name.to_string(),
                SchedOrExtern::Sched(SchedInfo::new(
                    specs.clone().try_into().map_err(|_| {
                        type_error(
                            *info,
                            &format!(
                                "{info}: Scheduling function {name} must have exactly 3 specs"
                            ),
                        )
                    })?,
                    &ctx,
                    info,
                )?),
            );
            ctx.scheds.get_mut(name).unwrap().unwrap_sched_mut().types = types;
        }
    }
    Ok(ctx)
}

impl Context {
    /// Creates a global context from a list of top-level declarations.
    /// # Errors
    /// Caused by type errors in the program.
    pub fn new(tl: &[TopLevel]) -> Result<Self, LocalError> {
        let ctx = Self {
            specs: HashMap::new(),
            type_decls: gen_type_decls(tl).into_iter().collect(),
            signatures: HashMap::new(),
            scheds: HashMap::new(),
        };
        let ctx = collect_type_signatures(tl, ctx)?;
        let ctx = collect_sched_signatures(tl, ctx)?;
        let ctx = type_check_spec(tl, ctx)?;
        type_check_schedules(tl, ctx)
    }
}
