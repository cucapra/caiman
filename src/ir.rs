use std::collections::{HashMap, BTreeMap};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
//use bitflags::bitflags;
use crate::arena::Arena;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope
{
	Local,
	Cpu,
	Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Place
{
	Simple{scope : Scope}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ResourceQueueStage
{
	None,
	Encoded,
	Submitted,
	Ready,
	Dead,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ResourceState
{
	pub stage : ResourceQueueStage,
	pub is_exclusive : bool
}

pub struct NodePlacement
{
	resource_id : usize
}

pub enum Resource
{
	Buffer{size : usize},
	CommandBuffer{size : usize},
}

pub type ExternalCpuFunctionId = usize;
pub type ExternalGpuFunctionId = usize;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type OperationId = NodeId;
pub type TypeId = usize;
pub type PlaceId = usize;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type
{
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
	ConstRef { element_type : TypeId },
	MutRef { element_type : TypeId },
	ConstSlice { element_type : TypeId },
	MutSlice { element_type : TypeId },
	Array { element_type : TypeId, length : usize },
	Struct { fields : Box<[StructField]>, byte_alignment : Option<usize>, byte_size : Option<usize> },
	Tuple { fields : Box<[TypeId]> },

	//Scoped { scope : Scope },

	//Buffer,
	//Texture
	//GpuVertexWorkerState,
	//GpuFragmentWorkerState,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Funclet
{
	pub input_types : Box<[TypeId]>,
	#[serde(default)]
	pub input_resource_states : Box<[BTreeMap<Place, ResourceState>]>,
	pub execution_scope : Option<Scope>,
	pub output_types : Box<[TypeId]>,
	#[serde(default)]
	pub output_resource_states : Box<[BTreeMap<Place, ResourceState>]>,
	pub nodes : Box<[Node]>,
	pub tail_edge : TailEdge
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalCpuFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	// Scopes are always CPU (for now)
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
	// Scopes are always GPU (for now)
	pub output_types : Box<[TypeId]>,
	// Contains pipeline and single render pass state
	pub entry_point : String,
	pub resource_bindings : Box<[ExternalGpuFunctionResourceBinding]>,
	pub shader_module_content : ShaderModuleContent,
	//pub shader_module : usize,
}

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
	pub types : Arena<Type>,
	pub funclets : Arena<Funclet>,
	pub external_cpu_functions : Vec<ExternalCpuFunction>,
	pub external_gpu_functions : Vec<ExternalGpuFunction>,
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
