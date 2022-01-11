use std::collections::HashMap;
use std::default::Default;
//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
use bitflags::bitflags;
use crate::arena::Arena;

/*#[derive(Serialize, Deserialize, Debug, Default)]
struct Scope
{
	//const None = 0b0;
	Cpu,
	Gpu,
}*/

/*
Scope works like a union type.
Constant is the bottom of the lattice and CpuOrGpu is the top.
Unknown is unknown.
*/
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Scope
{
	//Unknown,
	//Constant,
	Cpu,
	Gpu,
	//CpuOrGpu
}

bitflags!
{
	#[derive(Serialize, Deserialize, Default)]
	pub struct ScopeSet : u32
	{
		//const None = 0b0;
		const Cpu = 0b1;
		const GpuCoordinator = 0b10;
		const GpuWorker = 0b100;
		const Gpu = Self::GpuCoordinator.bits | Self::GpuWorker.bits;
	}
}

/*
#[derive(Debug)]
struct ExternalCpuFunctionId
{
	id : usize
}

#[derive(Debug)]
struct ExternalGpuFunctionId
{
	id : usize
}

#[derive(Debug)]
struct FuncletId
{
	id : usize
}

#[derive(Debug)]
struct NodeId
{
	id : usize
}

#[derive(Debug)]
struct TypeId
{
	id : usize
}*/

pub type ExternalCpuFunctionId = usize;
pub type ExternalGpuFunctionId = usize;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type TypeId = usize;

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

	Buffer,
	Texture
	//GpuVertexWorkerState,
	//GpuFragmentWorkerState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Node
{
	// Core
	Phi { index : usize },
	ConstantInteger (i64, TypeId),
	ConstantUnsignedInteger (u64, TypeId),
	ComputedResult { node_ids : Box<[NodeId]> }, // Tuple produced as a result of inlining
	ExtractResult { node_id : NodeId, index : usize },
	//ReadBuffer { node_id : NodeId, type_id : TypeId, byte_offset : usize },

	// Scopes
	GpuTaskStart{ local_variable_node_ids : Box<[NodeId]>, gpu_resident_node_ids : Box<[NodeId]> },
	GpuTaskEnd{ task_node_id : NodeId, local_variable_node_ids : Box<[NodeId]>, gpu_resident_node_ids : Box<[NodeId]> },

	// High Level Coordinator Language
	CallExternalCpu { external_function_id : ExternalCpuFunctionId, arguments : Box<[NodeId]> },
	CallExternalGpuRaster
	{
		external_function : ExternalGpuFunctionId,
		arguments : Box<[NodeId]>,
		// These can be set via GPU state
		vertex_count : NodeId,
		instance_count : NodeId,
		first_vertex : NodeId,
		first_instance : NodeId,
		// Not a complete list
		// Setting these with GPU state will force a GPU -> CPU sync
		viewport_x : NodeId,
		viewport_y : NodeId,
		viewport_width : NodeId,
		viewport_height : NodeId,
		viewport_min_depth : NodeId,
		viewport_max_depth : NodeId,
		scissor_rect_x : NodeId,
		scissor_rect_y : NodeId,
		scissor_rect_width : NodeId,
		scissor_rect_height : NodeId,
		blend_constant : NodeId,
		stencil_reference : NodeId
	},
	CallExternalGpuCompute { external_function_id : ExternalGpuFunctionId, arguments : Box<[NodeId]>, dimensions : [NodeId; 3] },

	// CPU, GPU Coordinator, GPU Worker split
	CallGpuCoordinator { funclet_id : FuncletId, arguments : Box<[NodeId]> },
	//CallGpuCoordinatorAndCpuAsync { gpu_funclet_id : FuncletId, gpu_arguments : Box<[NodeId]>, cpu_funclet_id : FuncletId, cpu_arguments },

	//Call { callee : FuncletId, arguments : Box<[NodeId]> },
	// CPU Only
	// GPU Coordinator Only
	//DispatchCompute { kernel : FuncletId, dimensions : [NodeId; 3], arguments : Box<[NodeId]> },
	//CopyBuffer { destination : NodeId, source : NodeId, destination_offset : NodeId, source_offset : NodeId, size : NodeId },
	// GPU Kernel Worker Only
	// GPU Vertex Worker Only
	//OutputVertex { state : NodeId, coords : [NodeId; 4] }
	// GPU Fragment Worker Only
	//OutputFragment { state : NodeId, coords : [NodeId; 4] }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	Return { return_values : Box<[NodeId]> },
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
	//input_scopes : Box<[ScopeSet]>,
	pub execution_scope : Option<Scope>,
	pub output_types : Box<[TypeId]>,
	//output_scopes : Box<[ScopeSet]>,
	pub nodes : Box<[Node]>,
	pub tail_edge : TailEdge
}

impl Funclet
{
	//fn is_gpu_executable()
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
	pub funclets : HashMap<usize, Funclet>,
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

/*
	/*struct FuncletLegalizationState
	{
		//is_gpu_compute_worker_legal : bool,
		legal_executor_set : ExecutorSet,
	}

	struct LegalizationState
	{
		remapped_funclets : HashMap<FuncletId, FuncletId>,
		funclet_states : HashMap<FuncletId, FuncletLegalizationState>,
	}*/

	fn legalize(state : &mut LegalizationState, program : &mut Program, funclet_id : FuncletId) -> FuncletId
	{
		/*if let Some(&new_funclet_id) = state.remapped_funclets.get(&funclet_id)
		{
			return new_funclet_id;
		}

		if state.funclet_states.contains_key(&funclet_id)
		{
			return funclet_id;
		}*/

		let fullExectorSet = ExecutorSet::Cpu | ExecutorSet::GpuCoordinator;

		let mut node_executor_sets  = HashMap::<NodeId, ExecutorSet>::new();

		let funclet = & program.funclets[&funclet_id];
		for current_node_id in 0 .. funclet.nodes.len()
		{
			let node = & funclet.nodes[current_node_id];
			let executor_set = match node
			{
				Node::Phi { index } => fullExectorSet,
				Node::Extract { node_id, index } => *node_executor_sets.get(node_id).unwrap(),
				ReadBuffer { node_id, type_id, byte_offset } => *node_executor_sets.get(node_id).unwrap(),
				CallExternalCpu { external_function_id, arguments } => ExecutorSet::Cpu,
				CallExternalGpuCompute { external_function_id, arguments } => ExecutorSet::Cpu,
				_ => ExecutorSet::empty() //CallExternalCpu { _ }
			};
			node_executor_sets.insert(current_node_id, executor_set);
		}

		panic!("Unfinished function");
		return 0;
	}
*/

/*od evaluation
{
	use crate::ir::*;

	enum Value
	{
		None
	}

	struct Context<'program>
	{
		program : & 'program Program,
	}

	impl<'program> Context<'program>
	{
		fn new(program : & 'program Program) -> Self
		{
			Self { program : program }
		}
	}
}*/
