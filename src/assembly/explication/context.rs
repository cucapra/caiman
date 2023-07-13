pub mod data_impls;
pub mod getters;
pub mod mutators;
pub mod static_getters;
pub mod initializers;

use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::table::Table;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};
use crate::ir;

pub struct Context<'context> {
    pub location: ast::RemoteNodeId,

    // holds the full program "as we go"
    program: &'context mut ast::Program,

    // information found about a given value funclet
    value_explication_data: HashMap<FuncletId, ValueFuncletData>,
    // information found about a given schedule funclet
    schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,

    meta_data: MetaData,
}

#[derive(Debug)]
struct ScheduleFuncletConnections {
    // stores connections of what refers to this value funclet
    schedule_funclets: Vec<FuncletId>,
}

#[derive(Debug)]
struct NodeExplicationInformation {
    // explication locations are in the scheduling world
    // maps from this node to the places it's been allocated on
    scheduled_allocations: HashMap<FuncletId, NodeId>,

    // indicates which operations were scheduled on this node (if any)
    scheduled_operations: HashMap<FuncletId, NodeId>,
}

#[derive(Debug)]
struct SpecFuncletData {
    pub connections: ScheduleFuncletConnections,

    // information about allocated value elements
    explication_information: HashMap<NodeId, NodeExplicationInformation>,

    // map from call index to output name for each call (if appropriate)
    call_outputs: HashMap<NodeId, HashMap<usize, NodeId>>,
}

#[derive(Debug)]
struct NodeAllocations {
    // map from location to established allocations
    known_allocations: HashMap<ir::Place, NodeId>
}

#[derive(Debug)]
struct ScheduleFuncletData {
    // associated specification funclets
    value_funclet: FuncletId,
    timeline_funclet: FuncletId,
    spatial_funclet: FuncletId,

    // map from the spec variables to information about their allocations
    allocations: HashMap<NodeId, NodeAllocations>,

    // list of explication holes found, by name
    // note that explication holes are named in corrections
    explication_holes: Vec<NodeId>,
}

#[derive(Debug)]
struct MetaData {
    name_index: usize,
}
