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
def_assembly_id_type!(ExternalFunctionId);
def_assembly_id_type!(ValueFunctionId);
def_assembly_id_type!(OperationId);
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
    pub input: Option<OperationId>,
    pub output: Option<OperationId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Type {
    FFI(FFIType),
    Local(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId {
    pub funclet_name: FuncletId,
    pub node_name: OperationId,
}

// Super Jank, but whatever

macro_rules! lookup_abstract_type_parser {
	([$elem_type:ident]) => { Box<[lookup_abstract_type_parser!($elem_type)]> };
	(Type) => { TypeId };
	(Immediate) => { String };
	(ImmediateI64) => { i64 };
	(ImmediateI32) => { i32 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalFunction) => { ExternalFunctionId };
	(ValueFunction) => { ValueFunctionId };
	(Operation) => { OperationId };
	(RemoteOperation) => { RemoteNodeId };
	(Place) => { ir::Place };
	(Funclet) => { FuncletId };
	(StorageType) => { StorageTypeId };
}

macro_rules! map_parser_refs {
    // When mapping referenced nodes, we only care about mapping the Operation types,
    // since those are the actual references.
    ($map:ident, $arg:ident : Operation) => {
        $map($arg.clone())
    };
    ($map:ident, $arg:ident : [Operation]) => {
        $arg.iter().map(|op| $map(op.clone())).collect()
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
            mut $map: impl FnMut(OperationId) -> OperationId) -> Self {
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
	(@ $map:ident {$name:ident ($($arg:ident : $arg_type:tt,)*), $($rest:tt)*} -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		make_parser_nodes! {
			@ $map { $($rest)* } ->
			($($fields)* $name { $($arg: lookup_abstract_type_parser!($arg_type)),* },),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlotInfo {
    pub value_tag: ValueTag,
    pub timeline_tag: TimelineTag,
    pub spatial_tag: SpatialTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FenceInfo {
    pub timeline_tag: TimelineTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferInfo {
    pub spatial_tag: SpatialTag,
}

#[derive(Debug, Clone)]
pub enum TailEdge {
    Return {
        return_values: Hole<Vec<Hole<OperationId>>>,
    },
    Yield {
        pipeline_yield_point: ExternalFunctionId,
        yielded_nodes: Hole<Vec<Hole<OperationId>>>,
        next_funclet: Hole<FuncletId>,
        continuation_join: Hole<OperationId>,
        arguments: Hole<Vec<Hole<OperationId>>>,
    },
    Jump {
        join: Hole<OperationId>,
        arguments: Hole<Vec<Hole<OperationId>>>,
    },
    ScheduleCall {
        value_operation: Hole<RemoteNodeId>,
        callee_funclet_id: Hole<FuncletId>,
        callee_arguments: Hole<Vec<Hole<OperationId>>>,
        continuation_join: Hole<OperationId>,
    },
    ScheduleSelect {
        value_operation: Hole<RemoteNodeId>,
        condition: Hole<OperationId>,
        callee_funclet_ids: Hole<Vec<Hole<FuncletId>>>,
        callee_arguments: Hole<Vec<Hole<OperationId>>>,
        continuation_join: Hole<OperationId>,
    },
    DynamicAllocFromBuffer {
        buffer: Hole<OperationId>,
        arguments: Hole<Vec<Hole<OperationId>>>,
        dynamic_allocation_size_slots: Hole<Vec<Hole<Option<OperationId>>>>,
        success_funclet_id: Hole<FuncletId>,
        failure_funclet_id: Hole<FuncletId>,
        continuation_join: Hole<OperationId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagCore {
    None,
    Operation(RemoteNodeId),
    Input(RemoteNodeId),
    Output(RemoteNodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueTag {
    Core(TagCore),
    FunctionInput(RemoteNodeId),
    FunctionOutput(RemoteNodeId),
    Halt(OperationId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimelineTag {
    Core(TagCore),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpatialTag {
    Core(TagCore),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    ValueTag(ValueTag),
    TimelineTag(TimelineTag),
    SpatialTag(SpatialTag),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    None,
    ID(String),
    FunctionLoc(RemoteNodeId),
    VarName(OperationId),
    FnName(FuncletId),
    Num(usize),
    Type(Type),
    Place(ir::Place),
    Stage(ir::ResourceQueueStage),
    Tag(Tag),
    SlotInfo(SlotInfo),
    FenceInfo(FenceInfo),
    BufferInfo(BufferInfo),
}

#[derive(Debug, Clone)]
pub enum DictValue {
    Raw(Value),
    List(Vec<DictValue>),
    Dict(UncheckedDict),
}

pub type UncheckedDict = HashMap<Value, DictValue>;

#[derive(Debug, Clone)]
pub struct FuncletHeader {
    pub name: FuncletId,
    pub ret: Vec<(Option<OperationId>, Type)>,
    pub args: Vec<Type>,
}

#[derive(Debug)]
pub struct NamedNode {
    pub name: OperationId,
    pub node: Node,
}

#[derive(Debug)]
pub struct Funclet {
    pub kind: ir::FuncletKind,
    pub header: FuncletHeader,
    pub commands: Vec<Hole<NamedNode>>,
    pub tail_edge: Hole<TailEdge>,
}

#[derive(Debug)]
pub enum LocalTypeInfo {
    NativeValue {
        storage_type: Type,
    },

    // Scheduling
    Slot {
        storage_type: Type,
        queue_stage: ir::ResourceQueueStage,
        queue_place: ir::Place,
    },
    SchedulingJoin {},
    Fence {
        queue_place: ir::Place,
    },
    Buffer {
        storage_place: ir::Place,
        static_layout_opt: Option<ir::StaticBufferLayout>,
    },

    // Timeline
    Event {
        place: ir::Place,
    },

    // Space
    BufferSpace,
}

#[derive(Debug)]
pub struct LocalType {
    pub name: String,
    pub data: LocalTypeInfo,
}

#[derive(Debug)]
pub enum TypeDecl {
    FFI(FFIType),
    Local(LocalType),
}

pub type Types = Vec<TypeDecl>;

#[derive(Debug)]
pub struct Var {
    pub id: usize,
}

#[derive(Debug)]
pub struct ExternalFunction {
    pub name: String,
    pub input_args: Vec<(FFIType, String)>,
    pub output_types: Vec<(FFIType, String)>,
    // Contains pipeline and single render pass state
    pub shader_module: String,
    pub entry_point: String,
    pub resource_bindings: Vec<ExternalGpuFunctionResourceBinding>,
}

#[derive(Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub detailed: u32,
}

#[derive(Debug)]
pub struct ValueFunction {
    pub name: String,
    pub input_types: Vec<TypeId>,
    pub output_types: Vec<TypeId>,
    pub allowed_funclets: Vec<FuncletId>,
}

#[derive(Debug)]
pub enum FuncletDef {
    External(ExternalFunction),
    Local(Funclet),
    ValueFunction(ValueFunction),
}

pub type FuncletDefs = Vec<FuncletDef>;

// todo: rework after extra changes
pub type Extras = HashMap<FuncletId, UncheckedDict>;

pub type Pipelines = HashMap<String, FuncletId>;

#[derive(Debug)]
pub struct Program {
    pub version: Version,
    pub types: Types,
    pub funclets: FuncletDefs,
    pub extras: Extras,
    pub pipelines: Pipelines,
}
