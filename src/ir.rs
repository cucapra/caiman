use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;
use serde::ser::{Serializer, SerializeStruct};
//use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
use itertools::Itertools;
//use bitflags::bitflags;
use crate::stable_vec::StableVec;

pub use crate::rust_wgpu_backend::ffi as ffi;

pub mod validation;
pub mod fusion;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant
{
	I32(i32),
	I64(i64),
	U64(u64),
}

pub type ExternalCpuFunctionId = usize;
pub type ExternalGpuFunctionId = usize;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type OperationId = NodeId;
pub type TypeId = usize;
pub type PlaceId = usize;
pub type ValueFunctionId = usize;
pub type StorageTypeId = ffi::TypeId;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId{pub funclet_id : FuncletId, pub node_id : NodeId}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PipelineYieldPointId(pub usize);

macro_rules! lookup_abstract_type {
	([$elem_type:ident]) => { Box<[lookup_abstract_type!($elem_type)]> };
	(Type) => { TypeId };
	(Immediate) => { Constant };
	(ImmediateI64) => { i64 };
	(ImmediateI32) => { i32 };
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
	None,
	Operation{ remote_node_id : RemoteNodeId },
	Input{ funclet_id : FuncletId, index : usize },
	Output{ funclet_id : FuncletId, index : usize },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpatialTag
{
	None,
	Operation{ remote_node_id : RemoteNodeId },
	Input{ funclet_id : FuncletId, index : usize },
	Output{ funclet_id : FuncletId, index : usize },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct StaticBufferLayout
{
	pub alignment_bits : usize,
	pub byte_size : usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type
{
	NativeValue { storage_type : StorageTypeId },
	//Integer { signed : bool, width : usize },

	// Scheduling
	Slot { storage_type : ffi::TypeId, queue_stage : ResourceQueueStage, queue_place : Place },
	SchedulingJoin {
		/*input_types : Box<[TypeId]>, 
		value_funclet_id : FuncletId,
		input_slots : HashMap<usize, SlotInfo>,
		input_fences : HashMap<usize, FenceInfo>,
		in_timeline_tag : TimelineTag,*/
	},
	Fence { queue_place : Place },
	Buffer { storage_place : Place, static_layout_opt : Option<StaticBufferLayout> },

	// Timeline
	Event { place : Place },

	// Space
	BufferSpace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge
{
	// Common?
	Return { return_values : Box<[NodeId]> },
	Yield { pipeline_yield_point_id : PipelineYieldPointId, yielded_nodes : Box<[NodeId]>, next_funclet : FuncletId, continuation_join : NodeId, arguments : Box<[NodeId]> },
	Jump { join : NodeId, arguments : Box<[NodeId]> },

	// Scheduling only
	// Split value - what will be computed
	ScheduleCall { value_operation : RemoteNodeId, callee_funclet_id : FuncletId, callee_arguments : Box<[NodeId]>, continuation_join : NodeId },
	ScheduleSelect { value_operation : RemoteNodeId, condition : NodeId, callee_funclet_ids : Box<[FuncletId]>, callee_arguments : Box<[NodeId]>, continuation_join : NodeId },

	// Split time - when it will be computed
	// SyncFence { fence : NodeId, immediate_funclet : FuncletId, deferred_funclet : FuncletId, arguments : Box<[NodeId]>, continuation_join : NodeId },

	// Split space - where the computation will be observed
	DynamicAllocFromBuffer { buffer : NodeId, arguments : Box<[NodeId]>, dynamic_allocation_size_slots : Box<[Option<NodeId>]>, success_funclet_id : FuncletId, failure_funclet_id : FuncletId, continuation_join : NodeId }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FuncletKind
{
	MixedImplicit,
	MixedExplicit,
	Value,
	ScheduleExplicit,
	Inline, // Adopts the constraints of the calling funclet
	Timeline,
	Spatial
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
}

// Funclet-relative slot info goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotInfo
{
	pub value_tag : ValueTag,
	pub timeline_tag : TimelineTag, // marks the event that put the slot into its current state
	pub spatial_tag : SpatialTag,
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
	pub timeline_tag : TimelineTag,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BufferInfo
{
	pub spatial_tag : SpatialTag,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SchedulingFuncletExtra
{
	pub value_funclet_id : FuncletId,
	//pub spatial_funclet_id : FuncletId,
	//pub temporal_funclet_id : FuncletId,
	pub input_slots : HashMap<usize, SlotInfo>,
	pub output_slots : HashMap<usize, SlotInfo>,
	pub input_fences : HashMap<usize, FenceInfo>,
	pub output_fences : HashMap<usize, FenceInfo>,
	pub input_buffers : HashMap<usize, BufferInfo>,
	pub output_buffers : HashMap<usize, BufferInfo>,
	//pub input_joins : HashMap<usize, JoinInfo>,

	// Applies to the computation itself
	pub in_timeline_tag : TimelineTag,
	pub out_timeline_tag : TimelineTag,
}

fn ordered_map<'a, T>(map : &HashMap<usize, T>) -> Vec<(&usize, &T)> {
	let mut elements = Vec::new();
	for key in map.keys().sorted() {
		// kinda sloppy, but gets the job done
		elements.push((key, map.get(key).unwrap()));
	}
	elements
}

impl serde::Serialize for SchedulingFuncletExtra {
	fn serialize<S>(& self, serializer : S) -> std::result::Result<<S as serde::Serializer>::Ok, <S as Serializer>::Error>
		where S : Serializer {
		let input_slots = ordered_map(&self.input_slots);
		let output_slots = ordered_map(&self.output_slots);
		let input_fences = ordered_map(&self.input_fences);
		let output_fences = ordered_map(&self.output_fences);
		let input_buffers = ordered_map(&self.input_buffers);
		let output_buffers = ordered_map(&self.output_buffers);

		let mut state = serializer.serialize_struct("SchedulingFucletExtra", 9)?;
		state.serialize_field("value_funclet_id", &self.value_funclet_id);
		state.serialize_field("input_states", &input_slots);
		state.serialize_field("output_states", &output_slots);
		state.serialize_field("input_fences", &input_fences);
		state.serialize_field("output_fences", &output_fences);
		state.serialize_field("input_buffers", &input_buffers);
		state.serialize_field("output_buffers", &output_buffers);
		state.serialize_field("in_timeline_tag", &self.in_timeline_tag);
		state.serialize_field("out_timeline_tag", &self.out_timeline_tag);
		state.end()
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct CompatibleValueFunctionKey
{
	pub value_function_id : ValueFunctionId
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueFuncletExtra
{
	// Value functions this funclet implements
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
	pub default_funclet_id : Option<FuncletId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline
{
	pub name : String,
	pub entry_funclet : FuncletId,
	#[serde(default)]
	pub yield_points : BTreeMap<PipelineYieldPointId, PipelineYieldPoint>
}

// Callee is permitted to change the location of slots within a buffer, the size of a space, and the timeline
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipelineYieldPoint
{
	pub name : String,
	pub yielded_types : Box<[TypeId]>,
	pub resuming_types : Box<[TypeId]>, // All value tags must be None (callee cannot change value)

	pub yielded_timeline_tag : TimelineTag,
	pub resuming_timeline_tag : TimelineTag,
	pub spatial_funclet_id : FuncletId,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Program
{
	#[serde(default)]
	pub native_interface : ffi::NativeInterface,
	#[serde(default)]
	pub types : StableVec<Type>,
	#[serde(default)]
	pub funclets : StableVec<Funclet>,
	#[serde(default)]
	pub value_functions : StableVec<ValueFunction>,
	#[serde(default)]
	pub pipelines : Vec<Pipeline>,
	#[serde(default)]
	pub value_funclet_extras : HashMap<FuncletId, ValueFuncletExtra>,
	#[serde(default)]
	pub scheduling_funclet_extras : HashMap<FuncletId, SchedulingFuncletExtra>,
}

impl serde::Serialize for Program {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<<S as serde::Serializer>::Ok, <S as Serializer>::Error>
		where S: Serializer {
		let value_funclet_extras = ordered_map(&self.value_funclet_extras);
		let scheduling_funclet_extras = ordered_map(&self.scheduling_funclet_extras);

		let mut state = serializer.serialize_struct("SchedulingFucletExtra", 9)?;
		state.serialize_field("native_interface", &self.native_interface);
		state.serialize_field("types", &self.types);
		state.serialize_field("funclets", &self.funclets);
		state.serialize_field("value_functions", &self.value_functions);
		state.serialize_field("pipelines", &self.pipelines);
		state.serialize_field("value_funclet_extras", &value_funclet_extras);
		state.serialize_field("scheduling_funclet_extras", &scheduling_funclet_extras);
		state.end()
	}
}

impl Program
{
	pub fn new() -> Self
	{
		Default::default()
	}
}