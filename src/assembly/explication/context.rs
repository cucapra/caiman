pub mod data_impls;
pub mod getters;
pub mod initializers;
mod internal_mutators;
pub mod static_getters;
pub mod mutators;

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

#[derive(Debug)]
struct ScheduleFuncletData {
    // associated specification funclets
    value_funclet: FuncletId,
    timeline_funclet: FuncletId,
    spatial_funclet: FuncletId,

    // map from the scheduled allocations to what they are instantiating (if known)
    type_instantiations: HashMap<NodeId, RemoteNodeId>,
}

// NOTE: we use "available" here to mean "either not filled or not used yet"
// so basically partially defined holes that the explicator can use

// information held by an "available" allocation hole
#[derive(Debug, Hash, Eq, PartialEq)]
struct AlloctionHoleInfo {
    pub ffi_type: Hole<FFIType>,
    pub place: Hole<ir::Place>,
}

// information held by a finished instantiation
#[derive(Debug, Hash, Eq, PartialEq)]
struct ScheduledInstantiationInfo {
    pub funclet: FuncletId,
    pub node: NodeId,
    pub place: ir::Place,
    // specific check for differentiating between references and values for now
    // bit janky, should be generalized to the valid types probably?
    pub is_value: bool,
}

// could restrict by language, but this works for now
macro_rules! make_op_codes {
    ($($_lang:ident $name:ident ($($_arg:ident : $_arg_type:tt,)*) -> $_output:ident;)*) => {
        #[derive(Debug, Hash, Eq, PartialEq)]
        pub enum OpCode {
            $($name,)*
        }
    };
}

with_operations!(make_op_codes);

#[derive(Debug)]
struct ScheduleScopeData {
    // structure to manage the explication information for the current scope
    // the rule is more-to-less specific, then go up to the next scope out
    name: FuncletId,

    // map from location information to an instantiation (if one exists)
    instantiations: HashMap<ScheduledInstantiationInfo, NodeId>,

    // map from operation code to a vector of "available" operations
    // note also that any node returned will still need explication
    // once a node is returned, it's removed from the vector
    // note that an unfinished allocation can be readded later
    available_operations: HashMap<OpCode, Vec<NodeId>>,

    // most recently found multiline hole, if one exists in this scope
    // note that explication holes are named in corrections
    explication_hole: Option<NodeId>
}

#[derive(Debug)]
struct MetaData {
    name_index: usize,
}
