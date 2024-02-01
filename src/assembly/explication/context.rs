pub mod data_impls;
pub mod getters;
pub mod instate;
mod internal_mutators;
pub mod outstate;
pub mod static_getters;

use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::explication::util::*;
use crate::assembly::table::Table;
use crate::ir;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet, Deque};

#[derive(Debug, Clone)]
pub struct InState {
    // state information needs to be duplicated as we recurse

    // information found about a given schedule funclet
    schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,

    // most recent scoped information found when constructing the current schedule
    // this represents a stack, so should be popped when scope ends
    scopes: Vec<ScheduleScopeData>,
}

#[derive(Debug, Default)]
pub struct FuncletOutState {
    // The return type from explicating a single funclet
    
    // the allocations that still need to be concretized
    // we map from funclet _index_ to the type to allocate to make recursion easier
    // note that this necessarily refers to the current state
    allocation_requests: HashMap<StorageTypeId, usize>,

    // The types that still need explication for this scope
    // they (by default) will be filled in the most specific open slot
    to_fill: HashSet<Location>,

    // commands we've built on this particular funclet of the stack
    commands: Deque<ast::NamedCommand>
}

#[derive(Debug)]
pub struct StaticContext {
    // static information we work out before explicating
    // may be updated between explicating individual funclets

    // the entire original program, useful for looking things up
    // note that we are constructing a completely fresh program recursively
    // so the original program is not mutated
    program: ast::Program,

    // information about each type
    type_declarations: HashMap<String, LocalTypeDeclaration>,

    // information found about a given spec funclet
    spec_explication_data: HashMap<FuncletId, SpecFuncletData>
}

#[derive(Debug)]
struct LocalTypeDeclaration {
    // if this (local) type has an associated place
    pub place: Option<ir::Place>,

    // if this type has an associated FFI Type
    pub ffi: Option<FFIType>,
}

// this information is static, and doesn't change as explication progresses
#[derive(Debug)]
struct SpecFuncletData {
    // map of direct node dependencies for scheduling
    node_dependencies: HashMap<NodeId, Vec<NodeId>>,

    // type information derived from dependencies
    // technically this can be derived from the nodes lazily
    // but it's slow enough recursing repeatedly when explicating to wanna do it up-front I guess
    deduced_types: HashMap<NodeId, Vec<ast::TypeId>>,

    // tailedge dependencies for scheduling
    tail_dependencies: Vec<NodeId>,

    // connections of which schedules refer to this spec funclet
    // this can be modified when adding a new schedule
    connections: Vec<FuncletId>,
}

#[derive(Clone, Debug)]
pub struct InstantiatedNodes {
    pub value: Option<NodeId>,
    pub timeline: Option<NodeId>,
    pub spatial: Option<NodeId>,
}

#[derive(Debug)]
struct ScheduleFuncletData {
    // map from the scheduled allocations to what things they are instantiating (if known)
    type_instantiations: HashMap<NodeId, InstantiatedNodes>
}

// NOTE: we use "available" here to mean "either not filled or not used yet"
// so basically partially defined holes that the explicator can use

// could restrict by language, but this works for now
macro_rules! make_op_codes {
    ($($_lang:ident $name:ident ($($_arg:ident : $_arg_type:tt,)*) -> $_output:ident;)*) => {
        #[derive(Clone, Debug, Hash, Eq, PartialEq)]
        pub enum OpCode {
            $($name,)*
        }
    };
}

with_operations!(make_op_codes);

#[derive(Debug, Clone)]
struct ScheduleScopeData {
    // structure to manage the explication information for the current scope
    // the rule is more-to-less specific, then go up to the next scope out

    // the funclet name being worked on in this scope
    // a funclet is always named, even a generated one
    funclet: ast::FuncletId,

    // the node of the original funclet we are working on
    // is none precisely when we are starting a new funclet 
    //   OR inside a synthesized funclet
    // note that we may want to actually have two structs here in a way
    // then we can hold the "goal" of the sub-funclet more easily?
    node: Option<usize>,

    // the spec functions being implemented at this point of the stack
    spec_functions: SpecLanguages,

    // map from spec location information to all instantiations in this funclet
    // note that there may be duplicates of the same node across scheduled instantiations
    // we only care about local information
    instantiations: HashMap<Location, Vec<(ir::Place, usize)>>,

    // map from operation code to a vector of "available" allocations
    // for now, these consist of exactly allocations where we don't yet know the type 
    allocations: HashMap<OpCode, Vec<usize>>,

    // most recently found multiline hole, if one exists in this scope
    // note that explication holes are named in corrections
    explication_hole: bool,
}
