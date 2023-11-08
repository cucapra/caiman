use crate::parse::ast::{ClassMembers, DataType, NumberType, TopLevel};
use caiman::assembly::ast as asm;
mod cfg;
mod global_context;
mod lower_schedule;
mod lower_spec;

use lower_spec::{lower_spatial_funclet, lower_timeline_funclet, lower_val_funclet};

const BOOL_FFI_TYPE: asm::FFIType = asm::FFIType::U8;

/// Converts a high-level caiman data type to a caiman assembly type id.
fn data_type_to_type(dt: &DataType) -> asm::TypeId {
    use asm::{FFIType, TypeId};
    match dt {
        DataType::Bool => TypeId::FFI(BOOL_FFI_TYPE),
        DataType::Num(NumberType::I32) => TypeId::FFI(FFIType::I32),
        DataType::Num(NumberType::I64) => TypeId::FFI(FFIType::I64),
        DataType::BufferSpace => TypeId::Local(String::from("BufferSpace")),
        DataType::Tuple(dts) => {
            let tys = data_types_to_type(dts)
                .into_iter()
                .map(|x| match x {
                    TypeId::FFI(ffi) => ffi,
                    TypeId::Local(_) => panic!("TODO: Tuple type contains non-FFI type"),
                })
                .collect();
            TypeId::FFI(FFIType::Tuple(tys))
        }
        DataType::Event => TypeId::Local(String::from("Event")),
        DataType::UserDefined(name) => TypeId::Local(name.clone()),
        _ => unimplemented!("TODO"),
    }
}

/// Convert a high-level caiman data type to a caiman assembly type.
fn data_types_to_type(dts: &[DataType]) -> Vec<asm::TypeId> {
    dts.iter().map(data_type_to_type).collect()
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
#[must_use]
pub fn lower(hlc: Vec<TopLevel>) -> asm::Program {
    // Preprocessing: (before this function)
    // 1. Match literals to literals in the spec
    // 2. Constant fold constants
    // 3. Flatten AST
    //  Flattening will do the following:
    //  * Remove all nested expressions
    //  * Convert all operators to external functions
    //  * Convert all function and expression arguments to names
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
                    input_types: data_types_to_type(&in_types),
                    output_types: data_types_to_type(&out_types),
                };
                for f in members {
                    match f {
                        ClassMembers::ValueFunclet { .. } => {
                            let funclet = lower_val_funclet(f, &name);
                            asm.declarations.push(asm::Declaration::Funclet(funclet));
                        }
                        ClassMembers::Extern { .. } => unimplemented!(),
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
            _ => unimplemented!(),
        }
    }
    asm
}
