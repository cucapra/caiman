use std::collections::{HashMap, HashSet};

use crate::lower::{binop_to_str, data_type_to_ffi, data_type_to_ffi_type};
use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{ClassMembers, DataType, SchedStmt, TopLevel},
};
use caiman::assembly::ast as asm;
use caiman::ir;

use super::specs::collect_spec;
use super::{Context, SpecInfo, SpecNode, SpecType, TypedBinop};

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
    ]
}

/// Collects a mapping between high-level variables and their types.
fn collect_sched_types(stmts: &Vec<SchedStmt>, types: &mut HashMap<String, DataType>) {
    for s in stmts {
        match s {
            SchedStmt::Decl { lhs, .. } => {
                for (name, tag) in lhs {
                    tag.as_ref()
                        .map(|t| types.insert(name.clone(), t.base.as_ref().unwrap().base.clone()));
                }
            }
            SchedStmt::Block(_, stmts) => {
                collect_sched_types(stmts, types);
            }
            _ => (),
        }
    }
}

/// Collects a context for top level declarations.
/// Generates a list of extern declarations needed for a given program.
/// # Arguments
/// * `tl` - the top-level declarations to scan
/// * `ctx` - the context to update with node definitions for each spec
/// # Returns
/// * A list of extern declarations needed for a given program.
/// * A map from spec names to a map from variable names to their types.
fn collect_top_level(tl: &[TopLevel], mut ctx: Context) -> Context {
    let mut existing_externs = HashSet::new();
    // TODO: do we need to scan schedules?
    for decl in tl {
        if let TopLevel::FunctionClass { members, .. } = decl {
            for m in members {
                if let ClassMembers::ValueFunclet {
                    statements,
                    input,
                    name,
                    output,
                    ..
                } = m
                {
                    let spec = ctx.specs.get_mut(name).unwrap();
                    for (name, typ) in input {
                        spec.types.insert(name.clone(), typ.clone());
                    }
                    for (name, _) in input.iter() {
                        spec.nodes
                            .insert(name.clone(), SpecNode::Input(name.clone()));
                    }
                    for name in output.iter().filter_map(|x| x.0.as_ref()) {
                        spec.nodes
                            .insert(name.clone(), SpecNode::Output(name.clone()));
                    }
                    collect_spec(
                        statements,
                        &mut existing_externs,
                        ctx.specs.get_mut(name).unwrap(),
                        &ctx.signatures,
                    );
                }
            }
        }
    }
    ctx.type_decls
        .append(&mut get_extern_decls(&existing_externs));
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
) -> Context {
    let mut member_sig = None;
    for m in members {
        match m {
            ClassMembers::ValueFunclet {
                name,
                input,
                output,
                ..
            } => {
                let sig = (
                    input.iter().map(|x| x.1.clone()).collect(),
                    output.iter().map(|x| x.1.clone()).collect(),
                );
                if let Some(member_sig) = &member_sig {
                    assert_eq!(
                        &sig, member_sig,
                        "All members of function class {class_name} must have the same signature"
                    );
                } else {
                    member_sig = Some(sig.clone());
                }
                ctx.specs
                    .insert(name.to_string(), SpecInfo::new(SpecType::Value, sig));
            }
            ClassMembers::Extern {
                name,
                input,
                output,
                ..
            } => {
                let sig = (
                    input.iter().map(|x| x.1.clone()).collect(),
                    output.iter().map(|x| x.1.clone()).collect(),
                );
                if let Some(member_sig) = &member_sig {
                    assert_eq!(
                        &sig, member_sig,
                        "All members of function class {class_name} must have the same signature"
                    );
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
    ctx
}

fn collect_type_signatures(tl: &[TopLevel], mut ctx: Context) -> Context {
    for decl in tl {
        match decl {
            TopLevel::SpatialFunclet { name, .. } => {
                ctx.specs.insert(
                    name.to_string(),
                    super::SpecInfo::new(
                        SpecType::Spatial,
                        (vec![DataType::BufferSpace], vec![DataType::BufferSpace]),
                    ),
                );
            }
            TopLevel::TimelineFunclet { name, .. } => {
                ctx.specs.insert(
                    name.to_string(),
                    super::SpecInfo::new(
                        SpecType::Timeline,
                        (vec![DataType::Event], vec![DataType::Event]),
                    ),
                );
            }
            TopLevel::FunctionClass {
                name: class_name,
                members,
                ..
            } => {
                ctx = collect_class_signatures(members, ctx, class_name);
            }
            TopLevel::SchedulingFunc {
                name,
                input,
                output,
                statements,
                ..
            } => {
                let mut types = HashMap::new();
                for (name, typ) in input {
                    types.insert(name.clone(), typ.base.as_ref().unwrap().base.clone());
                }
                collect_sched_types(statements, &mut types);
                ctx.sched_types.insert(name.to_string(), types);
                ctx.signatures.insert(
                    name.to_string(),
                    (
                        input
                            .iter()
                            .map(|x| x.1.base.as_ref().unwrap().base.clone())
                            .collect(),
                        output
                            .iter()
                            .map(|x| x.base.as_ref().unwrap().base.clone())
                            .collect(),
                    ),
                );
            }
            _ => (),
        }
    }
    ctx
}

impl Context {
    /// Creates a global context from a list of top-level declarations.
    #[must_use]
    pub fn new(tl: &[TopLevel]) -> Self {
        let ctx = Self {
            specs: HashMap::new(),
            type_decls: gen_type_decls(tl).into_iter().collect(),
            signatures: HashMap::new(),
            sched_types: HashMap::new(),
        };
        let ctx = collect_type_signatures(tl, ctx);
        collect_top_level(tl, ctx)
    }
}
