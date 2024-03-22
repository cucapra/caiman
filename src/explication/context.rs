pub mod data_impls;
pub mod instate;
pub mod outstate;
pub mod staticcontext;

use super::util::*;
use super::Hole;
use crate::debug_info::DebugInfo;
use crate::ir;
use crate::stable_vec;
use crate::explication::expir;
use crate::explication::expir::{NodeId, FuncletId, TypeId, PlaceId, StorageTypeId};
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::rust_wgpu_backend::ffi;

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
    // they (by default) will be filled in the most "recent" open slot
    to_fill: HashSet<Location>,

    // nodes we've built on this particular funclet of the stack
    nodes: VecDeque<ir::Node>,

    // found tail edge for this funclet (if we managed to explicate one)
    tail_edge: Option<ir::TailEdge>,
}

#[derive(Debug)]
pub struct StaticContext<'context> {
    // static information we work out before explicating
    // may be updated between explicating individual funclets

    // the entire original program, useful for looking things up
    // note that we are constructing a completely fresh program recursively
    // so the original program is not mutated
    program: &'context expir::Program,

    pub debug_map: &'context DebugInfo,

    // information found about a given spec funclet
    spec_explication_data: HashMap<FuncletId, SpecFuncletData>,
}

// this information is static, and doesn't change as explication progresses
#[derive(Debug)]
struct SpecFuncletData {
    // map of direct node dependencies for scheduling
    node_dependencies: HashMap<NodeId, Vec<NodeId>>,

    // type information derived from dependencies
    // technically this can be derived from the nodes lazily
    // but it's slow enough recursing repeatedly when explicating to wanna do it up-front I guess
    deduced_types: HashMap<NodeId, Vec<expir::TypeId>>,

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

#[derive(Debug, Clone)]
struct ScheduleFuncletData {
    // map from the scheduled allocations to what things they are instantiating (if known)
    type_instantiations: HashMap<NodeId, InstantiatedNodes>,
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

    // the funclet id being worked on in this scope
    pub funclet: expir::FuncletId,

    // the node of the original funclet we are working on
    // is none precisely when we are starting a new funclet
    //   OR inside a synthesized funclet
    // note that we may want to actually have two structs here in a way
    // then we can hold the "goal" of the sub-funclet more easily?
    node: Option<NodeId>,

    // the index of the command we are building
    // incremented by one each "step" of the recursion
    // useful to keep track of naming and boring indexing details
    node_index: usize,

    // the spec functions being implemented at this point of the stack
    spec_functions: SpecLanguages,

    // map from spec location information to all instantiations in this funclet
    // note that there may be duplicates of the same node across scheduled instantiations
    // we only care about local information
    instantiations: HashMap<Location, Vec<(expir::Place, usize)>>,

    // map from operation code to a vector of "available" allocations
    // for now, these consist of exactly allocations where we don't yet know the type
    allocations: HashMap<OpCode, Vec<usize>>,

    // most recently found multiline hole, if one exists in this scope
    // note that explication holes are named in corrections
    explication_hole: bool,
}