use std::collections::{HashMap, BTreeMap};
use std::default::Default;
use std::iter::{once, Once, Chain};

//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
//use bitflags::bitflags;
use crate::arena::Arena;
use crate::operations::{UnopKind, BinopKind};

pub mod utils;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place
{
	Local,
	Cpu,
	Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

macro_rules! lookup_abstract_type {
	([$elem_type:ident]) => { Box<[lookup_abstract_type!($elem_type)]> };
	(Type) => { TypeId };
	(ImmediateBool) => { bool };
	(ImmediateI64) => { i64 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ExternalGpuFunctionId };
	(ValueFunction) => { ValueFunctionId };
	(Operation) => { OperationId };
	(Place) => { Place };
	(Unop) => { UnopKind };
	(Binop) => { BinopKind };
}
macro_rules! map_refs {
	// When mapping referenced nodes, we only care about mapping the Operation types,
	// since those are the actual references.
	($map:ident, $arg:ident : Operation) => {$map(*$arg)};
	($map:ident, $arg:ident : [Operation]) => {
		$arg.iter().map(|op| $map(*op)).collect()
	};
	($_map:ident, $arg:ident : $_arg_type:tt) => {$arg.clone()};
}
macro_rules! make_nodes {
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
		make_nodes! {
			@ $map { $($rest)* } -> 
			($($fields)* $name,), 
			($($mapper)* Self::$name => Self::$name,)
		}
	};
	(@ $map:ident {$name:ident ($($arg:ident : $arg_type:tt,)*), $($rest:tt)*} -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		make_nodes! {
			@ $map { $($rest)* } -> 
			($($fields)* $name { $($arg: lookup_abstract_type!($arg_type)),* },), 
			($($mapper)* Self::$name { $($arg),* } => Self::$name { 
				$($arg: map_refs!($map, $arg : $arg_type)),*
			},)
		}
	};
	($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
		make_nodes! { @ f {$($name ($($arg : $arg_type,)*),)*} -> (), () }
	};
}
with_operations!(make_nodes);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructField
{
	pub name : String,
	pub type_id : TypeId,
	pub byte_offset : usize,
	pub byte_size : usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalValueTag
{
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Jump {
	pub target: FuncletId,
	pub args: Box<[NodeId]>
}
impl Jump {
	pub fn map_referenced_nodes(&self, mut f: impl FnMut(NodeId) -> NodeId) -> Self {
		Self { 
			target: self.target, 
			args: self.args.iter().map(|&id| f(id)).collect()
		}
	}
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	Return { return_values : Box<[NodeId]> },
	Jump(Jump),
	Branch { cond: NodeId, j0: Jump, j1: Jump },

	// invokes and waits on the gpu
	//ReturnWithGpuCoordinator { initial_return_values : Box<[NodeId]>, gpu_funclet_id : FuncletId, arguments : Box<[NodeId]> },
	//Wait { required_scope_set : ScopeSet, funclet_id : usize, arguments : Box<[usize]> }
	//Jump { join : usize, arguments : Box<[usize]> },
	//BranchIf { condition : NodeId, true_case : FuncletId, true_arguments : Box<[NodeId]>, false_case : FuncletId, false_arguments : Box<NodeId> },
	//Call { callee_block_id : usize, callee_block_arguments : Box<[usize]>, continuation_block_id : usize, continuation_context_values : Box<[usize]> },
	//CallGpuCoordinator { callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
	//CallGpuWorker{ callee_block_id : usize, callee_block_arguments : Box<[usize]>, join_block_id : usize, join_block_initial_arguments : Box<[usize]> },
}
impl TailEdge {
	pub fn for_each_referenced_node<F>(&self, mut f: F) where F: FnMut(NodeId) -> () {
		match self {
			Self::Return { return_values } => {
				return_values.iter().for_each(|&id| f(id));
			}
			Self::Jump ( Jump {args, ..} ) => {
				args.iter().for_each(|&id| f(id));
			}
			Self::Branch { cond, j0, j1, .. } => {
				f(*cond);
				j0.args.iter().for_each(|&id| f(id));
				j1.args.iter().for_each(|&id| f(id));
			}
		}
	}
	pub fn map_referenced_nodes<F>(&self, mut f: F) -> Self where F: FnMut(NodeId) -> NodeId {
		match self {
			Self::Return { return_values } => {
				let new_rvals: Vec<NodeId> = return_values.iter().map(|&id| f(id)).collect();
				Self::Return { return_values: new_rvals.into_boxed_slice() }
			}
			Self::Jump ( jump ) => {
				Self::Jump ( jump.map_referenced_nodes(f) )
			}
			Self::Branch { cond, j0, j1 } => {
				Self::Branch { 
					cond: f(*cond), 
					j0: j0.map_referenced_nodes(&mut f),
					j1: j1.map_referenced_nodes(&mut f)
				}
			}
		}
	}
	pub fn jumps(&self) -> impl Iterator<Item = &'_ Jump> {
		match self {
			Self::Return { .. } => TailJumps::Return,
			Self::Jump(jump) => TailJumps::Jump(once(jump)),
			Self::Branch { j0, j1, .. } => TailJumps::Branch(once(j0).chain(once(j1)))
		}
	}
	pub fn jumps_mut(&mut self) -> impl Iterator<Item = &'_ mut Jump> {
		match self {
			Self::Return { .. } => TailJumpsMut::Return,
			Self::Jump(jump) => TailJumpsMut::Jump(once(jump)),
			Self::Branch { j0, j1, .. } => TailJumpsMut::Branch(once(j0).chain(once(j1)))
		}
	}
}

enum TailJumps<'a> {
    Return,
    Jump(Once<&'a Jump>),
    Branch(Chain<Once<&'a Jump>, Once<&'a Jump>>),
}
impl<'a> Iterator for TailJumps<'a> {
    type Item = &'a Jump;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Return => None,
            Self::Jump(ref mut iter) => iter.next(),
            Self::Branch(ref mut iter) => iter.next(),
        }
    }
}
enum TailJumpsMut<'a> {
    Return,
    Jump(Once<&'a mut Jump>),
    Branch(Chain<Once<&'a mut Jump>, Once<&'a mut Jump>>),
}
impl<'a> Iterator for TailJumpsMut<'a> {
    type Item = &'a mut Jump;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Return => None,
            Self::Jump(ref mut iter) => iter.next(),
            Self::Branch(ref mut iter) => iter.next(),
        }
    }
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
pub enum PipelineKind {
	Function,
	Yield {
		// the ids of pipelines the host can choose to invoke next
		pipeline_ids: Box<[usize]>,
		// the indexes of the elements in the pipeline's return value which should be captured
		captured: Box<[usize]>,
		// the indexes of the elements in the pipeline's return value which should be returned
		returned: Box<[usize]>,
	}
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline
{
	pub name : String,
	pub entry_funclet : FuncletId,
	pub kind: PipelineKind
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

/// Represents an IR construct which depends on IR nodes.
#[derive(Debug, Clone, Copy)]
pub enum Dependent {
    /// Represents a [`ir::Node`](crate::ir) with the given node ID.
    Node(NodeId),
    /// Represents a [`ir::TailEdge`](crate::ir).
    Tail,
}
impl std::fmt::Display for Dependent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(id) => write!(f, "node (id: {id})"),
            Self::Tail => write!(f, "tail edge"),
        }
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
