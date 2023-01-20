use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use clap::App;
use crate::{ir, frontend};
use crate::ir::ffi;
use crate::ir::{Place, ResourceQueueStage, PlaceId};
use crate::arena::Arena;
use serde_derive::{Serialize, Deserialize};

// Parser "AST"

pub type ExternalCpuFunctionId = String;
pub type ExternalGpuFunctionId = String;
pub type FuncletId = String;
pub type NodeId = String;
pub type OperationId = NodeId;
pub type TypeId = String;
pub type ValueFunctionId = String;
pub type StorageTypeId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteNodeId{pub funclet_id : FuncletId, pub node_id : NodeId}

// Super Jank, but whatever

macro_rules! lookup_abstract_type_parser {
	([$elem_type:ident]) => { Box<[lookup_abstract_type_parser!($elem_type)]> };
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

macro_rules! map_parser_refs {
	// When mapping referenced nodes, we only care about mapping the Operation types,
	// since those are the actual references.
	($map:ident, $arg:ident : Operation) => {$map(*$arg)};
	($map:ident, $arg:ident : [Operation]) => {
		$arg.iter().map(|op| $map(*op)).collect()
	};
	($_map:ident, $arg:ident : $_arg_type:tt) => {$arg.clone()};
}

macro_rules! make_parser_nodes {
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
		make_parser_nodes! {
			@ $map { $($rest)* } ->
			($($fields)* $name,),
			($($mapper)* Self::$name => Self::$name,)
		}
	};
	(@ $map:ident {$name:ident ($($arg:ident : $arg_type:tt,)*), $($rest:tt)*} -> ($($fields:tt)*), ($($mapper:tt)*)) => {
		make_parser_nodes! {
			@ $map { $($rest)* } ->
			($($fields)* $name { $($arg: lookup_abstract_type_parser!($arg_type)),* },),
			($($mapper)* Self::$name { $($arg),* } => Self::$name {
				$($arg: map_parser_refs!($map, $arg : $arg_type)),*
			},)
		}
	};
	($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
		make_parser_nodes! { @ f {$($name ($($arg : $arg_type,)*),)*} -> (), () }
	};
}

with_operations!(make_parser_nodes);

#[derive(Debug)]
pub enum AnyTag {
    ValueTag(ir::ValueTag),
    TimelineTag(ir::TimelineTag),
    SpatialTag(ir::SpatialTag)
}

#[derive(Debug)]
pub enum Type {
	FFI(ffi::Type),
	IR(String)
}

#[derive(Debug)]
pub enum Value {
	ID(String),
    FunctionLoc(RemoteNodeId),
    VarName(String),
    FnName(String),
    Type(Type),
    Place(ir::Place),
    Stage(ir::ResourceQueueStage),
    Tag(AnyTag),
}

#[derive(Debug)]
pub enum DictValue {
	Raw(Value),
	List(Vec<Value>),
	Dict(UncheckedDict)
}

#[derive(Debug)]
pub struct DictPair {
	pub key : Value,
	pub value : DictValue
}

pub type UncheckedDict = Vec<DictPair>;

#[derive(Debug)]
pub struct IRType {
	pub id : usize,
	pub event : bool,
	pub type_name : String,
	pub data : UncheckedDict
}

#[derive(Debug)]
pub enum TypeDecl {
    FFI(ffi::Type),
    IR(IRType)
}

#[derive(Debug)]
pub struct Types {
    pub ffi_types: Arena<ffi::Type>,
	pub ir_types: Arena<IRType>
}

#[derive(Debug)]
pub struct Var {
	pub id : usize
}

#[derive(Debug)]
pub struct Argument {
    pub typ : Type,
    pub name : String
}

#[derive(Debug)]
pub struct Funclet {
	pub id : usize,
	pub kind : ir::FuncletKind,
	pub ret : Type,
	pub args : Vec<Argument>,
	pub commands : Vec<Node>
}

#[derive(Debug)]
pub struct Funclets {
	pub external_cpu: Arena<ffi::ExternalCpuFunction>,
	pub external_gpu: Arena<ffi::ExternalGpuFunction>,
	pub caiman: HashMap<String, Funclet>
}

#[derive(Debug)]
pub struct Extras {
	pub value_funclet_extras: HashMap<ir::FuncletId, ir::ValueFuncletExtra>,
	pub scheduling_funclet_extras: HashMap<ir::FuncletId, ir::SchedulingFuncletExtra>
}

pub struct Pipeline {
	pub name : String,
	pub funclet : String
}

pub type Pipelines = Vec<Pipeline>;

pub struct Program {
    pub types : Types,
	pub funclets : Funclets,
	pub extras : Extras,
	pub pipelines : Pipelines
}