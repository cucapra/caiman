use serde::ser::{SerializeStruct, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
//use bitflags::bitflags;
use crate::stable_vec::StableVec;

pub use crate::rust_wgpu_backend::ffi;

pub mod fusion;
pub mod validation;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Local,
    Cpu,
    Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceQueueStage {
    Unbound,
    Bound,
    Encoded,
    Submitted,
    Ready,
    Dead,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    I32(i32),
    I64(i64),
    U64(u64),
}

pub type ExternalFunctionId = ffi::ExternalFunctionId;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type OperationId = NodeId;
pub type TypeId = usize;
pub type PlaceId = usize;
pub type ValueFunctionId = usize;
pub type StorageTypeId = ffi::TypeId;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId {
    pub funclet_id: FuncletId,
    pub node_id: NodeId,
}

macro_rules! lookup_abstract_type {
	([$elem_type:ident]) => { Box<[lookup_abstract_type!($elem_type)]> };
	(Type) => { TypeId };
	(Immediate) => { Constant };
	(ImmediateI64) => { i64 };
	(ImmediateI32) => { i32 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalFunction) => { ExternalFunctionId };
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
    ($map:ident, $arg:ident : Operation) => {
        $map(*$arg)
    };
    ($map:ident, $arg:ident : [Operation]) => {
        $arg.iter().map(|op| $map(*op)).collect()
    };
    ($_map:ident, $arg:ident : $_arg_type:tt) => {
        $arg.clone()
    };
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
pub enum Tag {
    None,
    Node { node_id: usize },
    Input { index: usize },
    Output { index: usize },
    Halt { index: usize },
}

impl Tag {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct StaticBufferLayout {
    pub alignment_bits: usize,
    pub byte_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    NativeValue {
        storage_type: StorageTypeId,
    },
    //Integer { signed : bool, width : usize },

    // Scheduling
    Slot {
        storage_type: ffi::TypeId,
        queue_stage: ResourceQueueStage,
        queue_place: Place,
    },
    SchedulingJoin {
        /*input_types : Box<[TypeId]>,
        value_funclet_id : FuncletId,
        input_slots : HashMap<usize, SlotInfo>,
        input_fences : HashMap<usize, FenceInfo>,
        in_timeline_tag : TimelineTag,*/
    },
    Fence {
        queue_place: Place,
    },
    Buffer {
        storage_place: Place,
        static_layout_opt: Option<StaticBufferLayout>,
    },

    // Timeline
    Event {
        place: Place,
    },

    // Space
    BufferSpace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge {
    // Common?
    Return {
        return_values: Box<[NodeId]>,
    },
    Yield {
        external_function_id: ExternalFunctionId,
        yielded_nodes: Box<[NodeId]>,
        next_funclet: FuncletId,
        continuation_join: NodeId,
        arguments: Box<[NodeId]>,
    },
    Jump {
        join: NodeId,
        arguments: Box<[NodeId]>,
    },

    // Scheduling only
    // Split value - what will be computed
    ScheduleCall {
        value_operation: RemoteNodeId,
        callee_funclet_id: FuncletId,
        callee_arguments: Box<[NodeId]>,
        continuation_join: NodeId,
    },
    ScheduleSelect {
        value_operation: RemoteNodeId,
        condition: NodeId,
        callee_funclet_ids: Box<[FuncletId]>,
        callee_arguments: Box<[NodeId]>,
        continuation_join: NodeId,
    },

    // Split time - when it will be computed
    // SyncFence { fence : NodeId, immediate_funclet : FuncletId, deferred_funclet : FuncletId, arguments : Box<[NodeId]>, continuation_join : NodeId },

    // Split space - where the computation will be observed
    DynamicAllocFromBuffer {
        buffer: NodeId,
        arguments: Box<[NodeId]>,
        dynamic_allocation_size_slots: Box<[Option<NodeId>]>,
        success_funclet_id: FuncletId,
        failure_funclet_id: FuncletId,
        continuation_join: NodeId,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FuncletKind {
    Unknown,
    Value,
    ScheduleExplicit,
    Timeline,
    Spatial,
}

impl FuncletKind {
    fn default() -> Self {
        FuncletKind::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncletSpec {
    pub funclet_id_opt: Option<FuncletId>,
    pub input_tags: Box<[Tag]>,
    pub output_tags: Box<[Tag]>,
    #[serde(default = "Tag::default")]
    pub implicit_in_tag: Tag,
    #[serde(default = "Tag::default")]
    pub implicit_out_tag: Tag,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FuncletSpecBinding {
    None,
    Value {
        value_function_id_opt: Option<ValueFunctionId>,
    },
    ScheduleExplicit {
        value: FuncletSpec,
        spatial: FuncletSpec,
        timeline: FuncletSpec,
    },
}

impl FuncletSpecBinding {
    fn default() -> Self {
        FuncletSpecBinding::None
    }

    pub fn get_value_spec<'binding>(&'binding self) -> &'binding FuncletSpec {
        if let FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = self
        {
            value
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }

    pub fn get_timeline_spec<'binding>(&'binding self) -> &'binding FuncletSpec {
        if let FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = self
        {
            timeline
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }

    pub fn get_spatial_spec<'binding>(&'binding self) -> &'binding FuncletSpec {
        if let FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = self
        {
            spatial
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Funclet {
    #[serde(default = "FuncletKind::default")]
    pub kind: FuncletKind,
    #[serde(default = "FuncletSpecBinding::default")]
    pub spec_binding: FuncletSpecBinding,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
    pub nodes: Box<[Node]>,
    pub tail_edge: TailEdge,
}

fn ordered_map<'a, T>(map: &HashMap<usize, T>) -> Vec<(&usize, &T)> {
    let mut elements = Vec::new();
    for key in map.keys().sorted() {
        // kinda sloppy, but gets the job done
        elements.push((key, map.get(key).unwrap()));
    }
    elements
}

// A function class is just an equivalence class over functions that behave identically for some user-defined definition of identical
// A schedule can substitute a call to it for an implementation iff that implementation is associated with the function class
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionClass {
    pub name_opt: Option<String>,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
    // A hint about what funclet the explicator can use to instantiate this class
    // This doesn't need to exist for caiman to compile if everything is already explicit
    pub default_funclet_id: Option<FuncletId>,
    // The external functions that implement this function
    pub external_function_ids: BTreeSet<ExternalFunctionId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub entry_funclet: FuncletId,
    pub effect_id_opt: Option<ffi::EffectId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Program {
    #[serde(default)]
    pub native_interface: ffi::NativeInterface,
    #[serde(default)]
    pub types: StableVec<Type>,
    #[serde(default)]
    pub funclets: StableVec<Funclet>,
    #[serde(default)]
    pub function_classes: StableVec<FunctionClass>,
    #[serde(default)]
    pub pipelines: Vec<Pipeline>,
}

impl Program {
    pub fn new() -> Self {
        Default::default()
    }
}

// Hall of shame but mostly deprecated name

pub type ValueFunction = FunctionClass;
pub type ValueTag = Tag;
pub type TimelineTag = Tag;
pub type SpatialTag = Tag;

// Will phase this out, so don't depend on it
#[derive(Debug, Clone)]
pub struct SchedulingTagSet {
    //#[serde(default = "ValueTag::default")]
    pub value_tag: ValueTag,
    //#[serde(default = "TimelineTag::default")]
    pub timeline_tag: TimelineTag,
    //#[serde(default = "SpatialTag::default")]
    pub spatial_tag: SpatialTag,
}
