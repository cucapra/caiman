use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
//use bitflags::bitflags;
use crate::arena::Arena;

pub use crate::rust_wgpu_backend::ffi as ffi;

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
	Unbound,
	Bound,
	Encoded,
	Submitted,
	Ready,
	Dead
}

/*impl ResourceQueueStage
{
	fn next_stage(self) -> Self
	{
		use Self::*;
		match self
		{
			Unbound => Bound,
			Bound => Encoded,
			Encoded => Submitted,
			Submitted => Ready,
			Ready => Ready,
			Dead => Dead,
		}
	}
}*/

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
pub type ExternalTimestampId = usize;
pub type ExternalSpaceId = usize;
//pub type LocalMetaVariableId = usize;
pub type StorageTypeId = ffi::TypeId;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId{pub funclet_id : FuncletId, pub node_id : NodeId}

macro_rules! lookup_abstract_type {
	([$elem_type:ident]) => { Box<[lookup_abstract_type!($elem_type)]> };
	(Type) => { TypeId };
	(ImmediateI64) => { i64 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ExternalGpuFunctionId };
	(ValueFunction) => { ValueFunctionId };
	(Operation) => { OperationId };
	(RemoteOperation) => { RemoteNodeId };
	(Place) => { Place };
	(Funclet) => { FuncletId };
	(StorageType) => { StorageTypeId };
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueTag
{
	// Intended for scheduling purposes
	None,
	// These two are implementation-agnostic and are only allowed in external interfaces
	FunctionInput{function_id : ValueFunctionId, index : usize},
	FunctionOutput{function_id : ValueFunctionId, index : usize},
	// These are not, and are intended for funclets
	Operation{ remote_node_id : RemoteNodeId },
	Input{funclet_id : FuncletId, index : usize},
	Output{funclet_id : FuncletId, index : usize},
	Halt{index : usize}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimelineTag
{
	None, // Don't care
	Operation{ remote_node_id : RemoteNodeId },
	Input{ funclet_id : FuncletId, index : usize },
	Output{ funclet_id : FuncletId, index : usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type
{
	Integer { signed : bool, width : usize },

	Slot { storage_type : ffi::TypeId, queue_stage : ResourceQueueStage, queue_place : Place },
	SchedulingJoin {
		input_types : Box<[TypeId]>, 
		value_funclet_id : FuncletId,
		input_slots : HashMap<usize, SlotInfo>,
		input_fences : HashMap<usize, FenceInfo>,
		in_timeline_tag : TimelineTag,
	},
	Fence { queue_place : Place },
	Buffer { storage_place : Place },

	Agent { place : Place }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	// Common?
	Return { return_values : Box<[NodeId]> },
	Yield { funclet_ids : Box<[FuncletId]>, captured_arguments : Box<[NodeId]>, return_values : Box<[NodeId]> },
	Jump { join : NodeId, arguments : Box<[NodeId]> },

	// Scheduling only
	// Split value - what will be computed
	ScheduleCall { value_operation : RemoteNodeId, callee_funclet_id : FuncletId, callee_arguments : Box<[NodeId]>, continuation_join : NodeId },
	ScheduleSelect { value_operation : RemoteNodeId, condition : NodeId, callee_funclet_ids : Box<[FuncletId]>, callee_arguments : Box<[NodeId]>, continuation_join : NodeId },
	
	// Split time - when it will be computed
	// SyncFence { fence : NodeId, immediate_funclet : FuncletId, deferred_funclet : FuncletId, arguments : Box<[NodeId]>, continuation_join : NodeId },

	// Split space - where the computation will be observed
	AllocFromBuffer { buffer : NodeId, slot_count : usize, success_funclet_id : FuncletId, failure_funclet_id : FuncletId, arguments : Box<[NodeId]>, continuation_join : NodeId }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FuncletKind
{
	MixedImplicit,
	MixedExplicit,
	Value,
	ScheduleExplicit,
	Inline, // Adopts the constraints of the calling funclet
	Timeline
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
}

// Funclet-relative slot info goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotInfo
{
	pub value_tag : ValueTag,
	pub timeline_tag : TimelineTag, // marks the event that put the slot into its current state
	//pub queue_stage : ResourceQueueStage,
	//pub queue_place : Place,
	//pub resource_id : ...
	pub external_timestamp_id_opt : Option<ExternalTimestampId>,
	pub external_space_id_opt : Option<ExternalSpaceId>
}

// Funclet-relative join info goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinInfo
{
	// To do: Which subregions of resources are reserved by this join
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FenceInfo
{
	pub external_timestamp_id : ExternalTimestampId,
	pub timeline_tag : TimelineTag,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchedulingFuncletExtra
{
	pub value_funclet_id : FuncletId,
	pub input_slots : HashMap<usize, SlotInfo>,
	pub output_slots : HashMap<usize, SlotInfo>,
	pub input_fences : HashMap<usize, FenceInfo>,
	pub output_fences : HashMap<usize, FenceInfo>,
	//pub input_joins : HashMap<usize, JoinInfo>,
	//pub external_timestamps : BTreeSet<ExternalTimestampId>,
	//pub external_spaces : BTreeSet<ExternalSpaceId>,
	pub in_timeline_tag : TimelineTag,
	pub out_timeline_tag : TimelineTag,
	//pub default_join_type_id_opt : Option<TypeId>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct CompatibleValueFunctionKey
{
	pub value_function_id : ValueFunctionId,
	pub capture_count : usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueFuncletExtra
{
	// Value functions this funclet implements and the number of captures
	#[serde(default)]
	pub compatible_value_functions : BTreeSet<CompatibleValueFunctionKey>
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
	#[serde(default)]
	pub native_interface : ffi::NativeInterface,
	#[serde(default)]
	pub types : Arena<Type>,
	#[serde(default)]
	pub funclets : Arena<Funclet>,
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
