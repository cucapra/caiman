pub mod schedule_scope_data;
pub mod instate;
pub mod outstate;
pub mod staticcontext;

use super::expir::BufferFlags;
use super::util::*;
use super::Hole;
use crate::debug_info::DebugInfo;
use crate::ir;
use crate::stable_vec;
use crate::explication::expir;
use crate::explication::expir::{NodeId, FuncletId, TypeId, PlaceId, StorageTypeId};
use crate::stable_vec::StableVec;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::rust_wgpu_backend::ffi;

#[derive(Debug)]
pub struct StaticContext<'context> {
    // static information we work out before explicating
    // may be updated between explicating individual funclets

    // the entire original program, useful for looking things up
    // note that we are constructing a completely fresh program recursively
    // so the original program is not mutated
    program: &'context expir::Program,

    pub debug_info: &'context DebugInfo,

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
    deduced_types: HashMap<NodeId, SpecNodeTypeInformation>,

    // tailedge dependencies for scheduling
    tail_dependencies: Vec<NodeId>,
}

#[derive(Debug)]
pub struct SpecNodeTypeInformation {
    // Input types are the types of the nodes that are given as inputs
    pub input_types: Vec<expir::TypeId>,

    // Output types are the type(s) of this particular node\
    // Note that non-singular output types are assumed to be extracted before use
    pub output_types: Vec<expir::TypeId>
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
pub struct InState {
    // state information needs to be duplicated as we recurse

    // most recent scoped information found when constructing the current schedule
    // this represents a stack, so should be popped when scope ends
    scopes: Vec<ScheduleScopeData>,
}

#[derive(Debug, Clone)]
pub struct ScheduleScopeData {
    // structure to manage the explication information for the current scope
    // the rule is more-to-less specific, then go up to the next scope out

    // the funclet id being worked on in this scope
    pub funclet_id: expir::FuncletId,

    // the node of the original funclet we are working on
    // is none precisely when we are starting a new funclet
    //   OR inside a synthesized funclet
    // note that we may want to actually have two structs here in a way
    // then we can hold the "goal" of the sub-funclet more easily?
    node_id: Option<NodeId>,

    // the index of the command we are building
    // incremented by one each "step" of the recursion
    // useful to keep track of naming and boring indexing details
    node_index: usize,

    // map from spec location information to all instantiations in this funclet
    // note that there may be duplicates of the same node across scheduled instantiations
    // we only care about local information
    instantiations: HashMap<Location, HashSet<NodeId>>,

    // map from node id to which remote(s) it instantiates, and what type it has
    // we really do need both directions here, annoyingly
    storage_node_information: HashMap<NodeId, StorageNodeInformation>,

    // map from operation code to a vector of "available" operations with holes
    // for now, these consist of exactly allocations where we don't yet know the type
    available_operations: HashMap<OpCode, Vec<usize>>,

    // most recently found multiline hole, if one exists in this scope
    // note that explication holes are named in corrections
    explication_hole: bool,
}

#[derive(Debug, Clone)]
pub struct StorageNodeInformation {
    // Information about a single node storing stuff that we've recorded in our state

    // Which set of remote nodes this node stores data for (if any)
    // Observe that an empty location is completely valid
    // Also note that the empty option means specifically that we have not added anything
    //   which is distinct from adding something of all types none
    pub instantiation: Option<LocationTriple>,

    // The type of this storage
    pub typ: expir::Type,

    // Which node is "managing" our timeline
    // this could be a fence or an encoder (we don't really care here)
    // This information is used for updating the timeline when the manager changes
    pub timeline_manager: Option<NodeId>,
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

// Utility functions for defining the structures here

// Returns true if two types are "close enough" to equal
// specifically if the checked_type could be of target_type
// TODO: refine types with holes
fn is_of_type(checked_type: &expir::Type, target_type: &expir::Type) -> bool {
    match (checked_type, target_type) {
        (
            ir::Type::NativeValue {
                storage_type: storage_type1,
            },
            ir::Type::NativeValue {
                storage_type: storage_type2,
            },
        ) => true,
        (
            ir::Type::Ref {
                storage_type: storage_type1,
                storage_place: storage_place1,
                buffer_flags: buffer_flags1,
            },
            ir::Type::Ref {
                storage_type: storage_type2,
                storage_place: storage_place2,
                buffer_flags: buffer_flags2,
            },
        ) => true,
        (
            ir::Type::Fence {
                queue_place: queue_place1,
            },
            ir::Type::Fence {
                queue_place: queue_place2,
            },
        ) => true,
        (
            ir::Type::Buffer {
                storage_place: storage_place1,
                static_layout_opt: static_layout_opt1,
                flags: flags1,
            },
            ir::Type::Buffer {
                storage_place: storage_place2,
                static_layout_opt: static_layout_opt2,
                flags: flags2,
            },
        ) => true,
        (
            ir::Type::Encoder {
                queue_place: queue_place1,
            },
            ir::Type::Encoder {
                queue_place: queue_place2,
            },
        ) => true,
        (ir::Type::Event, ir::Type::Event) => true,
        (ir::Type::BufferSpace, ir::Type::BufferSpace) => true,
        _ => false,
    }
}