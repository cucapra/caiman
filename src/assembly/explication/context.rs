pub mod data_impls;
pub mod getters;
pub mod initializers;
pub mod mutators;
pub mod static_getters;

use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::table::Table;
use crate::ir;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Context<'context> {
    pub location: ast::RemoteNodeId,

    // holds the full program "as we go"
    program: DebugIgnore<&'context mut ast::Program>,

    // information found about a given value funclet
    spec_explication_data: HashMap<FuncletId, SpecFuncletData>,
    // information found about a given schedule funclet
    schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,

    // most recent scoped information found when constructing the current schedule
    // this represents a stack, so should be popped when scope ends
    scopes: Vec<ScheduleScopeData>,

    meta_data: MetaData,
}

// this information is static, and doesn't change as explication progresses
#[derive(Debug)]
struct SpecFuncletData {
    // map of node dependencies for scheduling
    node_dependencies: HashMap<NodeId, Vec<NodeId>>,

    // tailedge dependencies for scheduling
    tail_dependencies: Vec<NodeId>,

    // stores connections of which schedules refer to this value funclet
    connections: Vec<FuncletId>,
}

// NOTE: we use "available" here to mean "either not filled or not used yet"
// so basically partially defined holes that the explicator can use

// information held by an "available" allocation hole
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct AlloctionHoleInfo {
    pub ffi_type: Hole<FFIType>,
    pub place: Hole<ir::Place>,
}

// information held by a finished instantiation
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct ScheduledInstantiationInfo {
    pub funclet: FuncletId,
    pub node: NodeId,
    pub place: ir::Place,
    // specific check for differentiating between references and values for now
    // bit janky, should be generalized to the valid types probably?
    pub is_value: bool,
}

// classification of available operations on data
// this is mostly for documentation and error checking
// there isn't anything formal about these classifications
#[derive(Debug, Hash, Eq, PartialEq)]
enum Operation {
    // read from an allocation (e.g. to a value)
    Read,
    // write to an allocation
    Write,
    // copy between things of the same type
    Copy
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct OperationInfo {
    pub node: NodeId,
    pub funclet: FuncletId,
    pub operation: Operation
}

#[derive(Debug)]
struct ScheduleScopeData {
    // structure to manage the explication information for the current scope
    // the rule is more-to-less specific, then go up to the next scope out

    // map from location information to an instantiation (if one exists)
    instantiations: HashMap<ScheduledInstantiationInfo, NodeId>,

    // map from (optional) place and ffi type to "available" allocation slots
    // order of the slots doesn't formally matter, but for consistency in results, we use a vec
    available_allocations: HashMap<AlloctionHoleInfo, Vec<NodeId>>,

    // map from (optional) funclet and node to a collection of "available" operations
    // note that this is assumed that either the funclet or the nodeid must be None
    // note also that any node returned may still need to be explicated
    available_operations: HashMap<OperationInfo, Vec<NodeId>>,

    // most recently found multiline hole, if one exists in this scope
    // note that explication holes are named in corrections
    explication_hole: Option<NodeId>
}

// this information
#[derive(Debug)]
struct ScheduleFuncletData {
    // associated specification funclets
    value_funclet: FuncletId,
    timeline_funclet: FuncletId,
    spatial_funclet: FuncletId,

    // map from the scheduled allocations to what they are instantiating (if known)
    type_instantiations: HashMap<NodeId, RemoteNodeId>,
}

#[derive(Debug)]
struct MetaData {
    name_index: usize,
}
