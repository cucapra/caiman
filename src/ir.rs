use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
//use bitflags::bitflags;
use crate::arena::Arena;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place
{
	Local,
	Cpu,
	Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceQueueStage
{
	None,
	Encoded,
	Submitted,
	Ready,
	Dead
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ResourceState
{
	pub stage : ResourceQueueStage,
	pub is_exclusive : bool
}

pub type ExternalCpuFunctionId = usize;
pub type ExternalGpuFunctionId = usize;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type OperationId = NodeId;
pub type TypeId = usize;
pub type PlaceId = usize;
pub type ValueFunctionId = usize;
pub type FenceId = usize;
//pub type LocalMetaVariableId = usize;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId{pub funclet_id : FuncletId, pub node_id : NodeId}

mod generated
{
	use super::*;
	include!(concat!(env!("OUT_DIR"), "/generated/ir.txt"));
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructField
{
	pub name : String,
	pub type_id : TypeId,
	pub byte_offset : usize,
	pub byte_size : usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueTag
{
	// Intended for scheduling purposes
	None,
	// These two are implementation-agnostic and are only allowed in external interfaces
	Input{function_id : ValueFunctionId, index : usize},
	Output{function_id : ValueFunctionId, index : usize},
	// These are not, and are intended for funclets
	Operation{ remote_node_id : RemoteNodeId },
	ConcreteInput{funclet_id : FuncletId, index : usize},
	ConcreteOutput{funclet_id : FuncletId, index : usize},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type
{
	// Value types

	F32,
	F64,
	U8,
	U16,
	U32,
	U64,
	I8,
	I16,
	I32,
	I64,
	Array { element_type : TypeId, length : usize },
	Struct { fields : Box<[StructField]>, byte_alignment : Option<usize>, byte_size : Option<usize> },
	Tuple { fields : Box<[TypeId]> },

	// Resource types

	ConstRef { element_type : TypeId },
	MutRef { element_type : TypeId },
	ConstSlice { element_type : TypeId },
	MutSlice { element_type : TypeId },
	//Buffer {local_resource_id : LocalMetaVariableId},
	//BufferRef {local_resource_id : LocalMetaVariableId},
	//BufferMutRef {local_resource_id : LocalMetaVariableId},
	//Texture

	//Fence { id : FenceId },

	Slot{ value_type : TypeId, queue_stage : ResourceQueueStage, queue_place : Place },
	//Slot{ value_type : TypeId, value_tag : ValueTag, queue_stage : ResourceQueueStage, queue_place : Place, fence_id : FenceId },
	//Slot,
}

// Local Meta Variables are used to serve as ids for when types in input/output lists need to relate to each other
// This allows them to do so without refering directly to an input, output, or node position
/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LocalMetaVariable
{
	Resource,
	Fence(Fence),
	ValueInstance
}*/

pub use generated::Node;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	// Common
	Return { return_values : Box<[NodeId]> },
	Yield { funclet_ids : Box<[FuncletId]>, captured_arguments : Box<[NodeId]>, return_values : Box<[NodeId]> },

	// Scheduling only
	ScheduleCall { value_operation : RemoteNodeId, callee_funclet_id : FuncletId, callee_arguments : Box<[NodeId]>, continuation_funclet_id : FuncletId, continuation_arguments : Box<[NodeId]> },
	ScheduleSelect { value_operation : RemoteNodeId, callee_funclet_ids : Box<[FuncletId]>, callee_arguments : Box<[NodeId]>, continuation_funclet_id : FuncletId },

	// invokes and waits on the gpu
	//ReturnWithGpuCoordinator { initial_return_values : Box<[NodeId]>, gpu_funclet_id : FuncletId, arguments : Box<[NodeId]> },
	//Wait { required_scope_set : ScopeSet, funclet_id : usize, arguments : Box<[usize]> }
	//Jump { join : usize, arguments : Box<[usize]> },
	//BranchIf { condition : NodeId, true_case : FuncletId, true_arguments : Box<[NodeId]>, false_case : FuncletId, false_arguments : Box<NodeId> },
	//Call { callee_block_id : usize, callee_block_arguments : Box<[usize]>, continuation_block_id : usize, continuation_context_values : Box<[usize]> },
	//CallGpuCoordinator { callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
	//CallGpuWorker{ callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FuncletKind
{
	MixedImplicit,
	MixedExplicit,
	Value,
	ScheduleExplicit,
	Inline // Adopts the constraints of the calling funclet
}

impl FuncletKind
{
	fn easy_default() -> Self
	{
		FuncletKind::MixedImplicit
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fence
{
	pub prior_fence_ids : Box<[FenceId]>,
	pub place : Place
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Funclet
{
	#[serde(default = "FuncletKind::easy_default")]
	pub kind : FuncletKind,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
	pub nodes : Box<[Node]>,
	pub tail_edge : TailEdge,

	#[serde(default)]
	pub input_resource_states : Box<[BTreeMap<Place, ResourceState>]>,
	#[serde(default)]
	pub output_resource_states : Box<[BTreeMap<Place, ResourceState>]>,
}

// Funclet-relative slot info goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotInfo
{
	pub value_tag : ValueTag,
	//pub queue_stage : ResourceQueueStage,
	//pub queue_place : Place,
	//pub fence_id : FenceId
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchedulingFuncletExtra
{
	pub value_funclet_id : FuncletId,
	pub input_slots : HashMap<usize, SlotInfo>,
	pub output_slots : HashMap<usize, SlotInfo>,
	pub fences : BTreeMap<FenceId, Fence>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueFuncletExtra
{
	// Value functions this funclet implements
	#[serde(default)]
	pub value_function_ids : BTreeSet<ValueFunctionId>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalCpuFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
}

// This describes the initial mapping from the binding in the shader to the IR
// It's expected codegen will emit a module with a different mapping
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGpuFunctionResourceBinding
{
	pub group : usize,
	pub binding : usize,
	pub input : Option<usize>,
	pub output : Option<usize>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderModuleContent
{
	Wgsl(String)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGpuFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
	// Contains pipeline and single render pass state
	pub entry_point : String,
	pub resource_bindings : Box<[ExternalGpuFunctionResourceBinding]>,
	pub shader_module_content : ShaderModuleContent,
	//pub shader_module : usize,
}

// A value function is just an equivalence class over functions that behave identically at the value level
// A schedule can substitute a call to it for an implementation iff that implementation is associated with the value function
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
	pub default_funclet_id : Option<FuncletId>
}

// A user-facing entry point into the pipeline
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipelineMethod
{
	pub name : String,
	pub capture_types : Box<[TypeId]>,
	pub argument_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
	pub entry_funclet : Option<FuncletId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline
{
	pub name : String,
	pub entry_funclet : FuncletId,
	#[serde(default)]
	pub reentry_methods : Box<[PipelineMethod]>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Program
{
	//pub types : HashMap<usize, Type>,
	#[serde(default)]
	pub types : Arena<Type>,
	#[serde(default)]
	pub funclets : Arena<Funclet>,
	#[serde(default)]
	pub external_cpu_functions : Vec<ExternalCpuFunction>,
	#[serde(default)]
	pub external_gpu_functions : Vec<ExternalGpuFunction>,
	#[serde(default)]
	pub value_functions : Arena<ValueFunction>,
	#[serde(default)]
	pub pipelines : Vec<Pipeline>,
	#[serde(default)]
	pub value_funclet_extras : HashMap<FuncletId, ValueFuncletExtra>,
	#[serde(default)]
	pub scheduling_funclet_extras : HashMap<FuncletId, SchedulingFuncletExtra>,
	//pub shader_modules : HashMap<usize, ShaderModule>
}

impl Program
{
	pub fn new() -> Self
	{
		Default::default()
	}
}

/*#[derive(Debug, Default)]
pub struct ProgramBuilder
{

}

impl ProgramBuilder
{
	fn new() -> Self
	{
		Default::default()
	}
}*/
