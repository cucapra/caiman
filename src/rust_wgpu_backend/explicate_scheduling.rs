// #![allow(warnings, unused)]
use ron::value;

use crate::ir;
use std::collections::{HashMap, hash_map::Entry};
pub use crate::rust_wgpu_backend::ffi as ffi;

// TODO: for mutual recursion enum BackReferences

#[derive(Debug)]
enum ResolvedType {
    NoType,
    Single (ffi::TypeId),
    Multiple (Vec<ffi::TypeId>)
}

#[derive(Debug)]
struct ResolvedValueNode {
    schedule_id : usize, // TODO: breaks on mutual recursion
    timeline_id : usize
}

#[derive(Debug)]
struct ResolvedScheduleNode {
    type_info : ResolvedType,
}

#[derive(Debug)]
struct PartialData { // a dumb distinction with the rework, but whatever
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
}

#[derive(Debug)]
struct ScheduleBlob {
    schedule : PartialSchedule,
    information : PartialInformation,
    timeline : PartialTimeline,
    to_update : Vec<ir::RemoteNodeId>
}

#[derive(Debug)]
struct SchedulingContext<'a> {
    program : &'a mut ir::Program,
    new_schedules : HashMap<usize, ScheduleBlob>,
    location : ir::RemoteNodeId,
    head : Option<ir::RemoteNodeId>, // stupid pipeline setup stuff
    // scheduled node maps
    resolved_schedules : HashMap<ir::RemoteNodeId, ResolvedScheduleNode>, 
    resolved_values : HashMap<ir::RemoteNodeId, ResolvedValueNode>
}

fn new_partial_schedule() -> PartialSchedule {
    let mut schedule = PartialSchedule { 
        core: PartialData { 
            input_types: Vec::new(), 
            output_types: Vec::new(), 
            nodes: Vec::new(),
            tail_edge : None,
        }
    };
    schedule.core.input_types.push(1);
    schedule.core.output_types.push(1);
    schedule.core.nodes.push(ir::Node::Phi { index: 0 });
    schedule
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
    let mut input_slots_default = empty_slot();
    input_slots_default.value_tag = ir::ValueTag::Input { 
        funclet_id: *value_index, 
        index: 0 
    };
    information.input_slots.insert(0, input_slots_default);
    information.output_slots.insert(0, empty_slot());
    information
}

fn get_partial_schedule<'a>(value_index : &usize, 
context : &'a mut SchedulingContext) -> &'a mut ScheduleBlob {
    let entry = 
        context.new_schedules.entry(*value_index);
    match entry {
        Entry::Occupied(entry) => { 
            entry.into_mut() 
        },
        Entry::Vacant(entry) =>  {
            let schedule = new_partial_schedule();
            let information = 
                new_information(value_index);
            let timeline = new_partial_timeline();
            let mut new_funclet = ScheduleBlob { 
                schedule,
                information,
                timeline,
                to_update: Vec::new()
            };
            // todo: based on number of inputs, which we're lazily taking to be one
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
    dbg!(node_count);
    let mut return_values = Box::new([0]);
    if node_count > 0 {
        return_values[0] = node_count-1;
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

fn add_blob(mut blob : ScheduleBlob, context : &mut SchedulingContext) {
    let new_schedule = build_funclet(blob.schedule.core, 
        ir::FuncletKind::ScheduleExplicit, context);
    let new_timeline = build_funclet(blob.timeline.core, 
        ir::FuncletKind::Timeline, context);
    let schedule_id = context.program.funclets.create(new_schedule);
    let timeline_id = context.program.funclets.create(new_timeline);
    let extra = build_extra(blob.information, 
        &timeline_id, context);
    context.program.scheduling_funclet_extras.insert(schedule_id, extra);
    for location in blob.to_update.drain(..) {
        let resolved = ResolvedValueNode {
            schedule_id : schedule_id,
            timeline_id : timeline_id
        };
        context.resolved_values.insert(location, resolved);
    }
}

fn finish_funclet(tail_edge : ir::TailEdge, funclet_id : &usize,
context : &mut SchedulingContext) {
    match context.new_schedules.remove(funclet_id) {
        None => {
            panic!("Attempting to add uncreated partial funclet")
        }
        Some(partial) => {
            add_blob(partial, context);
        }
    }
}

fn add_current_funclet(tail_edge : ir::TailEdge, 
context : &mut SchedulingContext) {
    let funclet_id = context.location.funclet_id;
    finish_funclet(tail_edge, &funclet_id, context);
}

fn add_schedule_node(
resolved : ResolvedScheduleNode, 
nodes : Vec<ir::Node>,
context : &mut SchedulingContext) {
    let target_id = context.location.funclet_id;
    let mut blob = 
        get_partial_schedule(&target_id, context);

    for node in nodes {
        blob.schedule.core.nodes.push(node);
    }
    let location = ir::RemoteNodeId { 
        funclet_id : context.location.funclet_id, 
        node_id : context.location.node_id
    };

    context.resolved_schedules.insert(location, resolved);
}

fn explicate_extract_result(node_id : &usize, 
index : &usize, context : &mut SchedulingContext) -> bool {
    // The goal here is to maintain the hashmaps to keep track of ids
    // Specifically the funclet and node to extract from (or to)
    match context.resolved_schedules.get(&context.location) {
        Some(info) => {
            let remote = ir::RemoteNodeId {
                funclet_id : context.location.funclet_id,
                node_id : context.location.node_id
            };
            let typ = match &info.type_info {
                ResolvedType::NoType => {
                    panic!("Invalid lack of type")
                }
                ResolvedType::Single(typ) => 
                {
                    assert!(*index == 0);
                    typ.0 
                },
                ResolvedType::Multiple(typs) => 
                    { typs[*index].0 }
            };
            let node = ir::Node::StaticAllocFromStaticBuffer { buffer: 0, 
                place: ir::Place::Gpu,
                storage_type: ffi::TypeId{0: typ}, operation: remote };

            let resolved = ResolvedScheduleNode {
                type_info: ResolvedType::NoType
            };
            add_schedule_node(resolved, vec![node], context);
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
    let resolved = ResolvedScheduleNode {
        type_info: ResolvedType::Single(ffi::TypeId(*type_id))
    };
    add_schedule_node(resolved, nodes, context);
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
        match context.head { // get the head if we don't have it
            None => {
                match node {
                    ir::Node::Phi { index:_ } => {},
                    _ => { 
                        context.head = Some(context.location.clone());
                    }
                }
            }
            _ => {}
        }
        if !context.resolved_schedules.contains_key(&context.location) {
            let node_resolved = explicate_node(node, context);
            if node_resolved {
                let value_index = context.location.funclet_id;
                let location = context.location.clone();
                let mut blob = 
                    get_partial_schedule(&value_index, context);
                blob.to_update.push(location);
                resolved = true;
            }
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
    unresolved
}

fn cleanup_partials(context : &mut SchedulingContext) {
    let mut funclets = HashMap::new();
    std::mem::swap(&mut funclets, &mut context.new_schedules);
    for (index, mut partial) in funclets.drain() {
        add_blob(partial, context);
    }
}

fn construct_pipeline(context : &mut SchedulingContext) {
    let head = match &context.head {
        None => { panic!("No head found") },
        Some(v) => { v }
    };

    let start_funclet = match context.resolved_values.get(head) {
        None => { panic!("Invalid head") },
        Some(f) => { f.schedule_id }
    };

    let pipeline = ir::Pipeline {
        name: context.program.explicate.clone(),
        entry_funclet: start_funclet,
        yield_points: std::collections::BTreeMap::new(),
    };

    context.program.pipelines.push(pipeline);
}

fn debug_funclets(program : &ir::Program) {
    // cause I'm dumb
    let mut ordered = Vec::new();
    for index in 0..(program.funclets.get_next_id()) {
        ordered.push(None)
    }
    for items in program.funclets.iter() {
        ordered[*items.0] = Some(items.1);
    }
    let mut id = 0;
    println!("Funclets: ");
    for funclet in ordered {
        print!("{} : ", id);
        println!("{:#?}", funclet.unwrap());
        id += 1;
    }
    println!("Extras: {:#?}", program.scheduling_funclet_extras);
    println!("Pipelines: {:#?}", program.pipelines);

    panic!("Explicated"); // to see debug information
}

pub fn explicate_scheduling(program : &mut ir::Program)
{
    if program.explicate == "" { 
        return;
    }
    
    let original = program.funclets.clone();
    let mut initial_location = ir::RemoteNodeId 
        { funclet_id : 0, node_id : 0 };
    let starting_index = program.funclets.get_next_id();
    let mut context = 
        SchedulingContext{
            program : program, 
            new_schedules : HashMap::new(),
            location : initial_location,
            head : None,
            resolved_schedules : HashMap::new(),
            resolved_values : HashMap::new()
        };
    let mut unresolved = true;
    while unresolved {
        unresolved = false; // reset the count
        context.location.funclet_id = 0; // reset the funclet number
        for funclet in original.iter() {
            context.location.funclet_id = *funclet.0;
            unresolved = unresolved || explicate_funclet(
                funclet.1, &mut context);
        }
    }
    cleanup_partials(&mut context);
    construct_pipeline(&mut context);
    // debug_funclets(&program);
}