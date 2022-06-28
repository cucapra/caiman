use std::collections::{HashMap, BTreeMap};
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
pub type LocalMetaVariableId = usize;

mod generated
{
	use super::*;
	include!(concat!(env!("OUT_DIR"), "/generated/ir.txt"));
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructField
{
	pub name : String,
	pub type_id : TypeId,
	pub byte_offset : usize,
	pub byte_size : usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalValueTag
{
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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
	Buffer {local_resource_id : LocalMetaVariableId},
	BufferRef {local_resource_id : LocalMetaVariableId},
	BufferMutRef {local_resource_id : LocalMetaVariableId},
	//Texture

	Fence { id : LocalMetaVariableId, prior_fence_ids : Box<[LocalMetaVariableId]>, place : Place },

	Slot{ value_type : TypeId, value_tag : Option<LocalValueTag>, local_resource_id : LocalMetaVariableId, queue_stage : ResourceQueueStage, place : Place, fence_id : LocalMetaVariableId },
}

// Local Meta Variables are used to serve as ids for when types need to relate to each other
// This allows them to do so without refering directly to an input, output, or node position
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LocalMetaVariable
{
	Resource,
	Fence,
}

pub use generated::Node;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	Return { return_values : Box<[NodeId]> },
	Yield { funclet_ids : Box<[FuncletId]>, captured_arguments : Box<[NodeId]>, return_values : Box<[NodeId]> },
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

	#[serde(default)]
	pub local_meta_variables : BTreeMap<LocalMetaVariableId, LocalMetaVariable>,
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

// 
/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValueFunctionBinding
{
	ExternalCpu{ external_cpu_function_id : ExternalCpuFunctionId, input_map : HashMap<usize, usize>, output_map : HashMap<usize, usize>},
	ExternalGpuComputeDispatchFixedWorkgroups{ external_gpu_function_id : ExternalGpuFunctionId, dimensions : [u32; 3], input_map : HashMap<usize, usize>, output_map : HashMap<usize, usize>},
	ExternalGpuComputeDispatch{ external_gpu_function_id : ExternalGpuFunctionId, dimension_arguments : [usize; 3], input_map : HashMap<usize, usize>, output_map : HashMap<usize, usize>},
}*/

// A user-facing entry point into the pipeline
pub enum PipelineMethod
{
	
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline
{
	pub name : String,
	pub entry_funclet : FuncletId
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
