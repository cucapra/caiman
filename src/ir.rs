use serde::ser::{SerializeStruct, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
//use bitflags::bitflags;
use crate::stable_vec::StableVec;
use bitflags::bitflags;

/*bitflags! {
    //Copy, Clone, PartialEq, Eq, Hash
    #[derive(Serialize, Deserialize)]
    pub struct BufferFlags : u32 {
        const MAP_READ = 0b1;
        const MAP_WRITE = 0b10;
        const COPY_SRC = 0b100;
        const COPY_DST = 0b1000;
        const STORAGE = 0b10000000;
    }
}*/

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferFlags {
    #[serde(default)]
    pub map_read: bool,
    #[serde(default)]
    pub map_write: bool,
    #[serde(default)]
    pub copy_src: bool,
    #[serde(default)]
    pub copy_dst: bool,
    #[serde(default)]
    pub storage: bool,
    #[serde(default)]
    pub uniform: bool,
}

impl BufferFlags {
    pub fn new() -> Self {
        Self {
            map_read: false,
            map_write: false,
            copy_src: false,
            copy_dst: false,
            storage: false,
            uniform: false,
        }
    }

    pub fn or(&self, other: &Self) -> Self {
        Self {
            map_read: self.map_read | other.map_read,
            map_write: self.map_write | other.map_write,
            copy_src: self.copy_src | other.copy_src,
            copy_dst: self.copy_dst | other.copy_dst,
            storage: self.storage | other.storage,
            uniform: self.uniform | other.uniform,
        }
    }
}

impl std::fmt::Display for BufferFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BufferFlags[");
        if self.map_read {
            write!(f, "map_read, ");
        }
        if self.map_write {
            write!(f, "map_write, ");
        }
        if self.copy_src {
            write!(f, "copy_src, ");
        }
        if self.copy_dst {
            write!(f, "copy_dst, ");
        }
        if self.storage {
            write!(f, "storage, ");
        }
        if self.uniform {
            write!(f, "uniform, ");
        }
        write!(f, "]");
        Ok(())
    }
}

pub use crate::rust_wgpu_backend::ffi;

pub mod analysis;
#[cfg(feature = "fusion")]
pub mod fusion;
pub mod validation;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Local,
    Cpu,
    Gpu,
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Place::Local => "Local",
            Place::Cpu => "CPU",
            Place::Gpu => "GPU",
        });
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    I32(i32),
    I64(i64),
    U64(u64),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constant::I32(c) => {write!(f, "{}i32", c);}
            Constant::I64(c) => {write!(f, "{}i64", c);},
            Constant::U64(c) => {write!(f, "{}u64", c);},
        };
        Ok(())
    }
}

pub type ExternalFunctionId = ffi::ExternalFunctionId;
pub type FuncletId = usize;
pub type NodeId = usize;
pub type OperationId = NodeId;
pub type TypeId = usize;
pub type PlaceId = usize;
pub type ValueFunctionId = usize;
pub type FunctionClassId = ValueFunctionId;
pub type StorageTypeId = ffi::TypeId;

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
	(RemoteOperation) => { Quotient };
	(Place) => { Place };
	(Funclet) => { FuncletId };
	(StorageType) => { StorageTypeId };
    (BufferFlags) => { BufferFlags };
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
pub enum Quotient {
    None,
    Node {
        /// Node id
        node_id: usize,
    },
    Input {
        index: usize,
    },
    /// Output quotient holding the index into the funclet's output array
    Output {
        /// The index into the funclet's output array
        index: usize,
    },
}

impl Default for Quotient {
    fn default() -> Self {
        Self::None
    }
}

// positive = have, negative = need, neutral = paired
// Encodes the "sign" of the data (this can be made more formal categorically as the interaction of adjunction pairs with a chirality structure)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Flow {
    Dead,   // Does not transform, can only be discarded and not read
    Usable, // Transforms forwards with state changes
    Need,   // Transforms backwards with state changes
    Saved,  // Usable has met Need and associated state will not change, but can still be read
}

// Save is a zero value that cannot be discarded or duplicated
// Dead is a zero value that can only be discarded or duplicated
// Usable is a nonzero (owning) value that can only be read and discarded
// Need is a nonzero (owning) value that can only be written and duplicated

impl Flow {
    pub fn is_droppable(&self) -> bool {
        match self {
            Self::Dead => true,
            Self::Usable => true,
            Self::Need => false,
            Self::Saved => false,
        }
    }

    pub fn is_duplicable(&self) -> bool {
        match self {
            Self::Dead => false,
            Self::Usable => false,
            Self::Need => true,
            Self::Saved => false,
        }
    }

    pub fn is_readable(&self) -> bool {
        match self {
            Self::Dead => false,
            Self::Usable => true,
            Self::Need => false,
            Self::Saved => true,
        }
    }

    pub fn is_neutral(&self) -> bool {
        match self {
            Self::Dead => true,
            Self::Usable => false,
            Self::Need => false,
            Self::Saved => true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    pub quot: Quotient, // What a given value maps to in a specification
    pub flow: Flow,     // How this value transforms relative to the specification
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            quot: Quotient::None,
            flow: Flow::Usable,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct StaticBufferLayout {
    pub alignment_bits: usize,
    pub byte_size: usize,
}

impl StaticBufferLayout {
    pub fn alloc_static(
        &mut self,
        native_interface: &ffi::NativeInterface,
        storage_type: StorageTypeId,
    ) {
        // To do check alignment compatibility
        let storage_size = native_interface.calculate_type_byte_size(storage_type);
        let alignment_bits = native_interface.calculate_type_alignment_bits(storage_type);
        let starting_alignment_offset = 1usize << self.alignment_bits;
        let additional_alignment_offset = if alignment_bits > self.alignment_bits {
            let alignment_offset = 1usize << alignment_bits;
            alignment_offset - starting_alignment_offset
        } else {
            0usize
        };
        let total_byte_size = storage_size + additional_alignment_offset;

        assert!(self.byte_size >= total_byte_size);
        self.byte_size -= total_byte_size;
        self.alignment_bits =
            (total_byte_size + starting_alignment_offset).trailing_zeros() as usize;
    }

    pub fn split_static(&mut self, native_interface: &ffi::NativeInterface, size: usize) -> Self {
        let predecessor_static_layout = Self {
            byte_size: size,
            alignment_bits: self.alignment_bits,
        };

        assert!(self.byte_size >= size);
        self.byte_size -= size;
        let starting_alignment_offset = 1usize << self.alignment_bits;
        self.alignment_bits = (size + starting_alignment_offset).trailing_zeros() as usize;

        return predecessor_static_layout;
    }

    pub fn merge_static_left(
        &mut self,
        native_interface: &ffi::NativeInterface,
        predecessor_static_layout: Self,
    ) {
        self.byte_size += predecessor_static_layout.byte_size;
        self.alignment_bits = predecessor_static_layout.alignment_bits;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    // Common
    NativeValue {
        storage_type: StorageTypeId,
    },

    // Value only
    //Integer { signed : bool, width : usize },

    // Scheduling only
    Ref {
        storage_type: ffi::TypeId,
        storage_place: Place,
        buffer_flags: BufferFlags,
    },
    Fence {
        queue_place: Place,
    },
    Buffer {
        storage_place: Place,
        static_layout_opt: Option<StaticBufferLayout>,
        flags: BufferFlags,
    },
    Encoder {
        queue_place: Place,
    },

    // Timeline only
    Event,

    // Space only
    BufferSpace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge {
    // Common?
    Return {
        return_values: Box<[NodeId]>,
    },
    Jump {
        join: NodeId,
        arguments: Box<[NodeId]>,
    },

    // Scheduling only
    // Split value - what will be computed
    ScheduleCall {
        value_operation: Quotient,
        timeline_operation: Quotient,
        spatial_operation: Quotient,
        callee_funclet_id: FuncletId,
        callee_arguments: Box<[NodeId]>,
        continuation_join: NodeId,
    },
    ScheduleSelect {
        value_operation: Quotient,
        timeline_operation: Quotient,
        spatial_operation: Quotient,
        condition: NodeId,
        callee_funclet_ids: Box<[FuncletId]>,
        callee_arguments: Box<[NodeId]>,
        continuation_join: NodeId,
    },
    ScheduleCallYield {
        value_operation: Quotient,
        timeline_operation: Quotient,
        spatial_operation: Quotient,
        external_function_id: ExternalFunctionId,
        yielded_nodes: Box<[NodeId]>,
        continuation_join: NodeId,
    },

    // Split time - when it will be computed
    // SyncFence { fence : NodeId, immediate_funclet : FuncletId, deferred_funclet : FuncletId, arguments : Box<[NodeId]>, continuation_join : NodeId },

    // Split space - where the computation will be observed
    /*DynamicAllocFromBuffer {
        buffer: NodeId,
        arguments: Box<[NodeId]>,
        dynamic_allocation_size_slots: Box<[Option<NodeId>]>,
        success_funclet_id: FuncletId,
        failure_funclet_id: FuncletId,
        continuation_join: NodeId,
    },*/
    // Here for now as a type system debugging tool
    // Always passes type checking, but fails codegen
    DebugHole {
        // Scalar nodes
        inputs: Box<[NodeId]>,
        // Continuations
        //outputs : Box<[NodeId]>
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
    pub fn default() -> Self {
        FuncletKind::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncletSpec {
    pub funclet_id_opt: Option<FuncletId>,
    pub input_tags: Box<[Tag]>,
    //pub input_flows : Box<[Flow]>,
    pub output_tags: Box<[Tag]>,
    //pub output_flows : Box<[Flow]>,
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
    Timeline {
        function_class_id_opt: Option<FunctionClassId>,
    },
    ScheduleExplicit {
        value: FuncletSpec,
        timeline: FuncletSpec,
        spatial: FuncletSpec,
    },
}

impl FuncletSpecBinding {
    pub fn default() -> Self {
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
    #[serde(default)]
    pub name_opt: Option<String>,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
    // A hint about what funclet the explicator can use to instantiate this class
    // This doesn't need to exist for caiman to compile if everything is already explicit
    #[serde(default)]
    pub default_funclet_id: Option<FuncletId>,
    // The external functions that implement this function
    #[serde(default)]
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
