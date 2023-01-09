// #![allow(warnings, unused)]
use ron::value;

use crate::ir;
use crate::ir::Funclet;
use crate::ir::Program;
use crate::ir::RemoteNodeId;
use crate::ir::SchedulingFuncletExtra;
use crate::ir::TailEdge;
use crate::shadergen;
use crate::arena::Arena;
use std::collections::btree_map::VacantEntry;
use std::collections::hash_map::Entry;
use std::default;
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

#[derive(Debug)]
struct PartialData {
    input_types : Vec<ir::TypeId>,
    output_types : Vec<ir::TypeId>,
    nodes : Vec<ir::Node>,
    tail_edge : Option<ir::TailEdge>
}

#[derive(Debug)]
struct PartialTimeline {
    core : PartialData
}

#[derive(Debug)]
struct PartialInformation {
    pub value_funclet_id : ir::FuncletId,
	pub input_slots : HashMap<usize, ir::SlotInfo>,
	pub output_slots : HashMap<usize, ir::SlotInfo>,
	pub input_fences : HashMap<usize, ir::FenceInfo>,
	pub output_fences : HashMap<usize, ir::FenceInfo>,
	pub input_buffers : HashMap<usize, ir::BufferInfo>,
	pub output_buffers : HashMap<usize, ir::BufferInfo>,
}

#[derive(Debug)]
struct PartialSchedule {
	core : PartialData,
    information : PartialInformation,
    timeline : PartialTimeline
}

#[derive(Debug)]
enum ResolvedType {
    NoType,
    Single (ffi::TypeId),
    Multiple (Vec<ffi::TypeId>)
}

#[derive(Debug)]
struct ResolvedNode {
    type_info : ResolvedType
}

struct SchedulingContext<'a> {
    program : &'a mut ir::Program,
    new_schedules : HashMap<usize, PartialSchedule>,
    location : ir::RemoteNodeId,
    // scheduled node map
    resolved_map : HashMap<ir::RemoteNodeId, ResolvedNode>, 
}

fn new_partial_timeline() -> PartialTimeline {
    PartialTimeline {
        core: PartialData { 
            input_types: vec![2],
            output_types: vec![2],
            nodes: vec![ir::Node::Phi { index: 0 }],
            tail_edge: None,
        }
    }
}

fn empty_slot() -> ir::SlotInfo {
    ir::SlotInfo {
        value_tag : ir::ValueTag::None,
        timeline_tag: ir::TimelineTag::None,
        spatial_tag: ir::SpatialTag::None,
    }
}

fn new_information(value_index : &usize) 
-> PartialInformation {
    let mut information = PartialInformation {
        value_funclet_id: *value_index,
        input_slots: HashMap::new(),
        output_slots: HashMap::new(),
        input_fences: HashMap::new(),
        output_fences: HashMap::new(),
        input_buffers: HashMap::new(),
        output_buffers: HashMap::new(),
    };
    // todo: based on number of inputs, which we're lazily taking to be one
    information.input_slots.insert(0, empty_slot());
    information.output_slots.insert(0, empty_slot());
    information
}

fn get_partial_schedule<'a>(value_index : &usize, 
context : &'a mut SchedulingContext) -> &'a mut PartialSchedule {
    let entry = 
        context.new_schedules.entry(*value_index);
    match entry {
        Entry::Occupied(entry) => { 
            entry.into_mut() 
        },
        Entry::Vacant(entry) =>  {
            let information = 
                new_information(value_index);
            let timeline = new_partial_timeline();
            let mut new_funclet = PartialSchedule { 
                core: PartialData { 
                    input_types: Vec::new(), 
                    output_types: Vec::new(), 
                    nodes: Vec::new(),
                    tail_edge : None,
                },
                information: information,
                timeline: timeline,
            };
            // todo: based on number of inputs, which we're lazily taking to be one
            new_funclet.core.input_types.push(1);
            new_funclet.core.output_types.push(1);
            new_funclet.core.nodes.push(ir::Node::Phi { index: 0 });
            entry.insert(new_funclet)
        }
    }
}

fn default_tail(partial : &PartialData, context : &mut SchedulingContext) 
-> ir::TailEdge {
    let mut node_count = 0;
    for node in &partial.nodes {
        match node {
            ir::Node::Phi{index:_} => {}
            _ => { node_count += 1 }
        }
    }
    let mut return_values = Box::new([]);
    if node_count > 0 {
        Box::new([node_count-1]);
    }
    ir::TailEdge::Return { 
        return_values: return_values  // zero-indexing
    }
}

fn build_funclet(core : PartialData, kind : ir::FuncletKind,
context : &mut SchedulingContext) -> ir::Funclet {
    let default = default_tail(&core, context);
    ir::Funclet {
        kind : kind,
        input_types : core.input_types.into_boxed_slice(),
        output_types : core.output_types.into_boxed_slice(),
        nodes : core.nodes.into_boxed_slice(),
        tail_edge : match core.tail_edge {
            None => { default },
            Some(tail) => { tail }
        },
    }
}

fn build_extra(info : PartialInformation, timeline_id : &usize,
context : &mut SchedulingContext) -> ir::SchedulingFuncletExtra {
    // todo: lazy location information
    let in_timeline = ir::TimelineTag::Input { 
        funclet_id: *timeline_id, 
        index: 0, 
    };
    let out_timeline = ir::TimelineTag::Input { 
        funclet_id: *timeline_id, 
        index: 0, 
    };
    ir::SchedulingFuncletExtra {
        value_funclet_id: info.value_funclet_id,
        input_slots: info.input_slots,
        output_slots: info.output_slots,
        input_fences: info.input_fences,
        output_fences: info.output_fences,
        input_buffers: info.input_buffers,
        output_buffers: info.output_buffers,
        in_timeline_tag: in_timeline,
        out_timeline_tag: out_timeline,
    }
}

fn add_funclet(partial : PartialSchedule, context : &mut SchedulingContext) {
    let new_schedule = build_funclet(partial.core, 
        ir::FuncletKind::ScheduleExplicit, context);
    let new_timeline = build_funclet(partial.timeline.core, 
        ir::FuncletKind::Timeline, context);
    let schedule_id = context.program.funclets.create(new_schedule);
    let timeline_id = context.program.funclets.create(new_timeline);
    let extra = build_extra(partial.information, 
        &timeline_id, context);
    context.program.scheduling_funclet_extras.insert(schedule_id, extra);
}

fn finish_funclet(tail_edge : ir::TailEdge, funclet_id : &usize,
context : &mut SchedulingContext) {
    match context.new_schedules.remove(funclet_id) {
        None => {
            panic!("Attempting to add uncreated partial funclet")
        }
        Some(partial) => {
            add_funclet(partial, context);
        }
    }
}

fn add_current_funclet(tail_edge : ir::TailEdge, 
    context : &mut SchedulingContext) {
    let funclet_id = context.location.funclet_id;
    finish_funclet(tail_edge, &funclet_id, context);
}

fn add_nodes(resolution : ResolvedNode, nodes : Vec<ir::Node>,
context : &mut SchedulingContext) {
    let target_id = context.location.funclet_id;
    let mut funclet = 
        get_partial_schedule(&target_id, context);
    for node in nodes {
        funclet.core.nodes.push(node);
    }
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
                place: ir::Place::Gpu,
                storage_type: ffi::TypeId{0: typ}, operation: remote };
            let resolution = ResolvedNode {
                type_info : ResolvedType::NoType
            };
            add_nodes(resolution, vec![node], context);
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
    let mut nodes = Vec::new();
    nodes.push(ir::Node::AllocTemporary { 
        place: ir::Place::Local, 
        storage_type: ffi::TypeId {0: *type_id}, 
        operation: new_id
    });
    nodes.push(ir::Node::EncodeDo { 
        place: ir::Place::Local, 
        operation: ir::RemoteNodeId {
            funclet_id: context.location.funclet_id,
            node_id: context.location.node_id
        }, 
        inputs: Box::new([]), 
        outputs: Box::new([1]) // todo: probably wrong in general
    });
    let resolution = ResolvedNode {
        type_info : ResolvedType::Single(ffi::TypeId(*type_id))
    };
    add_nodes(resolution, nodes, context);
    true
}

fn explicate_value_function(function_id : &usize, arguments : &Box<[usize]>, 
context : &mut SchedulingContext) -> bool {
    let id = context.program.funclets.get_next_id();
    let tail_edge = ir::TailEdge::ScheduleCall { 
        value_operation: context.location, 
        callee_funclet_id: id,
        callee_arguments: Box::new([]), // TODO: clearly wrong
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
    resolved
}

fn explicate_nodes(nodes : &Box<[ir::Node]>, 
context : &mut SchedulingContext) -> bool {
    let mut resolved = false;
    for node in &**nodes {
        if !context.resolved_map.contains_key(&context.location) {
            resolved = explicate_node(node, context) || resolved;
        }
        context.location.node_id += 1;
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
    std::mem::swap(&mut funclets, &mut context.new_schedules);
    for (index, mut partial) in funclets.drain() {
        add_funclet(partial, context);
    }
}

fn print_funclets(program : &ir::Program) {
    // cause I'm dumb
    let mut ordered = Vec::new();
    for index in 0..(program.funclets.get_next_id()) {
        ordered.push(None)
    }
    for items in program.funclets.iter() {
        ordered[*items.0] = Some(items.1);
    }
    let mut id = 0;
    for funclet in ordered {
        print!("{} : ", id);
        println!("{:#?}", funclet);
        id += 1;
    }
    println!("{:#?}", program.scheduling_funclet_extras);
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
        SchedulingContext{program : program, new_schedules : HashMap::new(),
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
    print_funclets(&program);
    panic!("Explicated");
}