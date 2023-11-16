use std::collections::HashMap;

use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{ClassMembers, TopLevel},
};
use caiman::assembly::ast as asm;

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
            name: String::from("i64"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(asm::FFIType::I64),
            },
        })),
    ]
}

/// Creates a global context from a list of top-level declarations.
pub fn gen_context(tl: &[TopLevel]) -> Context {
    let mut ctx = Context {
        specs: HashMap::new(),
        type_decls: gen_type_decls(tl),
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
