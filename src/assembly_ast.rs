use crate::assembly_context;
use crate::ir;
use crate::rust_wgpu_backend::ffi;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Explication and frontend AST

pub type Hole<T> = Option<T>;

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
pub enum Type {
    FFI(FFIType),
    Local(String),
}

pub type ExternalCpuFunctionId = String;
pub type ExternalGpuFunctionId = String;
pub type FuncletId = String;
pub type NodeId = String;
pub type OperationId = NodeId;
pub type TypeId = Type;
pub type ValueFunctionId = String;
pub type StorageTypeId = Type;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId {
    pub funclet_id: FuncletId,
    pub node_id: NodeId,
}

// Super Jank, but whatever

macro_rules! lookup_abstract_type_parser {
	([$elem_type:ident]) => { Box<[Hole<lookup_abstract_type_parser!($elem_type)>]> };
	(Type) => { TypeId };
	(Immediate) => { String };
	(ImmediateI64) => { i64 };
	(ImmediateI32) => { i32 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ExternalGpuFunctionId };
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
        $arg.clone().map(|x| $map((*x.clone()).to_string()))
    };
    ($map:ident, $arg:ident : [Operation]) => {
        $arg.clone().map(|x| {
            x.iter()
                .map(|y| y.clone().map(|op| $map((*op.clone()).to_string())))
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
			pub fn map_referenced_nodes(&self, mut $map: impl FnMut(NodeId) -> NodeId) -> Self {
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
        return_values: Hole<Vec<Hole<NodeId>>>,
    },
    Yield {
        pipeline_yield_point_id: Hole<ir::PipelineYieldPointId>,
        yielded_nodes: Hole<Vec<Hole<NodeId>>>,
        next_funclet: Hole<FuncletId>,
        continuation_join: Hole<NodeId>,
        arguments: Hole<Vec<Hole<NodeId>>>,
    },
    Jump {
        join: Hole<NodeId>,
        arguments: Hole<Vec<Hole<NodeId>>>,
    },
    ScheduleCall {
        value_operation: Hole<RemoteNodeId>,
        callee_funclet_id: Hole<FuncletId>,
        callee_arguments: Hole<Vec<Hole<NodeId>>>,
        continuation_join: Hole<NodeId>,
    },
    ScheduleSelect {
        value_operation: Hole<RemoteNodeId>,
        condition: Hole<NodeId>,
        callee_funclet_ids: Hole<Vec<Hole<FuncletId>>>,
        callee_arguments: Hole<Vec<Hole<NodeId>>>,
        continuation_join: Hole<NodeId>,
    },
    DynamicAllocFromBuffer {
        buffer: Hole<NodeId>,
        arguments: Hole<Vec<Hole<NodeId>>>,
        dynamic_allocation_size_slots: Hole<Vec<Hole<Option<NodeId>>>>,
        success_funclet_id: Hole<FuncletId>,
        failure_funclet_id: Hole<FuncletId>,
        continuation_join: Hole<NodeId>,
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
    Halt(NodeId),
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
    VarName(String),
    FnName(String),
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
    pub ret: Vec<(Option<String>, Type)>,
    pub name: String,
    pub args: Vec<(Option<String>, Type)>,
}

#[derive(Debug)]
pub struct Funclet {
    pub kind: ir::FuncletKind,
    pub header: FuncletHeader,
    pub commands: Vec<Hole<Node>>,
    pub tail_edge: Hole<TailEdge>,
}

#[derive(Debug)]
pub enum TypeKind {
    NativeValue,
    Slot,
    Fence,
    Buffer,
    Event,
    BufferSpace,
}

#[derive(Debug)]
pub struct LocalType {
    pub type_kind: TypeKind,
    pub name: String,
    pub data: UncheckedDict,
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
pub struct ExternalCpuFunction {
    pub name: String,
    pub input_types: Vec<FFIType>,
    pub output_types: Vec<FFIType>,
}

#[derive(Debug)]
pub struct ExternalGpuFunction {
    pub name: String,
    pub input_args: Vec<(FFIType, String)>,
    pub output_types: Vec<(FFIType, String)>,
    // Contains pipeline and single render pass state
    pub shader_module: String,
    pub entry_point: String,
    pub resource_bindings: Vec<UncheckedDict>, // do the work later
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
    ExternalCPU(ExternalCpuFunction),
    ExternalGPU(ExternalGpuFunction),
    Local(Funclet),
    ValueFunction(ValueFunction),
}

pub type FuncletDefs = Vec<FuncletDef>;

#[derive(Debug)]
pub struct Extra {
    pub name: String,
    pub data: UncheckedDict,
}

pub type Extras = Vec<Extra>;

#[derive(Debug)]
pub struct Pipeline {
    pub name: String,
    pub funclet: String,
}

pub type Pipelines = Vec<Pipeline>;

#[derive(Debug)]
pub struct Program {
    pub version: Version,
    pub types: Types,
    pub funclets: FuncletDefs,
    pub extras: Extras,
    pub pipelines: Pipelines,
    pub context: assembly_context::Context,
}
