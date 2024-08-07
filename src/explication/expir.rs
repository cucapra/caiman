use crate::ir;
use serde::ser::{SerializeStruct, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
//use serde::{Serialize, Deserialize};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
//use bitflags::bitflags;
use crate::stable_vec::StableVec;

use crate::explication::Hole;
pub use crate::rust_wgpu_backend::ffi;

// Explication AST
// identical to the "real" ir, but with holes everywhere
// should be macro'd out in principle
// but also I need something working

pub type Place = ir::Place;
pub type Constant = ir::Constant;
pub type BufferFlags = ir::BufferFlags;

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
	([$elem_type:ident]) => { Box<[Hole<lookup_abstract_type!($elem_type)>]> };
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
    (BufferFlags) => { ir::BufferFlags };
}

macro_rules! map_refs {
    // When mapping referenced nodes, we only care about mapping the Operation types,
    // since those are the actual references.
    ($map:ident, $arg:ident : Operation) => {
        $arg.clone().opt().map(|x| $map(x)).into()
    };
    ($map:ident, $arg:ident : [Operation]) => {
        $arg.as_ref().opt().map(|lst| {
            lst.iter()
                .map(|arg_hole| arg_hole.clone())
                .collect()
        }).into()
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
			($($fields)* $name { $($arg: Hole<lookup_abstract_type!($arg_type)>),* },),
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

pub type Quotient = crate::ir::Quotient;
pub type Flow = crate::ir::Flow;
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq
)]
pub struct Tag {
    pub quot: Quotient, // What a given value maps to in a specification
    pub flow: Hole<Flow>,     // How this value transforms relative to the specification
}

impl Default for Tag {
    fn default() -> Self {
        Self { quot: Default::default(), flow: Hole::Filled(crate::ir::Flow::Usable) }
    }
}

pub type StaticBufferLayout = crate::ir::StaticBufferLayout;
pub type Type = crate::ir::Type;

// TODO: hole macro
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TailEdge {
    // Common?
    Return {
        return_values: Hole<Box<[Hole<NodeId>]>>,
    },
    Jump {
        join: Hole<NodeId>,
        arguments: Hole<Box<[Hole<NodeId>]>>,
    },

    // Scheduling only
    // Split value - what will be computed
    ScheduleCall {
        value_operation: Hole<Quotient>,
        timeline_operation: Hole<Quotient>,
        spatial_operation: Hole<Quotient>,
        callee_funclet_id: Hole<FuncletId>,
        callee_arguments: Hole<Box<[Hole<NodeId>]>>,
        continuation_join: Hole<NodeId>,
    },
    ScheduleSelect {
        value_operation: Hole<Quotient>,
        timeline_operation: Hole<Quotient>,
        spatial_operation: Hole<Quotient>,
        condition: Hole<NodeId>,
        callee_funclet_ids: Hole<Box<[Hole<FuncletId>]>>,
        callee_arguments: Hole<Box<[Hole<NodeId>]>>,
        continuation_join: Hole<NodeId>,
    },
    ScheduleCallYield {
        value_operation: Hole<Quotient>,
        timeline_operation: Hole<Quotient>,
        spatial_operation: Hole<Quotient>,
        external_function_id: Hole<ExternalFunctionId>,
        yielded_nodes: Hole<Box<[Hole<NodeId>]>>,
        continuation_join: Hole<NodeId>,
    },
    // Here for now as a type system debugging tool
    // Always passes type checking, but fails codegen
    DebugHole {
        // Scalar nodes
        inputs: Box<[NodeId]>,
        // Continuations
        //outputs : Hole<Box<[Hole<NodeId>]>>
    },
}

pub type FuncletKind = ir::FuncletKind;

// TODO: macro
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncletSpec {
    pub funclet_id_opt: Option<FuncletId>,
    pub input_tags: Box<[Hole<Tag>]>,
    pub output_tags: Box<[Hole<Tag>]>,
    pub implicit_in_tag: Hole<Tag>,
    pub implicit_out_tag: Hole<Tag>,
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

// TODO: macro
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Funclet {
    #[serde(default = "FuncletKind::default")]
    pub kind: FuncletKind,
    #[serde(default = "FuncletSpecBinding::default")]
    pub spec_binding: FuncletSpecBinding,
    pub input_types: Box<[TypeId]>,
    pub output_types: Box<[TypeId]>,
    pub nodes: Box<[Hole<Node>]>,
    pub tail_edge: Hole<TailEdge>,
}

impl FuncletSpecBinding {
    pub fn default() -> Self {
        FuncletSpecBinding::None
    }
}

fn ordered_map<'a, T>(map: &HashMap<usize, T>) -> Vec<(&usize, &T)> {
    let mut elements = Vec::new();
    for key in map.keys().sorted() {
        // kinda sloppy, but gets the job done
        elements.push((key, map.get(key).unwrap()));
    }
    elements
}

pub type FunctionClass = ir::FunctionClass;
pub type Pipeline = ir::Pipeline;

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
