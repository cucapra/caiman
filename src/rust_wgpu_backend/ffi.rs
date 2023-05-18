use crate::stable_vec::StableVec;
use serde_derive::{Deserialize, Serialize};

// Maybe this should be a derive macro
#[macro_export]
macro_rules! def_id_type {
    ( $type : ident ) => {
        #[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
        pub struct $type(pub usize); // temporarily exposed internals

        /*impl $type
        {
            pub fn from(v : usize) -> Self
            {
                Self(v)
            }

            pub fn as_usize(&self) -> usize
            {
                self.0
            }
        }*/

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_u64(self.0 as u64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::*;
                u64::deserialize::<D>(deserializer).map(|x| Self(x as usize))
            }
        }
    };
}

def_id_type!(TypeId);
def_id_type!(ExternalFunctionId);
def_id_type!(EffectId);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_id: TypeId,
    pub byte_offset: usize,
    pub byte_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    // Value types
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    USize,
    I8,
    I16,
    I32,
    I64,
    Array {
        element_type: TypeId,
        length: usize,
    },
    ErasedLengthArray {
        element_type: TypeId,
    },
    Struct {
        fields: Box<[StructField]>,
        byte_alignment: Option<usize>,
        byte_size: Option<usize>,
    },
    Tuple {
        fields: Box<[TypeId]>,
    },

    // Reference types
    ConstRef {
        element_type: TypeId,
    },
    MutRef {
        element_type: TypeId,
    },
    ConstSlice {
        element_type: TypeId,
    },
    MutSlice {
        element_type: TypeId,
    },
    GpuBufferRef {
        element_type: TypeId,
    },
    GpuBufferSlice {
        element_type: TypeId,
    },
    GpuBufferAllocator,
    CpuBufferAllocator,
    CpuBufferRef {
        element_type: TypeId,
    },
}

impl Type {
    pub fn estimate_size(&self, types: &StableVec<Type>) -> usize {
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
            Self::U64 => 8,
            Self::USize => std::mem::size_of::<usize>(),
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::Array {
                element_type,
                length,
            } => {
                // assuming no padding, probably incorrect
                // that's ok though, this is only an estimate
                return length * types[element_type.0].estimate_size(types);
            }
            // TODO: finish this for other types
            _ => usize::MAX,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuPureOperation {
    pub name: String,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuEffectfulOperation {
    pub name: String,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
}

// This describes the initial mapping from the binding in the shader to the IR
// It's expected codegen will emit a module with a different mapping
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuKernelResourceBinding {
    pub group: usize,
    pub binding: usize,
    pub input: Option<usize>,
    pub output: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderModuleContent {
    Wgsl(String),
    Glsl(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuKernel {
    pub name: String,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
    // Contains pipeline and single render pass state
    pub entry_point: String,
    pub resource_bindings: Box<[GpuKernelResourceBinding]>,
    pub shader_module_content: ShaderModuleContent,
    //pub shader_module : usize,
}

impl GpuKernel {
    // Once we stop needing to directly program in .ron files, we can make this more efficient
    pub fn output_of_forwarding_input(&self, input_index: usize) -> Option<usize> {
        for resource_binding in self.resource_bindings.iter() {
            if let Some(index) = resource_binding.input {
                if index == input_index {
                    return resource_binding.output;
                }
            }
        }

        return None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExternalFunction {
    CpuEffectfulOperation(CpuEffectfulOperation),
    CpuPureOperation(CpuPureOperation),
    GpuKernel(GpuKernel),
}

impl ExternalFunction {
    pub fn get_gpu_kernel(&self) -> Option<&GpuKernel> {
        if let Self::GpuKernel(kernel) = self {
            Some(kernel)
        } else {
            None
        }
    }

    pub fn get_cpu_effectful_operation(&self) -> Option<&CpuEffectfulOperation> {
        if let Self::CpuEffectfulOperation(op) = self {
            Some(op)
        } else {
            None
        }
    }

    pub fn get_cpu_pure_operation(&self) -> Option<&CpuPureOperation> {
        if let Self::CpuPureOperation(op) = self {
            Some(op)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Effect {
    Unrestricted,
    FullyConnected {
        effectful_function_ids: Vec<ExternalFunctionId>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NativeInterface {
    #[serde(default)]
    pub types: StableVec<Type>,
    #[serde(default)]
    pub external_functions: StableVec<ExternalFunction>,
    #[serde(default)]
    pub effects: StableVec<Effect>,
}

pub struct TypeBindingInfo {
    pub size: usize,
    pub alignment: usize,
}

impl NativeInterface {
    // For the sake of an MVP, this is assuming that we're compiling for the machine we're running on

    pub fn calculate_type_binding_info(&self, type_id: TypeId) -> TypeBindingInfo {
        match &self.types[type_id.0] {
            Type::F32 => TypeBindingInfo {
                size: std::mem::size_of::<f32>(),
                alignment: std::mem::align_of::<f32>(),
            },
            Type::F64 => TypeBindingInfo {
                size: std::mem::size_of::<f64>(),
                alignment: std::mem::align_of::<f64>(),
            },
            Type::U8 => TypeBindingInfo {
                size: std::mem::size_of::<u8>(),
                alignment: std::mem::align_of::<u8>(),
            },
            Type::U16 => TypeBindingInfo {
                size: std::mem::size_of::<u16>(),
                alignment: std::mem::align_of::<u16>(),
            },
            Type::U32 => TypeBindingInfo {
                size: std::mem::size_of::<u32>(),
                alignment: std::mem::align_of::<u32>(),
            },
            Type::U64 => TypeBindingInfo {
                size: std::mem::size_of::<u64>(),
                alignment: std::mem::align_of::<u64>(),
            },
            Type::USize => TypeBindingInfo {
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::align_of::<usize>(),
            },
            Type::I8 => TypeBindingInfo {
                size: std::mem::size_of::<i8>(),
                alignment: std::mem::align_of::<i8>(),
            },
            Type::I16 => TypeBindingInfo {
                size: std::mem::size_of::<i16>(),
                alignment: std::mem::align_of::<i16>(),
            },
            Type::I32 => TypeBindingInfo {
                size: std::mem::size_of::<i32>(),
                alignment: std::mem::align_of::<i32>(),
            },
            Type::I64 => TypeBindingInfo {
                size: std::mem::size_of::<i64>(),
                alignment: std::mem::align_of::<i64>(),
            },
            Type::ConstRef { element_type } => panic!("Unimplemented"),
            Type::MutRef { element_type } => panic!("Unimplemented"),
            Type::ConstSlice { element_type } => panic!("Unimplemented"),
            Type::MutSlice { element_type } => panic!("Unimplemented"),
            Type::Array {
                element_type,
                length,
            } => {
                let inner = self.calculate_type_binding_info(*element_type);
                TypeBindingInfo {
                    size: inner.size * length,
                    alignment: inner.alignment,
                }
            }
            Type::Struct {
                fields,
                byte_alignment,
                byte_size,
            } => panic!("Unimplemented"),
            _ => panic!("Unimplemented"),
        }
    }

    pub fn calculate_type_alignment_bits(&self, type_id: TypeId) -> usize {
        self.calculate_type_binding_info(type_id)
            .alignment
            .trailing_zeros() as usize
    }

    pub fn calculate_type_byte_size(&self, type_id: TypeId) -> usize {
        self.calculate_type_binding_info(type_id).size
    }
}
