use ron::value;

use crate::ir;
use crate::ir::Funclet;
use crate::ir::Program;
use crate::ir::TailEdge;
use crate::shadergen;
use crate::arena::Arena;
use std::collections::btree_map::VacantEntry;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::hash;
use std::hash::Hash;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;
pub use crate::rust_wgpu_backend::ffi as ffi;

struct PartialFunclet {
    id : usize,
    associated_value : usize,
    kind : ir::FuncletKind,
	input_types : Vec<ir::TypeId>,
	output_types : Vec<ir::TypeId>,
	nodes : Vec<ir::Node>,
}

enum ResolvedType {
    NoType,
    Single (ffi::TypeId),
    Multiple (Vec<ffi::TypeId>)
}

struct ResolvedNode {
    type_info : ResolvedType
}

struct SchedulingContext<'a> {
    program : &'a mut ir::Program,
    new_funclets : HashMap<usize, PartialFunclet>,
    funclet_index : usize,
    location : ir::RemoteNodeId,
    // scheduled node map
    // note that nodes can only be referenced in the funclet,
    //   so this should get reset on starting a new funclet
    resolved_map : HashMap<ir::RemoteNodeId, ResolvedNode>, 
}

fn get_partial_funclet<'a>(value_index : &usize, context : &'a mut SchedulingContext)
    -> &'a mut PartialFunclet {
    
    match context.new_funclets.entry(*value_index) {
        Entry::Occupied(entry) => { 
            entry.into_mut() 
        },
        Entry::Vacant(entry) =>  {
            let id = context.funclet_index;
            let new_funclet = PartialFunclet { 
                id : id, 
                associated_value : *value_index,
                kind: ir::FuncletKind::ScheduleExplicit, 
                input_types: Vec::new(), 
                output_types: Vec::new(), 
                nodes: Vec::new()
            };
            context.funclet_index += 1;
            entry.insert(new_funclet)
        }
    }
}

fn get_current_funclet_id(context : &mut SchedulingContext) -> usize {
    // Panics if the value_index isn't built yet
    match context.new_funclets.get(&context.location.node_id) {
        None => { panic!("Accessing index of funclet not built") }
        Some(partial) => {
            partial.id
        }
    }
}

fn add_funclet(tail_edge : ir::TailEdge, funclet_id : &usize,
    context : &mut SchedulingContext) {
    match context.new_funclets.remove(funclet_id) {
        None => {
            panic!("Attempting to add uncreated partial funclet")
        }
        Some(updated_funclet) => {
            let new_funclet = ir::Funclet {
                kind : updated_funclet.kind,
                input_types : updated_funclet.input_types.into_boxed_slice(),
                output_types : updated_funclet.output_types.into_boxed_slice(),
                nodes : updated_funclet.nodes.into_boxed_slice(),
                tail_edge : tail_edge,
            };
            context.program.funclets.create(new_funclet);
        }
    }
}

fn add_current_funclet(tail_edge : ir::TailEdge, 
    context : &mut SchedulingContext) {
    let funclet_id = &get_current_funclet_id(context);
    add_funclet(tail_edge, funclet_id, context);
}

fn add_current_node(
    resolution : ResolvedNode, node : ir::Node,
    context : &mut SchedulingContext) {
    let target_id = context.location.funclet_id;
    let mut funclet = 
        get_partial_funclet(&target_id, context);
    funclet.nodes.push(node);
    let location = ir::RemoteNodeId { 
        funclet_id : context.location.funclet_id, 
        node_id : context.location.node_id
    };
    context.resolved_map.insert(location, resolution);
}

fn explicate_extract_result(node_id : &usize, 
    index : &usize, context : &mut SchedulingContext) -> bool {
    // The goal here is to maintain the hashmaps to keep track of ids
    // Specifically the funclet and node to extract from (or to)
    match context.resolved_map.get(&context.location) {
        Some(callInfo) => {
            let remote = ir::RemoteNodeId {
                funclet_id : context.location.funclet_id,
                node_id : context.location.node_id
            };
            let typ = match &callInfo.type_info {
                ResolvedType::NoType => {
                    panic!("Invalid lack of type")
                }
                ResolvedType::Single(typ) => 
                {
                    assert!(*index == 0);
                    typ.0 
                },
                ResolvedType::Multiple(typs) => { typs[*index].0 }
            };
            let node = ir::Node::StaticAllocFromStaticBuffer { buffer: 0, 
                place: ir::Place::Cpu,
                storage_type: ffi::TypeId{0: typ}, operation: remote };
            let resolution = ResolvedNode {
                type_info : ResolvedType::NoType
            };
            add_current_node(resolution, node, context);
            true
        }
        None => {
            false
        }
    }
}

fn explicate_constant(type_id : &usize, 
    context : &mut SchedulingContext) -> bool {
    let new_id = ir::RemoteNodeId {
        funclet_id: context.location.funclet_id,
        node_id: context.location.node_id
    };
    let node = ir::Node::StaticAllocFromStaticBuffer { 
        buffer: 0,
        place: ir::Place::Cpu, storage_type: ffi::TypeId {0: *type_id}, 
        operation: new_id };
    let resolution = ResolvedNode {
        type_info : ResolvedType::NoType
    };
    add_current_node(resolution, node, context);
    true
}

fn explicate_value_function(function_id : &usize, arguments : &Box<[usize]>, 
    context : &mut SchedulingContext) -> bool {
    let tail_edge = ir::TailEdge::ScheduleCall { 
        value_operation: context.location, 
        callee_funclet_id: get_current_funclet_id(context), 
        callee_arguments: Box::new([0]), // TODO: clearly wrong
        continuation_join: 0
    };
    add_current_funclet(tail_edge, context);
    true
}

fn explicate_select(condition : &usize, true_case : &usize, false_case : &usize, 
    context : &mut SchedulingContext) -> bool {
    true
}

fn explicate_external(node : &ir::Node, context : &mut SchedulingContext) -> bool {
    true
}

fn explicate_node(node : &ir::Node, context : &mut SchedulingContext) -> bool {
    let resolved = match node {
        ir::Node::ExtractResult { node_id, 
            index } => explicate_extract_result(node_id, index, context),
        ir::Node::ConstantInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::ConstantUnsignedInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::CallValueFunction { function_id, 
            arguments } => 
            explicate_value_function(function_id, arguments, context),
        ir::Node::Select { condition, true_case, 
            false_case } => 
            explicate_select(condition, true_case, false_case, context),
        ir::Node::CallExternalCpu { external_function_id, 
            arguments } => explicate_external(node, context),
        ir::Node::CallExternalGpuCompute { external_function_id, 
            dimensions, arguments } => 
            explicate_external(node, context),
        _ => true
    };
    context.location.node_id += 1;
    resolved
}

fn explicate_nodes(nodes : &Box<[ir::Node]>, 
    context : &mut SchedulingContext) -> bool {
    let mut resolved = false;
    for node in &**nodes {
        if !context.resolved_map.contains_key(&context.location) {
            resolved = resolved || explicate_node(node, context);
            context.location.node_id += 1
        }
    }
    !resolved
}

fn explicate_funclet(funclet : &ir::Funclet,
    context : &mut SchedulingContext) -> bool {
    // Calculates the new funclets to work with (if any)
    context.location.node_id = 0; // reset node_id to new funclet
    let unresolved = match funclet.kind {
        ir::FuncletKind::MixedImplicit => false,
        ir::FuncletKind::MixedExplicit => false,
        ir::FuncletKind::Value => explicate_nodes(&funclet.nodes, context),
        ir::FuncletKind::ScheduleExplicit => false,
        ir::FuncletKind::Inline => false,
        ir::FuncletKind::Timeline => false,
        ir::FuncletKind::Spatial => false,
    };
    context.location.funclet_id += 1;
    unresolved
}

fn cleanup_partials(context : &mut SchedulingContext) {
    let mut funclets = HashMap::new();
    std::mem::swap(&mut funclets, &mut context.new_funclets);
    for partials in funclets.drain() {
        let tail_edge = ir::TailEdge::Return { 
            return_values: Box::new([]) 
        };
        add_funclet(tail_edge, &partials.0, context);
    }
}

pub fn explicate_scheduling(program : &mut ir::Program)
{
    if !program.explicate { 
        return; 
    }
    
    let original = program.funclets.clone();
    let mut initial_location = ir::RemoteNodeId 
        { funclet_id : 0, node_id : 0 };
    let starting_index = program.funclets.get_next_id();
    let mut context = 
        SchedulingContext{program : program, new_funclets : HashMap::new(),
            funclet_index : starting_index,
            location : initial_location, resolved_map : HashMap::new()};

    let mut unresolved = true;
    while unresolved {
        unresolved = false; // reset the count
        context.location.funclet_id = 0; // reset the funclet number
        for funclet in original.iter() {
            unresolved = unresolved || explicate_funclet(
                funclet.1, &mut context);
        }
    }
    cleanup_partials(&mut context);
}