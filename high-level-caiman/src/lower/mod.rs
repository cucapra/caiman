use crate::{
    error,
    parse::ast::{ClassMembers, DataType, NumberType, SchedulingFunc, TopLevel},
};
use caiman::assembly::ast as asm;
mod global_context;
mod lower_schedule;
mod lower_spec;
mod sched_hir;

use lower_schedule::lower_schedule;
use lower_spec::{lower_spatial_funclet, lower_timeline_funclet, lower_val_funclet};

// TODO: only i32, i64, and u64 are currently supported in the IR
// change this to u8 or i8 once we support those types
const BOOL_FFI_TYPE: asm::FFIType = asm::FFIType::I32;

/// Converts a high-level caiman data type to a caiman assembly type id.
fn data_type_to_local_type(dt: &DataType) -> asm::TypeId {
    use asm::TypeId;
    match dt {
        DataType::Bool => TypeId::Local(String::from("bool")),
        DataType::Num(NumberType::I32) => TypeId::Local(String::from("i32")),
        DataType::Num(NumberType::I64) => TypeId::Local(String::from("i64")),
        DataType::BufferSpace => TypeId::Local(String::from("BufferSpace")),
        DataType::Event => TypeId::Local(String::from("Event")),
        DataType::UserDefined(name) => TypeId::Local(name.clone()),
        _ => todo!("TODO"),
    }
}

/// For types that have FFI equivalents, convert a high-level caiman data type
/// to the caiman assembly type id for the corresponding FFI type. For types
/// that do not have FFI equivalents, this is the same as `data_type_to_local_type`.
fn data_type_to_ffi_type(dt: &DataType) -> asm::TypeId {
    use asm::TypeId;
    match dt {
        DataType::Bool => TypeId::FFI(BOOL_FFI_TYPE),
        DataType::Num(NumberType::I32) => TypeId::FFI(asm::FFIType::I32),
        DataType::Num(NumberType::I64) => TypeId::FFI(asm::FFIType::I64),
        dt => data_type_to_local_type(dt),
    }
}

/// Convert a high-level caiman data type to a caiman assembly type.
fn data_types_to_local_type(dts: &[DataType]) -> Vec<asm::TypeId> {
    dts.iter().map(data_type_to_local_type).collect()
}

#[macro_export]
macro_rules! enum_cast {
    ($p:path, $e:expr) => {
        match $e {
            $p(x) => x,
            _ => panic!("AST Not flattened!: Expected {}", stringify!($p)),
        }
    };
    ($p:pat, $r:expr, $e:expr) => {
        match $e {
            $p => $r,
            _ => panic!("AST Not flattened!: Expected {}", stringify!($p)),
        }
    };
}

/// Lower a high-level caiman program to caiman assembly.
/// Requires that the high-level caiman program is well-typed and flattened.
/// # Errors
/// Returns an error if the program is not well-typed or flattened.
/// # Panics
/// If lowering something with currently unsupported language features.
pub fn lower(hlc: Vec<TopLevel>) -> Result<asm::Program, error::LocalError> {
    // Preprocessing: (before this function)
    // 1. Match literals to literals in the spec
    // 2. Constant fold constants
    // 3. Flatten AST
    //  Flattening will do the following:
    //  * Remove all nested expressions
    //  * Convert all operators to external functions
    //  * Convert all function and expression arguments to names
    //  * Convert tuple assignments to multiple assignments
    // 4. Type deduction / type checking

    // This function: (assumes all expressions are terms)
    // Steps:
    // 1. Collect all used types into type declarations
    // 2. Lower spec funclets
    // 3. Convert scheduling functions into CFGs
    // 4. Perform live variable analysis to determine basic block inputs/outputs
    // 5. Lower basic blocks into assembly w/ CPS
    let mut asm = asm::Program {
        path: String::new(),
        version: asm::Version {
            major: 0,
            minor: 0,
            detailed: 2,
        },
        declarations: Vec::new(),
    };
    let ctx = global_context::gen_context(&hlc);
    asm.declarations.extend(ctx.type_decls.iter().cloned());
    for top in hlc {
        match top {
            TopLevel::Pipeline { name, entry, .. } => {
                let pipeline = asm::Pipeline {
                    name,
                    funclet: asm::FuncletId(entry),
                };
                asm.declarations.push(asm::Declaration::Pipeline(pipeline));
            }
            TopLevel::FunctionClass { name, members, .. } => {
                let (in_types, out_types) = members[0].get_type_signature();
                // lower funclets and fill in their funclass class bindings
                let class = asm::FunctionClass {
                    name: asm::FunctionClassId(name.clone()),
                    input_types: data_types_to_local_type(&in_types),
                    output_types: data_types_to_local_type(&out_types),
                };
                for f in members {
                    match f {
                        ClassMembers::ValueFunclet { .. } => {
                            let funclet = lower_val_funclet(f, &name);
                            asm.declarations.push(asm::Declaration::Funclet(funclet));
                        }
                        ClassMembers::Extern { .. } => todo!(),
                    }
                }
                asm.declarations
                    .push(asm::Declaration::FunctionClass(class));
            }
            sf @ TopLevel::SpatialFunclet { .. } => asm
                .declarations
                .push(asm::Declaration::Funclet(lower_spatial_funclet(sf))),
            tf @ TopLevel::TimelineFunclet { .. } => asm
                .declarations
                .push(asm::Declaration::Funclet(lower_timeline_funclet(tf))),
            TopLevel::SchedulingFunc {
                name,
                input,
                output,
                specs,
                statements,
                info,
            } => {
                let res = lower_schedule(
                    &ctx,
                    SchedulingFunc {
                        info,
                        name,
                        input,
                        output,
                        specs,
                        statements,
                    },
                )?;
                asm.declarations
                    .extend(res.into_iter().map(asm::Declaration::Funclet));
            }
            _ => todo!(),
        }
    }
    Ok(asm)
}
