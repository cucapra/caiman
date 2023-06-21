pub mod data_impls;
pub mod getters;
pub mod mutators;

use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::table::Table;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};

pub struct Context<'context> {
    pub location: LocationNames,

    // holds the full program "as we go"
    program: &'context mut ast::Program,

    // information found about a given value funclet
    value_explication_data: HashMap<FuncletId, ValueFuncletData>,
    // information found about a given schedule funclet
    schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,

    meta_data: MetaData,
}

#[derive(Debug)]
pub struct LocationNames {
    // a bit confusing, but unwrapping holes is annoying
    pub funclet_name: FuncletId,
    pub node_name: NodeId,
}

#[derive(Debug)]
struct ValueFuncletConnections {
    // stores connections of what refers to this value funclet
    schedule_funclets: Vec<FuncletId>,
    timeline_funclets: Vec<FuncletId>,
    spatial_funclets: Vec<FuncletId>,
}

#[derive(Debug)]
struct ValueExplicationInformation {
    // explication locations are in the scheduling world
    // maps from this node to the places it's been allocated on
    scheduled_allocations: HashMap<FuncletId, NodeId>,

    // indicates whether this operation has been written yet
    // used primarily to add operations when needed
    written: bool,
}

#[derive(Debug)]
struct ValueFuncletData {
    pub connections: ValueFuncletConnections,

    // information about allocated value elements
    explication_information: HashMap<NodeId, ValueExplicationInformation>,

    // map from call index to output name for each call
    call_outputs: HashMap<NodeId, HashMap<usize, NodeId>>,
}

#[derive(Debug)]
struct ScheduleFuncletData {
    // associated value funclet
    value_funclet: FuncletId,
    // map from the variables available to which node they are allocated
    allocations: HashMap<NodeId, NodeId>,
    // list of explication holes found, by index
    explication_holes: Vec<usize>,
}

#[derive(Debug)]
struct MetaData {
    name_index: usize,
}
