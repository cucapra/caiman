use crate::{
    error::{self, type_error, Info, LocalError},
    parse::ast::{
        Binop, ClassMembers, DataType, ExternDef, FloatSize, InputOrOutputVal, IntSize,
        SchedulingFunc, TopLevel, Uop,
    },
    typing::Context,
};
use caiman::assembly::ast as asm;
mod lower_schedule;
mod lower_spec;
mod sched_hir;

use lower_schedule::lower_schedule;
use lower_spec::lower_spec;

#[macro_export]
macro_rules! enum_cast {
    ($p:path, $e:expr) => {
        match $e {
            $p(x) => x,
            _x => panic!(
                "AST Not flattened!: Expected {}, but got {:?}",
                stringify!($p),
                _x
            ),
        }
    };
    ($p:pat, $r:expr, $e:expr) => {
        match $e {
            $p => $r,
            _x => panic!(
                "AST Not flattened!: Expected {}, but got {:?}",
                stringify!($p),
                _x
            ),
        }
    };
}

// TODO: only i32, i64, and u64 are currently supported in the IR
// change this to u8 or i8 once we support those types
pub const BOOL_FFI_TYPE: asm::FFIType = asm::FFIType::I32;
/// The prefix for the name of the actual input arguments in spec funclets
/// (as opposed to the phi node which is the same name without the prefix).
const IN_STEM: &str = "_in_";

impl DataType {
    /// For types that have FFI equivalents, convert a high-level caiman data type
    /// to the caiman assembly type for the corresponding FFI type. For types
    /// that do not have FFI equivalents, return `None`.
    ///
    /// The types with equivalents are value types, not reference types.
    #[must_use]
    pub const fn ffi(&self) -> Option<asm::FFIType> {
        use asm::FFIType;
        match self {
            Self::Bool => Some(BOOL_FFI_TYPE),
            Self::Int(IntSize::I32) => Some(FFIType::I32),
            Self::Int(IntSize::I64) => Some(FFIType::I64),
            Self::Float(FloatSize::F64) => Some(FFIType::F64),
            _ => None,
        }
    }

    /// For types that can be stored in memory, converts a high-level caiman data type
    /// to the caiman assembly type that an allocation would have to store a value
    /// of this type. References are unwrapped to get the underlying type.
    #[must_use]
    pub fn storage_type(&self) -> asm::FFIType {
        match self {
            Self::Ref(d) => d.storage_type(),
            _ => self.ffi().unwrap_or_else(|| unimplemented!("Undefined type {self:?}")),
        }
    }


    /// Converts a high-level caiman data type to a caiman assembly type id.
    #[must_use] 
    pub fn asm_type(&self) -> asm::TypeId {
        use asm::TypeId;
        match self {
            Self::Bool => TypeId(String::from("bool")),
            Self::Int(IntSize::I32) => TypeId(String::from("i32")),
            Self::Int(IntSize::I64) => TypeId(String::from("i64")),
            Self::Float(FloatSize::F64) => TypeId(String::from("f64")),
            Self::BufferSpace => TypeId(String::from("BufferSpace")),
            Self::Event => TypeId(String::from("Event")),
            Self::UserDefined(name) => TypeId(name.clone()),
            Self::Encoder(_) => TypeId(String::from("Encoder")),
            Self::Fence(_) => TypeId(String::from("Fence")),
            Self::Ref(t) => TypeId(format!(
                "&{}",
                t.asm_type()
            )),
            x => unimplemented!("TODO: {x:?}"),
        }
    }
    
}

/// Convert a high-level caiman data type to a caiman assembly type.
fn data_types_to_local_type(dts: &[DataType]) -> Vec<asm::TypeId> {
    dts.iter().map(DataType::asm_type).collect()
}

/// Lower a high-level caiman program to caiman assembly.
/// Requires that the high-level caiman program is well-typed and flattened.
/// # Errors
/// Returns an error if the program is not well-typed or flattened.
/// # Panics
/// If lowering something with currently unsupported language features.
pub fn lower(hlc: Vec<TopLevel>, typing_ctx: &Context, no_inference: bool) -> Result<asm::Program, error::LocalError> {
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
    asm.declarations
        .extend(typing_ctx.type_decls.iter().cloned());
    for top in hlc {
        match top {
            TopLevel::Pipeline { name, entry, .. } => {
                let pipeline = asm::Pipeline {
                    name,
                    funclet: asm::FuncletId(entry),
                    effect: Some(asm::EffectId(String::from("_loop_eff"))),
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
                        ClassMembers::SpatialFunclet(..) |
                        ClassMembers::TimelineFunclet(..) |
                        ClassMembers::ValueFunclet(..) => {
                            let funclet = lower_spec(f, &name, typing_ctx);
                            asm.declarations.push(asm::Declaration::Funclet(funclet));
                        }
                        ClassMembers::Extern {
                            name,
                            device,
                            pure,
                            input,
                            output,
                            def,
                            info,
                        } => asm.declarations.push(extern_to_asm(&name, device, pure, input, output, def, info, &class.name)?),
                    }
                }
                asm.declarations
                    .push(asm::Declaration::FunctionClass(class));
            }
            TopLevel::SchedulingFunc {
                name,
                input,
                output,
                specs,
                statements,
                info,
            } => {
                let res = lower_schedule(
                    typing_ctx,
                    SchedulingFunc {
                        info,
                        name,
                        input,
                        output,
                        specs,
                        statements,
                    },
                    no_inference,
                )?;
                asm.declarations
                    .extend(res.into_iter().map(asm::Declaration::Funclet));
            }
            TopLevel::Typedef { .. } => (), // TODO: more than just records
            _ => todo!(),
        }
    }
    Ok(asm)
}

const fn binop_name(op: Binop) -> &'static str {
    match op {
        Binop::Lt => "lt",
        Binop::Leq => "leq",
        Binop::Gt => "gt",
        Binop::Geq => "geq",
        Binop::Eq => "eq",
        Binop::Neq => "neq",
        Binop::Add => "add",
        Binop::Sub => "sub",
        Binop::Mul => "mul",
        Binop::Div => "div",
        Binop::Mod => "mod",
        Binop::And => "and",
        Binop::Or => "or",
        Binop::Xor => "xor",
        Binop::Shl => "shl",
        Binop::Shr => "shr",
        Binop::Dot => "dot",
        Binop::Cons => "cons",
        Binop::Index => "index",
        Binop::Land => "land",
        Binop::Lor => "lor",
        Binop::AShr => "ashr",
        Binop::Range => "range",
    }
}

const fn uop_name(op: Uop) -> &'static str {
    match op {
        Uop::Neg => "neg",
        Uop::Not => "not",
        Uop::LNot => "lnot",
        Uop::Deref => "deref",
        Uop::Ref => "ref",
    }
}

/// Converts a high-level caiman data type to an extern funclet id.
#[must_use]
pub fn binop_to_str(op: Binop, type_left: &str, type_right: &str) -> String {
    format!("_{}_{type_left}_{type_right}", binop_name(op))
}

#[must_use]
pub fn uop_to_str(op: Uop, type_in: &str) -> String {
    format!("_{}_{type_in}", uop_name(op))
}

/// Gets the id of the direct result of an operation or call that results in `names`.
/// # Panics
/// Panics if `names` starts with a digit or is empty.
#[must_use]
pub fn tuple_id(names: &[String]) -> String {
    assert!(names
        .iter()
        .all(|n| !n.is_empty() && !char::is_digit(n.chars().next().unwrap(), 10)));
    if names.len() == 1 {
        format!("_t{}", names[0])
    } else {
        format!("_t_{}", names.join("_"))
    }
}

/// Converts an extern def into an assembly external gpu info.
/// Returns `None` if the extern def is `None`.
fn get_gpu_info(def: Option<ExternDef>) -> Option<asm::ExternalGPUInfo> {
    def.map(|def| asm::ExternalGPUInfo {
        shader_module: def.path,
        entry_point: def.entry,
        dimensionality: def.dimensions,
        resource_bindings: def
            .resources
            .into_iter()
            .map(|res| {
                let input;
                let output;
                match res.caiman_val {
                    InputOrOutputVal::Input(i) => {
                        input = Some(asm::NodeId(i));
                        output = None;
                    }
                    InputOrOutputVal::Output(o) => {
                        input = None;
                        output = Some(asm::NodeId(o));
                    }
                };
                asm::ExternalGpuFunctionResourceBinding {
                    input,
                    output,
                    group: res.group,
                    binding: res.binding,
                }
            })
            .collect(),
    })
}

/// Converts an extern def into a declaration for an assembly external function
/// # Errors
/// Returns an error if the device is not recognized.
/// # Panics
/// Panics if types could not be converted to FFI types
#[allow(clippy::too_many_arguments)]
fn extern_to_asm(
    name: &str,
    device: String,
    pure: bool,
    input: Vec<(Option<String>, DataType)>,
    output: Vec<(Option<String>, DataType)>,
    def: Option<ExternDef>,
    info: Info,
    class_name: &asm::FunctionClassId,
) -> Result<asm::Declaration, LocalError> {
    Ok(asm::Declaration::ExternalFunction(asm::ExternalFunction {
        name: name.to_string(),
        kind: match (device, pure) {
            (d, true) if d == "cpu" => asm::ExternalFunctionKind::CPUPure,
            (d, false) if d == "cpu" => asm::ExternalFunctionKind::CPUEffect,
            (d, _) if d == "gpu" => asm::ExternalFunctionKind::GPU(
                get_gpu_info(def).map_or_else(|| Err(type_error(info, 
                    &format!("{name} is declared to be a gpu external function but contains no GPU info"))), Ok)?,
            ),
            (d, _) => {
                return Err(type_error(
                    info,
                    &format!(
                        "Unknown external function device {d} for function {name}"
                    ),
                ))
            }
        },
        input_args: input
            .into_iter()
            .map(|(n, t)| asm::ExternalArgument{
                name: n.map(asm::NodeId),
                ffi_type: t.ffi().unwrap(),
            })
            .collect(),
        output_types: output.into_iter().map(|(n, t)| asm::ExternalArgument {
            name: n.map(asm::NodeId),
            ffi_type: t.ffi().unwrap(),
        }).collect(),
        value_function_binding: asm::FunctionClassBinding {
            default: false,
            function_class: class_name.clone(),
        },
        
    }))
}
