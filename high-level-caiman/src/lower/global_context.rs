use std::collections::{HashMap, HashSet};

use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{Binop, ClassMembers, SchedExpr, SchedStmt, TopLevel},
};
use caiman::assembly::ast as asm;
use caiman::ir;

use super::binop_to_str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of a spec.
pub enum SpecType {
    Value,
    Timeline,
    Spatial,
}

/// A global context for a caiman program. This contains information about constants,
/// type aliases, and function signatures.
pub struct Context {
    pub specs: HashMap<String, SpecType>,
    pub type_decls: Vec<asm::Declaration>,
}

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

/// Generates a list of extern declarations needed for a given program.
#[allow(clippy::single_match)]
fn gen_extern_decls(tl: &[TopLevel]) -> Vec<asm::Declaration> {
    let mut res = vec![];
    let mut existing_externs = HashSet::new();
    // TODO: scan spects
    for decl in tl {
        match decl {
            TopLevel::SchedulingFunc { statements, .. } => {
                for stmt in statements {
                    match stmt {
                        SchedStmt::Decl { expr, .. } => {
                            if let Some(expr) = expr.as_ref() {
                                res.extend(get_extern_decls(expr, &mut existing_externs));
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    res
}

/// Returns a list of extern declarations needed for a given expression.
fn get_extern_decls(
    expr: &SchedExpr,
    existing_externs: &mut HashSet<Binop>,
) -> Vec<asm::Declaration> {
    match expr {
        SchedExpr::Binop { op, .. } => match op {
            Binop::Lt if !existing_externs.contains(op) => {
                existing_externs.insert(*op);
                vec![
                    asm::Declaration::FunctionClass(asm::FunctionClass {
                        name: asm::FunctionClassId(binop_to_str(*op).to_string()),
                        // TODO: generalize
                        input_types: vec![
                            asm::TypeId::FFI(asm::FFIType::I64),
                            asm::TypeId::FFI(asm::FFIType::I64),
                        ],
                        output_types: vec![asm::TypeId::FFI(BOOL_FFI_TYPE)],
                    }),
                    asm::Declaration::ExternalFunction(asm::ExternalFunction {
                        name: binop_to_str(*op).to_string(),
                        kind: asm::ExternalFunctionKind::CPUPure,
                        value_function_binding: asm::FunctionClassBinding {
                            default: false,
                            function_class: asm::FunctionClassId(binop_to_str(*op).to_string()),
                        },
                        // TODO: generalize
                        input_args: vec![
                            asm::ExternalArgument {
                                name: None,
                                ffi_type: asm::FFIType::I64,
                            },
                            asm::ExternalArgument {
                                name: None,
                                ffi_type: asm::FFIType::I64,
                            },
                        ],
                        output_types: vec![asm::ExternalArgument {
                            name: None,
                            ffi_type: BOOL_FFI_TYPE,
                        }],
                    }),
                ]
            }
            _ => todo!(),
        },
        _ => vec![],
    }
}

/// Creates a global context from a list of top-level declarations.
pub fn gen_context(tl: &[TopLevel]) -> Context {
    let mut ctx = Context {
        specs: HashMap::new(),
        type_decls: gen_type_decls(tl)
            .into_iter()
            .chain(gen_extern_decls(tl))
            .collect(),
    };
    for decl in tl {
        match decl {
            TopLevel::SpatialFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Spatial);
            }
            TopLevel::TimelineFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Timeline);
            }
            TopLevel::FunctionClass { members, .. } => {
                for m in members {
                    match m {
                        ClassMembers::ValueFunclet { name, .. } => {
                            ctx.specs.insert(name.to_string(), SpecType::Value);
                        }
                        ClassMembers::Extern { .. } => (),
                    }
                }
            }
            _ => (),
        }
    }
    ctx
}
