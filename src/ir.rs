use std::collections::HashMap;

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
}

#[derive(Debug)]
enum Type
{
	Integer,
	Float,
	Buffer,
	Texture,
	//GpuVertexWorkerState,
	//GpuFragmentWorkerState,
}

#[derive(Debug)]
enum Node
{
	// Core
	Phi { index : usize },
	Extract { node : NodeId, index : usize },

	// High Level Coordinator Language
	CallExternalCpu { external_function : ExternalCpuFunctionId, arguments : Box<[NodeId]> },
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
	CallExternalGpuCompute { external_function : ExternalGpuFunctionId, arguments : Box<[NodeId]>, dimensions : [NodeId; 3] },

	//

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

#[derive(Debug)]
enum TailEdge
{
	Return { return_values : Box<[NodeId]> },
	//Jump { join : usize, arguments : Box<[usize]> },
	//BranchIf { condition : NodeId, true_case : FuncletId, true_arguments : Box<[NodeId]>, false_case : FuncletId, false_arguments : Box<NodeId> },
	//Call { callee_block_id : usize, callee_block_arguments : Box<[usize]>, continuation_block_id : usize, continuation_context_values : Box<[usize]> },
	//CallGpuCoordinator { callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
	//CallGpuWorker{ callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
}

#[derive(Debug)]
struct Funclet
{
	input_types : Box<[usize]>,
	output_types : Box<[usize]>,
	nodes : Box<[Node]>,
	tail_edge : TailEdge
}

#[derive(Debug)]
struct ExternalCpuFunction
{

}

#[derive(Debug)]
struct ExternalGpuFunction
{
	// Contains pipeline and single render pass state
}

#[derive(Debug)]
struct Program
{
	types : HashMap<usize, Type>,
	funclets : HashMap<usize, Funclet>,
	external_cpu_functions : Vec<ExternalCpuFunction>,
	external_gpu_functions : Vec<ExternalGpuFunction>,
	entry_funclet : Option<FuncletId>
}

mod transformation
{
	use crate::ir::*;

	// Step 1: Separate GPU and CPU parts of coordinator functions
	// Step 2: Generate bindgroups?
}

mod evaluation
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
}
