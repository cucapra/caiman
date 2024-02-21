use crate::ir;
use crate::rust_wgpu_backend::ffi;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Explication and frontend AST

pub type Hole<T> = Option<T>;

#[macro_export]
macro_rules! def_assembly_id_type {
    ( $type : ident ) => {
        #[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
        pub struct $type(pub String); // temporarily exposed internals

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
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::*;
                String::deserialize::<D>(deserializer).map(|x| Self(x))
            }
        }
    };
}

def_assembly_id_type!(FuncletId);
def_assembly_id_type!(MetaId);
def_assembly_id_type!(ExternalFunctionId);
def_assembly_id_type!(FunctionClassId);
def_assembly_id_type!(NodeId);
def_assembly_id_type!(LocalTypeId);

pub type StorageTypeId = TypeId;

// FFI stuff, rebuilt for a few reasons (mostly strings)

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FFIStructField {
    pub name: String,
    pub type_id: TypeId,
    pub byte_offset: usize,
    pub byte_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum FFIType {
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
        element_type: Box<FFIType>,
        length: usize,
    },
    ErasedLengthArray(Box<FFIType>),
    Struct {
        fields: Box<[FFIStructField]>,
        byte_alignment: Option<usize>,
        byte_size: Option<usize>,
    },
    Tuple(Vec<FFIType>),

    // Reference types
    ConstRef(Box<FFIType>),
    MutRef(Box<FFIType>),
    ConstSlice(Box<FFIType>),
    MutSlice(Box<FFIType>),
    GpuBufferRef(Box<FFIType>),
    GpuBufferSlice(Box<FFIType>),
    GpuBufferAllocator,
    CpuBufferAllocator,
    CpuBufferRef(Box<FFIType>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum TypeId {
    FFI(FFIType),
    Local(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGpuFunctionResourceBinding {
    pub group: usize,
    pub binding: usize,
    pub input: Option<NodeId>,
    pub output: Option<NodeId>,
}

// keeping this idea around for the frontend, easier to reason about for tags
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteNodeId {
    pub funclet: Hole<FuncletId>,
    pub node: Hole<NodeId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagRemoteId {
    pub funclet: Hole<MetaId>,
    // we need an option of a hole
    // since None is explicitly different than ?
    pub node: Option<Hole<NodeId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub quot: Hole<TagRemoteId>, // What a given value maps to in a specification
    pub flow: ir::Flow,          // How this value transforms relative to the specification
}

// Super Jank, but whatever

macro_rules! lookup_abstract_type_parser {
	([$elem_type:ident]) => { Vec<Hole<lookup_abstract_type_parser!($elem_type)>> };
	(Type) => { TypeId };
	(Immediate) => { String };
	(ImmediateI64) => { i64 };
	(ImmediateI32) => { i32 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalFunction) => { ExternalFunctionId };
	(ValueFunction) => { FunctionClassId };
	(Operation) => { NodeId };
	(RemoteOperation) => { RemoteNodeId };
	(Place) => { ir::Place };
	(Funclet) => { FuncletId };
	(StorageType) => { StorageTypeId };
    (BufferFlags) => { ir::BufferFlags };
}

macro_rules! map_parser_refs {
    // When mapping referenced nodes, we only care about mapping the Operation types,
    // since those are the actual references.
    ($map:ident, $arg:ident : Operation) => {
        $arg.as_ref().map(|x| $map(x.clone()))
    };
    ($map:ident, $arg:ident : [Operation]) => {
        $arg.as_ref().map(|lst| {
            lst.iter()
                .map(|arg_hole| arg_hole.as_ref().map(|arg| $map(arg.clone())))
                .collect()
        })
    };
    ($_map:ident, $arg:ident : $_arg_type:tt) => {
        $arg.clone()
    };
}

macro_rules! make_parser_nodes {
	(@ $map:ident {} -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		#[derive(Serialize, Deserialize, Debug, Clone)]
		pub enum Node {
			$($fields)*
		}
		impl Node {
			pub fn map_referenced_nodes(&self,
            mut $map: impl FnMut(NodeId) -> NodeId) -> Self {
				match self {$($mapper)*}
			}
		}
	};
	(@ $map:ident {$name:ident (), $($rest:tt)*} -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		make_parser_nodes! {
			@ $map { $($rest)* } ->
			($($fields)* $name,),
			($($mapper)* Self::$name => Self::$name,)
		}
	};
	(@ $map:ident {$name:ident ($($arg:ident : $arg_type:tt,)*), $($rest:tt)*}
        -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		make_parser_nodes! {
			@ $map { $($rest)* } ->
			($($fields)* $name { $($arg: Hole<lookup_abstract_type_parser!($arg_type)>),* },),
			($($mapper)* Self::$name { $($arg),* } => Self::$name {
				$($arg: map_parser_refs!($map, $arg : $arg_type)),*
			},)
		}
	};
	($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
		make_parser_nodes! { @ f {$($name ($($arg : $arg_type,)*),)*} -> (), () }
	};
}

with_operations!(make_parser_nodes);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge {
    // Here for now as a type system debugging tool
    // Always passes type checking, but fails codegen
    DebugHole {
        // Scalar nodes
        inputs: Vec<NodeId>,
        // Continuations
        //outputs : Box<[NodeId]>
    },

    // Common?
    Return {
        return_values: Hole<Vec<Hole<NodeId>>>,
    },
    Jump {
        join: Hole<NodeId>,
        arguments: Hole<Vec<Hole<NodeId>>>,
    },

    // Scheduling only
    // Split value - what will be computed
    ScheduleCall {
        operations: Hole<Vec<Hole<TagRemoteId>>>,
        callee_funclet_id: Hole<FuncletId>,
        callee_arguments: Hole<Vec<Hole<NodeId>>>,
        continuation_join: Hole<NodeId>,
    },
    ScheduleSelect {
        operations: Hole<Vec<Hole<TagRemoteId>>>,
        condition: Hole<NodeId>,
        callee_funclet_ids: Hole<Vec<Hole<FuncletId>>>,
        callee_arguments: Hole<Vec<Hole<NodeId>>>,
        continuation_join: Hole<NodeId>,
    },
    ScheduleCallYield {
        operations: Hole<Vec<Hole<TagRemoteId>>>,
        external_function_id: Hole<ExternalFunctionId>,
        yielded_nodes: Hole<Vec<Hole<NodeId>>>,
        continuation_join: Hole<NodeId>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionClassBinding {
    pub default: bool,
    pub function_class: FunctionClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaMapping {
    // map from the name to the associated funclet id
    pub value: (MetaId, FuncletId),
    pub timeline: (MetaId, FuncletId),
    pub spatial: (MetaId, FuncletId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleBinding {
    pub implicit_tags: (Tag, Tag),
    pub meta_map: MetaMapping
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FuncletBinding {
    SpecBinding(FunctionClassBinding),
    ScheduleBinding(ScheduleBinding),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncletArgument {
    pub name: Option<NodeId>,
    pub typ: TypeId,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncletHeader {
    pub name: FuncletId,
    pub args: Vec<FuncletArgument>,
    pub ret: Vec<FuncletArgument>,
    pub binding: FuncletBinding,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamedNode {
    pub name: Option<NodeId>,
    pub node: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Node(NamedNode),
    TailEdge(TailEdge),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Funclet {
    pub kind: ir::FuncletKind,
    pub header: FuncletHeader,
    pub commands: Vec<Hole<Command>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LocalTypeInfo {
    NativeValue {
        storage_type: TypeId,
    },

    // Scheduling
    Ref {
        storage_type: TypeId,
        storage_place: ir::Place,
        buffer_flags: ir::BufferFlags,
    },
    Fence {
        queue_place: ir::Place,
    },
    Buffer {
        storage_place: ir::Place,
        static_layout_opt: Option<ir::StaticBufferLayout>,
        flags: ir::BufferFlags,
    },
    Encoder {
        queue_place: ir::Place,
    },

    // Timeline
    Event,

    // Space
    BufferSpace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalType {
    pub name: String,
    pub data: LocalTypeInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeDecl {
    FFI(FFIType),
    Local(LocalType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Var {
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGPUInfo {
    pub shader_module: String,
    pub entry_point: String,
    pub dimensionality: usize,
    pub resource_bindings: Vec<ExternalGpuFunctionResourceBinding>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExternalFunctionKind {
    CPUPure,
    CPUEffect,
    GPU(ExternalGPUInfo),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalArgument {
    pub name: Option<NodeId>,
    pub ffi_type: FFIType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalFunction {
    pub kind: ExternalFunctionKind,
    pub value_function_binding: FunctionClassBinding,
    pub name: String,
    pub input_args: Vec<ExternalArgument>,
    pub output_types: Vec<ExternalArgument>,
    // Contains pipeline and single render pass state
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub detailed: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionClass {
    pub name: FunctionClassId,
    pub input_types: Vec<TypeId>,
    pub output_types: Vec<TypeId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub funclet: FuncletId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Declaration {
    TypeDecl(TypeDecl),
    ExternalFunction(ExternalFunction),
    FunctionClass(FunctionClass),
    Funclet(Funclet),
    Pipeline(Pipeline),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    // need the path to open locally from this program file
    pub path: String,
    pub version: Version,
    pub declarations: Vec<Declaration>,
}
