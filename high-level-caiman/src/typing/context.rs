use std::collections::{HashMap, HashSet};

use crate::error::{type_error, Info, LocalError};
use crate::lower::binop_to_str;
use crate::parse::ast::{
    ExternDef, FlaggedType, FullType, IntSize, SpecExpr, SpecFunclet, SpecStmt, SpecTerm,
};
use crate::typing::{
    ENCODE_DST_FLAGS, ENCODE_IO_FLAGS, ENCODE_SRC_FLAGS, ENCODE_STORAGE_FLAGS, LOCAL_TEMP_FLAGS,
};
use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{ClassMembers, DataType, TopLevel},
};
use caiman::assembly::ast::{self as asm};
use caiman::ir;

use super::sched::{collect_sched_names, collect_schedule};
use super::specs::collect_spec;
use super::types::DTypeConstraint;
use super::{
    sig_match, Context, DTypeEnv, Mutability, NamedSignature, SchedInfo, SchedOrExtern, Signature,
    SpecInfo, SpecType, TypedBinop,
};

/// Gets a list of type declarations for the base types used in the program.
#[allow(clippy::too_many_lines)]
fn gen_type_decls(_tl: &[TopLevel]) -> Vec<asm::Declaration> {
    // TODO: collect used types
    vec![
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("BufferSpace"),
            data: asm::LocalTypeInfo::BufferSpace,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("Event"),
            data: asm::LocalTypeInfo::Event,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("Encoder"),
            data: asm::LocalTypeInfo::Encoder {
                queue_place: ir::Place::Gpu,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("Fence"),
            data: asm::LocalTypeInfo::Fence {
                queue_place: ir::Place::Gpu,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(asm::FFIType::I64)),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(BOOL_FFI_TYPE)),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("bool"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: BOOL_FFI_TYPE,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&bool"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: BOOL_FFI_TYPE,
                storage_place: ir::Place::Local,
                buffer_flags: LOCAL_TEMP_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::FFIType::I64,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i64"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I64,
                storage_place: ir::Place::Local,
                buffer_flags: LOCAL_TEMP_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64::gs"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I64,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_SRC_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64::gd"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I64,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_DST_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64::g"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I64,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_STORAGE_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::FFIType::I32,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i32"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Local,
                buffer_flags: LOCAL_TEMP_FLAGS,
            },
        })),
        // TODO: type names
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i32::gs"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_SRC_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32::gs"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_SRC_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32::gd"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_DST_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32::g"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_STORAGE_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i32::gds"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_IO_FLAGS,
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i32::gds"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::FFIType::I32,
                storage_place: ir::Place::Gpu,
                buffer_flags: ENCODE_IO_FLAGS,
            },
        })),
    ]
}

fn get_other_decls() -> Vec<asm::Declaration> {
    vec![
        asm::Declaration::FunctionClass(asm::FunctionClass {
            name: asm::FunctionClassId(String::from("_loop")),
            input_types: vec![],
            output_types: vec![],
        }),
        asm::Declaration::ExternalFunction(asm::ExternalFunction {
            kind: asm::ExternalFunctionKind::CPUEffect,
            value_function_binding: asm::FunctionClassBinding {
                default: false,
                function_class: asm::FunctionClassId(String::from("_loop")),
            },
            name: String::from("_loop_impl"),
            input_args: vec![],
            output_types: vec![],
        }),
        asm::Declaration::Effect(asm::EffectDeclaration {
            name: asm::EffectId(String::from("_loop_eff")),
            effect: asm::Effect::FullyConnected {
                effectful_function_ids: vec![asm::ExternalFunctionId(String::from("_loop_impl"))],
            },
        }),
    ]
}

/// If the timeline spec is the identity function, it is trivial.
fn is_trivial_tmln(spec: &SpecFunclet) -> bool {
    if spec.input.len() == 1 && spec.output.len() == 1 && spec.statements.len() == 1 {
        if let SpecStmt::Returns(_, SpecExpr::Term(SpecTerm::Var { name, .. })) =
            &spec.statements[0]
        {
            return name == &spec.input[0].0;
        }
    }
    false
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
                if let ClassMembers::ValueFunclet(funclet)
                | ClassMembers::TimelineFunclet(funclet) = m
                {
                    if is_trivial_tmln(funclet) {
                        ctx.trivial_tmlns.insert(funclet.name.clone());
                    }
                    let spec = ctx.specs.get_mut(&funclet.name).unwrap();
                    for (name, typ) in &funclet.input {
                        spec.types.insert(name.clone(), typ.clone());
                    }
                    let (externs, callees) = collect_spec(
                        &funclet.statements,
                        spec,
                        &ctx.signatures,
                        &ctx.class_dimensions,
                    )?;
                    existing_externs.extend(externs);
                    ctx.called_specs.extend(callees);
                }
            }
        }
    }
    ctx.type_decls
        .append(&mut get_extern_decls(&existing_externs));
    Ok(add_ext_ops(&existing_externs, ctx))
}

/// Adds all base types to the scheduling info struct for all defined names that
/// can be resolved. There is no error if a name cannot be resolved. This is
/// bc the schedule may contain holes.
///
/// Also  performs final cleanup checks such as making sure there are no
/// references to references.
/// # Arguments
/// * `env` - the type environment to use for resolving types
/// * `types` - the map to add types to
/// * `names` - the map of names to mutabilities
/// # Returns
/// * Ok if all types are resolved
/// * Err if a post-processing check fails
fn resolve_types(
    env: &DTypeEnv,
    types: &mut HashMap<String, DataType>,
    names: &HashMap<String, Mutability>,
    flags: &mut HashMap<String, ir::BufferFlags>,
) -> Result<(), LocalError> {
    for name in names.keys() {
        if let Some(dt) = env.env.get_type(name) {
            if let Ok(dt) = DTypeConstraint::try_from(dt) {
                if let Ok(dt) = DataType::try_from(dt) {
                    if matches!(&dt, DataType::Ref(inner) if matches!(**inner, DataType::Ref(_)))
                        || (matches!(dt, DataType::Ref(_)) && names[name] == Mutability::Mut)
                    {
                        return Err(type_error(
                            Info::default(),
                            &format!(
                                "Reference to reference types are not allowed. Found: {name}: {dt}",
                            ),
                        ));
                    }
                    match &dt {
                        DataType::Fence(Some(t)) | DataType::Encoder(Some(t)) => {
                            if let DataType::RemoteObj { all, read, write } = &**t {
                                use std::collections::hash_map::Entry;
                                for readable in read {
                                    if !all.iter().any(|(f, _)| f == readable) {
                                        return Err(type_error(
                                            Info::default(),
                                            &format!(
                                                "Field {readable} is read from {name}, but {name}.{readable} is not defined",
                                            ),
                                        ));
                                    }
                                }
                                for (field, typ) in all {
                                    let final_name = format!("{name}::{field}");
                                    types.insert(final_name.clone(), typ.clone());
                                    match flags.entry(final_name) {
                                        Entry::Occupied(mut e) => {
                                            e.get_mut().storage = true;
                                            if read.contains(field) {
                                                e.get_mut().map_read = true;
                                            }
                                            if write.contains(field) {
                                                e.get_mut().copy_dst = true;
                                            }
                                        }
                                        Entry::Vacant(e) => {
                                            let mut f = ir::BufferFlags::new();
                                            f.storage = true;
                                            if read.contains(field) {
                                                f.map_read = true;
                                            }
                                            if write.contains(field) {
                                                f.copy_dst = true;
                                            }
                                            e.insert(f);
                                        }
                                    }
                                }
                            }
                        }
                        DataType::Record(fields) => {
                            for (field, typ) in fields {
                                let final_name = format!("{name}::{field}");
                                types.insert(final_name.clone(), typ.clone());
                            }
                        }
                        _ => (),
                    }
                    types.insert(name.clone(), dt);
                }
            }
        }
    }
    Ok(())
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
            for (decl_name, decl_typ) in input {
                if let Some(FullType { base: Some(dt), .. }) = decl_typ {
                    env.add_dtype_constraint(decl_name, dt.base.clone(), *info)?;
                } else {
                    panic!("All input data types should be specified for now");
                }
            }
            let num_dims = ctx.scheds[name].unwrap_sched().dtype_sig.num_dims;
            let must_be_mut = collect_schedule(&ctx, &mut env, statements, output, input, *info)?;
            let sched_info = ctx.scheds.get_mut(name).unwrap().unwrap_sched_mut();
            for (in_name, _) in input {
                sched_info
                    .defined_names
                    .insert(in_name.clone(), Mutability::Const);
            }
            for i in 0..num_dims {
                env.add_dtype_constraint(&format!("_dim{i}"), DataType::Int(IntSize::I32), *info)?;
                sched_info
                    .defined_names
                    .insert(format!("_dim{i}"), Mutability::Const);
            }
            collect_sched_names(statements.iter(), &mut sched_info.defined_names)?;
            resolve_types(
                &env,
                &mut sched_info.types,
                &sched_info.defined_names,
                &mut sched_info.flags,
            )?;
            for (var, info) in must_be_mut {
                if !matches!(sched_info.types.get(&var), Some(DataType::Ref(_)))
                    && !matches!(sched_info.defined_names.get(&var), Some(Mutability::Mut))
                {
                    return Err(type_error(
                        info,
                        &format!("Immutable variable {var} cannot be assigned to or have its reference taken",),
                    ));
                }
            }
            for (k, v) in env.flags {
                sched_info.flags.insert(k, v);
            }
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
        let sig = Signature::new(vec![op_l.clone(), op_r.clone()], vec![ret.clone()], 0);
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
                    input_types: vec![op_l.asm_type(), op_r.asm_type()],
                    output_types: vec![ret.asm_type()],
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
                            ffi_type: op_l.ffi().unwrap(),
                        },
                        asm::ExternalArgument {
                            name: None,
                            ffi_type: op_r.ffi().unwrap(),
                        },
                    ],
                    output_types: vec![asm::ExternalArgument {
                        name: None,
                        ffi_type: ret.ffi().unwrap(),
                    }],
                }),
            ]
            .into_iter(),
        );
    }
    res
}

/// Collects spec info for a given spec funclet. Returns the updated context and
/// member signature.
fn collect_spec_sig(
    mut member_sig: Option<Signature>,
    class_name: &str,
    spec: &SpecFunclet,
    spec_type: SpecType,
    mut ctx: Context,
    num_dims: usize,
) -> Result<(Context, Option<Signature>), LocalError> {
    let sig = NamedSignature::new(&spec.input, spec.output.iter(), num_dims);
    if let Some(member_sig) = &member_sig {
        if !sig_match(member_sig, &sig) {
            return Err(type_error(
                spec.info,
                &format!(
                    "Function class {class_name} has inconsistent signatures for member {}",
                    spec.name,
                ),
            ));
        }
    } else {
        member_sig = Some(From::from(&sig));
    }
    ctx.specs.insert(
        spec.name.to_string(),
        SpecInfo::new(spec_type, sig, spec.info, class_name),
    );
    Ok((ctx, member_sig))
}

fn collect_class_dimension(
    class_name: &str,
    members: &[ClassMembers],
) -> Result<usize, LocalError> {
    let mut member_dimensions = 0;
    for m in members {
        if let ClassMembers::Extern {
            name,
            info,
            def: Some(ExternDef { dimensions, .. }),
            ..
        } = m
        {
            if member_dimensions == 0 {
                member_dimensions = *dimensions;
            } else if member_dimensions != *dimensions {
                return Err(type_error(
                    *info,
                    &format!(
                        "Function class {class_name} has inconsistent dimensions for member {name}",
                    ),
                ));
            }
        }
    }
    Ok(member_dimensions)
}

fn collect_class_signatures(
    members: &[ClassMembers],
    mut ctx: Context,
    class_name: &str,
) -> Result<Context, LocalError> {
    let mut member_sig = None;
    let member_dimensions = collect_class_dimension(class_name, members)?;
    for m in members {
        match m {
            ClassMembers::TimelineFunclet(spec) => {
                let (new_ctx, new_sig) = collect_spec_sig(
                    member_sig,
                    class_name,
                    spec,
                    SpecType::Timeline,
                    ctx,
                    member_dimensions,
                )?;
                ctx = new_ctx;
                member_sig = new_sig;
            }
            ClassMembers::SpatialFunclet(spec) => {
                let (new_ctx, new_sig) = collect_spec_sig(
                    member_sig,
                    class_name,
                    spec,
                    SpecType::Spatial,
                    ctx,
                    member_dimensions,
                )?;
                ctx = new_ctx;
                member_sig = new_sig;
            }
            ClassMembers::ValueFunclet(spec) => {
                let (new_ctx, new_sig) = collect_spec_sig(
                    member_sig,
                    class_name,
                    spec,
                    SpecType::Value,
                    ctx,
                    member_dimensions,
                )?;
                ctx = new_ctx;
                member_sig = new_sig;
            }
            ClassMembers::Extern {
                name,
                input,
                output,
                info,
                ..
            } => {
                let sig = NamedSignature::new(
                    &input
                        .iter()
                        .map(|x| (x.0.clone().unwrap_or_default(), x.1.clone()))
                        .collect::<Vec<_>>(),
                    output.iter(),
                    member_dimensions,
                );
                let unnamed_sig = Signature::from(&sig);
                if let Some(member_sig) = &member_sig {
                    if member_sig != &unnamed_sig {
                        return Err(type_error(
                            *info,
                            &format!(
                                "Function class {class_name} has inconsistent signatures for member {name}",
                            ),
                        ));
                    }
                } else {
                    member_sig = Some(unnamed_sig.clone());
                }
                // extern can be called directly from a schedule or a spec
                ctx.scheds
                    .insert(name.to_string(), SchedOrExtern::Extern(unnamed_sig.clone()));
                ctx.signatures.insert(name.to_string(), unnamed_sig.clone());
                ctx.specs.insert(
                    name.to_string(),
                    SpecInfo::new(SpecType::Value, sig, *info, class_name),
                );
                ctx.externs.insert(name.to_string());
            }
        }
    }
    ctx.class_dimensions
        .insert(class_name.to_string(), member_dimensions);
    ctx.signatures.insert(
        class_name.to_string(),
        member_sig
            .unwrap_or_else(|| panic!("Function class {class_name} must have at least one member")),
    );
    Ok(ctx)
}

fn collect_type_signatures(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    for decl in tl {
        if let TopLevel::FunctionClass {
            name: class_name,
            members,
            ..
        } = decl
        {
            ctx = collect_class_signatures(members, ctx, class_name)?;
        }
    }
    Ok(ctx)
}

/// Converts schedule function inputs and outputs into a datatype signature.
/// # Errors
/// Caused if a schedule does not specify datatypes for its inputs or outputs.
fn make_signature(
    input: &[(String, Option<FullType>)],
    output: &[FullType],
    info: Info,
    num_dims: usize,
) -> Result<Signature, LocalError> {
    Ok(Signature {
        input: input
            .iter()
            .map(|x| {
                Ok(x.1
                    .as_ref()
                    .ok_or_else(|| type_error(info, "Schedule inputs require a type"))?
                    .base
                    .as_ref()
                    .ok_or_else(|| type_error(info, "Schedule inputs require a type"))?
                    .clone())
            })
            .collect::<Result<Vec<_>, _>>()?,
        output: output
            .iter()
            .map(|x| {
                Ok(x.base
                    .as_ref()
                    .ok_or_else(|| type_error(info, "Function outputs require a data type"))?
                    .clone())
            })
            .collect::<Result<Vec<_>, _>>()?,
        num_dims,
    })
}

/// Collects spec info for scheduling functions.
fn collect_sched_signatures(tl: &[TopLevel], mut ctx: Context) -> Result<Context, LocalError> {
    for s in tl {
        if let TopLevel::SchedulingFunc {
            name,
            specs,
            info,
            input,
            output,
            ..
        } = s
        {
            let num_dims = specs
                .iter()
                .find_map(|spec| {
                    if let Some(SpecInfo {
                        typ: SpecType::Value,
                        sig: NamedSignature { num_dims, .. },
                        ..
                    }) = ctx.specs.get(spec)
                    {
                        Some(*num_dims)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let sig = make_signature(input, output, *info, num_dims)?;
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
                    sig,
                    info,
                )?),
            );
        }
    }
    Ok(ctx)
}

fn collect_user_defined_types(tl: &[TopLevel]) -> HashMap<String, FlaggedType> {
    let mut res = HashMap::new();
    for decl in tl {
        if let TopLevel::Typedef { name, typ, .. } = decl {
            res.insert(name.clone(), typ.clone());
        }
    }
    res
}

impl Context {
    /// Creates a global context from a list of top-level declarations.
    /// # Errors
    /// Caused by type errors in the program.
    pub fn new(tl: &[TopLevel]) -> Result<Self, LocalError> {
        let ctx = Self {
            specs: HashMap::new(),
            type_decls: gen_type_decls(tl)
                .into_iter()
                .chain(get_other_decls())
                .collect(),
            signatures: HashMap::new(),
            scheds: HashMap::new(),
            externs: HashSet::new(),
            user_types: collect_user_defined_types(tl),
            class_dimensions: HashMap::new(),
            called_specs: HashSet::new(),
            trivial_tmlns: HashSet::new(),
        };
        let ctx = collect_type_signatures(tl, ctx)?;
        let ctx = collect_sched_signatures(tl, ctx)?;
        let ctx = type_check_spec(tl, ctx)?;
        type_check_schedules(tl, ctx)
    }
}
