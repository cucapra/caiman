// #![allow(warnings, unused)]
use ron::value;

use crate::ir;
use crate::ast;
use std::{collections::{HashMap, hash_map::Entry, HashSet}, any, hash::Hash};
use crate::rust_wgpu_backend::ffi as ffi;

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
struct ResolvedScheduleNode {}

#[derive(Debug)]
struct PartialData { // a dumb distinction with the rework, but whatever
    input_types : Vec<ast::TypeId>,
    output_types : Vec<ast::TypeId>,
    nodes : Vec<ast::Node>,
    tail_edge : Option<ast::TailEdge>
}

#[derive(Debug)]
struct PartialTimeline {
    core : PartialData
}

#[derive(Debug)]
struct PartialInformation {
    // todo: add ast slots and fences and crap
    pub value_funclet_id : ast::FuncletId,
	pub input_slots : ast::UncheckedDict,
	pub output_slots : ast::UncheckedDict,
	pub input_fences : ast::UncheckedDict,
	pub output_fences : ast::UncheckedDict,
	pub input_buffers : ast::UncheckedDict,
	pub output_buffers : ast::UncheckedDict,
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
    remote_update : Vec<ast::RemoteNodeId>,
    allocated : HashMap<String, HashMap<String, String>>
}

#[derive(Debug)]
struct SchedulingContext<'a> {
    program : &'a mut ast::Program,
    new_schedules : HashMap<String, ScheduleBlob>,
    location : ast::RemoteNodeId,
    // scheduled node maps
    resolved_schedules : HashMap<ast::RemoteNodeId, ResolvedScheduleNode>, 
    resolved_values : HashMap<String, ResolvedValueNode>
}

fn id(s : &str) -> ast::Value {
    ast::Value::ID(s.to_string())
}

fn get_slot(context : &mut SchedulingContext) -> String {
    for potential in context.program.types {
        match potential {
            ast::TypeDecl::Local(typ) => {
                if !typ.event {
                    return typ.name
                }
            }
            _ => {}
        }
    }
    let name = "s0".to_string(); // lazy for now
    let result = ast::TypeDecl::Local(ast::LocalType {
        event: false,
        name : name.clone(),
        data: HashMap::new(),
    });
    context.program.types.push(result);
    name
}

fn get_event(context : &mut SchedulingContext) -> String {
    for potential in context.program.types {
        match potential {
            ast::TypeDecl::Local(typ) => {
                if typ.event {
                    return typ.name
                }
            }
            _ => {}
        }
    }
    let name = "e0".to_string(); // lazy for now
    let result = ast::TypeDecl::Local(ast::LocalType {
        event: true,
        name : name.clone(),
        data: HashMap::new(),
    });
    context.program.types.push(result);
    name
}

fn new_partial_schedule(context : &mut SchedulingContext) -> PartialSchedule {
    let mut schedule = PartialSchedule { 
        core: PartialData { 
            input_types: Vec::new(), 
            output_types: Vec::new(), 
            nodes: Vec::new(),
            tail_edge : None,
        }
    };
    schedule.core.input_types.push(ast::Type::Local(get_slot(context)));
    schedule.core.output_types.push(ast::Type::Local(get_slot(context)));
    schedule.core.nodes.push(ast::Node::Phi { index: 0 });
    schedule
}

fn new_partial_timeline(context : &mut SchedulingContext) -> PartialTimeline {
    PartialTimeline {
        core: PartialData { 
            input_types: vec![ast::Type::Local(get_event(context))],
            output_types: vec![ast::Type::Local(get_event(context))],
            nodes: vec![ast::Node::Phi { index: 0 }],
            tail_edge: None,
        }
    }
}

fn empty_slot(context : &mut SchedulingContext) -> ast::SlotInfo {
    ast::SlotInfo {
        value_tag: ast::ValueTag::Core(ast::TagCore::None),
        timeline_tag: ast::TimelineTag::Core(ast::TagCore::None),
        spatial_tag: ast::SpatialTag::Core(ast::TagCore::None),
    }
}

fn new_information(value_index : &String, context : &mut SchedulingContext)
-> PartialInformation {
    let mut information = PartialInformation {
        value_funclet_id: value_index.clone(),
        input_slots: HashMap::new(),
        output_slots: HashMap::new(),
        input_fences: HashMap::new(),
        output_fences: HashMap::new(),
        input_buffers: HashMap::new(),
        output_buffers: HashMap::new(),
    };
    // todo: based on number of inputs, which we're lazily taking to be one
    let mut input_slots_default = empty_slot(context);
    input_slots_default.value_tag = ast::ValueTag::Core(ast::TagCore::Input(ast::RemoteNodeId {
        funclet_id: value_index.clone(),
        node_id: "".to_string(), // not ideal
    }));
    information.input_slots.insert(ast::Value::ID("".to_string()),
                                   ast::DictValue::Raw(ast::Value::SlotInfo(input_slots_default)));
    information.output_slots.insert(ast::Value::ID("".to_string()),
                                    ast::DictValue::Raw(ast::Value::SlotInfo(empty_slot(context))));
    information
}

fn new_blob(value_index : &String, context : &mut SchedulingContext) -> ScheduleBlob {
    let schedule = new_partial_schedule(context);
    let information = new_information(value_index, context);
    let timeline = new_partial_timeline(context);
    ScheduleBlob { 
        schedule,
        information,
        timeline,
        remote_update: Vec::new(),
        allocated: HashMap::new()
    }
}

fn get_blob<'a>(value_index : &String,
context : &'a mut SchedulingContext) -> &'a ScheduleBlob {
    context.new_schedules.entry(value_index.clone()).
        or_insert(new_blob(value_index, context))
}

fn get_blob_mut<'a>(value_index : &String,
context : &'a mut SchedulingContext) -> &'a mut ScheduleBlob {
    context.new_schedules.entry(value_index.clone()).
        or_insert(new_blob(value_index, context))
}

fn get_current_blob<'a>(context : &'a mut SchedulingContext)
-> &'a ScheduleBlob {
    let value_index = context.location.funclet_id.clone();
    get_blob(&value_index, context)
}

fn get_current_blob_mut<'a>(context : &'a mut SchedulingContext)
-> &'a mut ScheduleBlob {
    let value_index = context.location.funclet_id.clone();
    get_blob_mut(&value_index, context)
}

fn get_local_funclet<'a>(name : &String, context : &'a SchedulingContext)
-> &'a ast::Funclet {
    for funclet in context.program.funclets {
        match funclet {
            ast::FuncletDef::Local(f) => {
                if f.header.name == *name {
                    return &f
                }
            },
            _ => {}
        }
    };
    panic!(format!("No funclet named {:?} found", name))
}

fn get_index(funclet_id : &String, index : usize, context : &mut SchedulingContext) -> String {
    let funclet = get_local_funclet(funclet_id, context);
    let command = funclet.commands.get(index).unwrap();
    match command {
        ast::Command::IRNode {
            name, node
        } => {
            if name != "" {
                name.clone()
            }
            else {
                panic!(format!("Invalid index {} for funclet {:?}", index, funclet_id))
            }
        },
        _ => panic!(format!("Unknown funclet {:?}", funclet_id))
    }
}

fn get_current_funclet<'a>(context : &'a SchedulingContext)
-> &'a ast::Funclet {
    let index = context.location.funclet_id.clone();
    get_local_funclet(&index, context)
}

fn get_external_cpu<'a>(name: &String,
context: &'a SchedulingContext) -> &'a ast::ExternalCpuFunction {
    for funclet in context.program.funclets {
        match funclet {
            ast::FuncletDef::ExternalCPU(f) => {
                if f.name == *name {
                    return &f
                }
            },
            _ => {}
        }
    };
    panic!(format!("No external funclet named {:?} found", name))
}

fn get_current_allocated<'a>(node_id : &String, argument : &String,
context : &'a mut SchedulingContext) -> Option<&'a String> {
    let blob = get_current_blob_mut(context);
    match blob.allocated.get(node_id.as_str()) {
        None => { None }
        Some(map) => { map.get(argument) }
    }
}

fn update_current_allocated(node_id : &String, index : &String,
context : &mut SchedulingContext) {
    // Assumes we _haven't_ added this node yet
    let blob = get_current_blob_mut(context);
    let current_node = blob.schedule.core.nodes.len();
    let entry = blob.allocated.entry(node_id.clone()).
        or_insert(HashMap::new());
    entry.insert(index.clone(), format!("{}", current_node));
}

fn default_tail(partial : &PartialData, context : &mut SchedulingContext) 
-> ast::TailEdge {
    let mut node_count = 0;
    for node in &partial.nodes {
        match node {
            ast::Node::Phi{index:_} => {}
            _ => { node_count += 1 }
        }
    }
    let mut var = format!("{}", 0);
    if node_count > 0 {
        var = format!("{}", node_count-1);
    }
    ast::TailEdge::Return { var }
}

fn build_funclet(mut core : PartialData, kind : ir::FuncletKind,
context : &mut SchedulingContext) -> ast::Funclet {
    let mut commands = Vec::new();
    let tail_found = false;
    for node in core.nodes.drain(..) {
        commands.push(ast::Command::IRNode(node));
    }
    if !tail_found {
        commands.push(ast::Command::Tail(default_tail(&core, context)));
    }
    ast::Funclet {
        kind,
        header: ast::FuncletHeader {
            ret: core.output_types[0].clone(),
            name: "".to_string(),
            args: core.input_types,
        },
        commands,
    }
}

fn build_extra(info : PartialInformation, timeline_id : &String,
context : &mut SchedulingContext) -> ast::Extra {
    // todo: lazy location information
    let mut data = HashMap::new();
    data.insert(id("value_funclet_id"), ast::DictValue::Raw(ast::Value::FnName(info.value_funclet_id)));
    data.insert(id("input_slots"), ast::DictValue::Dict(info.input_slots));
    data.insert(id("output_slots"), ast::DictValue::Dict(info.output_slots));
    data.insert(id("input_fences"), ast::DictValue::Dict(info.input_fences));
    data.insert(id("output_fences"), ast::DictValue::Dict(info.output_fences));
    data.insert(id("input_buffers"), ast::DictValue::Dict(info.input_buffers));
    data.insert(id("output_buffers"), ast::DictValue::Dict(info.output_buffers));

    let
    data.insert(id("in_timeline"), ast::TimelineTag::Core(ast::TagCore::Input(timeline_id));
    data.insert(id("in_timeline"), ast::DictValue::Dict(out_timeline));
    ast::Extra {
        name: context.location.funclet_id.clone(),
        data,
    }
}

fn add_blob(mut blob : ScheduleBlob, context : &mut SchedulingContext) {
    let new_schedule = build_funclet(blob.schedule.core, 
        ast::FuncletKind::ScheduleExplicit, context);
    let new_timeline = build_funclet(blob.timeline.core, 
        ast::FuncletKind::Timeline, context);
    let schedule_id = context.program.funclets.create(new_schedule);
    let timeline_id = context.program.funclets.create(new_timeline);
    let extra = build_extra(blob.information, 
        &timeline_id, context);
    context.program.scheduling_funclet_extras.insert(schedule_id, extra);
    for location in blob.remote_update.drain(..) {
        let resolved = ResolvedValueNode {
            schedule_id,
            timeline_id
        };
        context.resolved_values.insert(location.funclet_id, resolved);
    }
}

fn finish_funclet(tail_edge : ast::TailEdge, funclet_id : &usize,
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

fn add_current_funclet(tail_edge : ast::TailEdge, 
context : &mut SchedulingContext) {
    let funclet_id = context.location.funclet_id;
    finish_funclet(tail_edge, &funclet_id, context);
}

fn add_schedule_node(
resolved : ResolvedScheduleNode, 
nodes : Vec<ast::Node>,
context : &mut SchedulingContext) {
    let target_id = context.location.funclet_id;
    let mut blob = get_blob_mut(&target_id, context);

    for node in nodes {
        blob.schedule.core.nodes.push(node);
    }

    context.resolved_schedules.insert(context.location.clone(), resolved);
}

fn explicate_extract_result(node_id : &usize, 
index : &usize, context : &mut SchedulingContext) -> bool {
    // The goal here is to maintain the hashmaps to keep track of ids
    // Specifically the funclet and node to extract from (or to)
    let remote = ast::RemoteNodeId {
        funclet_id : context.location.funclet_id,
        node_id : context.location.node_id
    };
    let node = &get_current_funclet(context).nodes[*node_id];
    match node {
        ast::Node::CallExternalCpu { 
        external_function_id, 
        arguments } => {
            let output = get_external(external_function_id, 
                context).output_types[*index];
            let node = ast::Node::AllocTemporary {
                place: ast::Place::Local,
                storage_type: output, 
                operation: remote 
            };

            update_current_allocated(node_id, index, context);
            let resolved = ResolvedScheduleNode {};
            add_schedule_node(resolved, vec![node], context);
        }
        _ => {
            panic!("Unimplemented extract type")
        }
    };
    true
}         

fn explicate_constant(type_id : &usize, 
context : &mut SchedulingContext) -> bool {
    let new_id = ast::RemoteNodeId {
        funclet_id: context.location.funclet_id,
        node_id: context.location.node_id
    };
    let mut nodes = Vec::new();
    let ret_index = get_current_blob(context).schedule.core.nodes.len();
    nodes.push(ast::Node::AllocTemporary { 
        place: ast::Place::Local, 
        storage_type: ffi::TypeId {0: *type_id}, 
        operation: new_id
    });
    nodes.push(ast::Node::EncodeDo { 
        place: ast::Place::Local, 
        operation: ast::RemoteNodeId {
            funclet_id: context.location.funclet_id,
            node_id: context.location.node_id
        }, 
        inputs: Box::new([]), 
        outputs: Box::new([ret_index])
    });
    let resolved = ResolvedScheduleNode {};
    add_schedule_node(resolved, nodes, context);
    true
}

fn explicate_value_function(function_id : &usize, arguments : &Box<[usize]>, 
context : &mut SchedulingContext) -> bool {
    // let id = context.program.funclets.get_next_id();
    // let tail_edge = ast::TailEdge::ScheduleCall { 
    //     value_operation: context.location, 
    //     callee_funclet_id: id,
    //     callee_arguments: Box::new([]), // TODO: clearly wrong
    //     continuation_join: 0
    // };
    // add_current_funclet(tail_edge, context);
    todo!()
}

fn explicate_select(condition : &usize, true_case : &usize, false_case : &usize, 
    context : &mut SchedulingContext) -> bool {
    todo!()
}

// TODO: GPU externals
fn explicate_external(
    external_function_id : &usize,
    dimensions : Option<&Box<[usize]>>, // None in case of CPU things
    arguments : &Box<[usize]>,
    context : &mut SchedulingContext) 
-> bool {
    let external_size = get_external(external_function_id, context)
        .output_types.len();
    let mut allocations = Vec::new();
    // Get the spot of all the return values
    let current_id = context.location.node_id;
    for out_index in 0..external_size {
        match get_current_allocated(&current_id, &out_index, context) {
            None => { return false }
            Some(node_id) => { allocations.push(*node_id) }
        }
    };
    // just so we don't screw this up
    assert_eq!(allocations.len(), external_size);
    let place = match dimensions {
        None => { ast::Place::Local }
        Some(_) => { todo!() }
    };
    let node = ast::Node::EncodeDo { 
        place,
        operation: context.location.clone(),
        inputs: arguments.clone(), 
        outputs: allocations.into_boxed_slice() 
    };
    let resolved = ResolvedScheduleNode {};
    add_schedule_node(resolved, vec![node], context);
    true
}

fn explicate_node(node : &ast::Node, 
context : &mut SchedulingContext) -> bool {
    match node {
        ast::Node::ExtractResult { node_id, 
            index } => explicate_extract_result(
                node_id, 
                index, 
                context),
        ast::Node::ConstantInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ast::Node::ConstantUnsignedInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ast::Node::CallValueFunction { function_id, 
            arguments } => 
            explicate_value_function(function_id, arguments, context),
        ast::Node::Select { condition, true_case, 
            false_case } => 
            explicate_select(condition, true_case, false_case, context),
        ast::Node::CallExternalCpu { external_function_id, 
            arguments } => 
            explicate_external(
                external_function_id, 
                None, 
                arguments,
                context),
        ast::Node::CallExternalGpuCompute { external_function_id, 
            dimensions, arguments } => 
            explicate_external(
                external_function_id,
                Some(dimensions),
                arguments,
                context),
        _ => true
    }
}

fn explicate_nodes(nodes : &Box<[ast::Node]>, 
context : &mut SchedulingContext) -> bool {
    let mut unresolved = false;
    for node in &**nodes {
        if !context.resolved_schedules.contains_key(&context.location.clone()) {
            let node_resolved = explicate_node(node, context);
            if node_resolved {
                let location = context.location.clone();
                let blob = &mut get_current_blob_mut(context);
                blob.remote_update.push(location);
            } else {
                unresolved = true;
            }
        }
        context.location.node_id += 1;
    }
    unresolved
}

fn explicate_funclet(funclet : &ast::Funclet,
context : &mut SchedulingContext) -> bool {
    // Calculates the new funclets to work with (if any)
    context.location.node_id = 0; // reset node_id to new funclet
    let unresolved = match funclet.kind {
        ast::FuncletKind::MixedImplicit => false,
        ast::FuncletKind::MixedExplicit => false,
        ast::FuncletKind::Value => explicate_nodes(&funclet.nodes, context),
        ast::FuncletKind::ScheduleExplicit => false,
        ast::FuncletKind::Inline => false,
        ast::FuncletKind::Timeline => false,
        ast::FuncletKind::Spatial => false,
    };
    unresolved
}

fn cleanup_partials(context : &mut SchedulingContext) {
    let mut funclets = HashMap::new();
    std::mem::swap(&mut funclets, &mut context.new_schedules);
    for (_, mut partial) in funclets.drain() {
        add_blob(partial, context);
    }
}

fn construct_pipeline(context : &mut SchedulingContext) {
    for mut pipeline in context.program.pipelines.iter_mut() {
        match context.resolved_values.get(&pipeline.entry_funclet) {
            Some(resolved) => {
                pipeline.entry_funclet = resolved.schedule_id;
            }
            _ => { panic!("Unresolved funclet") }
        }
    }
}

fn debug_funclets(program : &ast::Program) {
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
        match funclet {
            None => {}
            Some(f) => { println!("{:#?}", f); } 
        }
        id += 1;
    }
    println!("Extras: {:#?}", program.scheduling_funclet_extras);
    println!("Pipelines: {:#?}", program.pipelines);

    panic!("Explicated"); // to see debug information
}

fn should_explicate(program : &mut ast::Program) -> bool {
    let mut any_pipelines = false;
    let mut all_value_pipelines = true;
    for pipeline in program.pipelines.iter() {
        any_pipelines = true;
        match program.funclets.get(&pipeline.entry_funclet) {
            None => { panic!(format!("Undefined funclet {}", 
                pipeline.entry_funclet)); }
            Some(funclet) => {
                match funclet.kind {
                    ast::FuncletKind::Value => {}, // dumb, but whatever
                    _ => { all_value_pipelines = false; }
                }
            }
        }
    }
    any_pipelines && all_value_pipelines
}

pub fn explicate_scheduling(program : &mut ast::Program)
{
    // only explicate if there are pipelines to explicate (value funclet pipes)
    if !should_explicate(program) {
        return
    }
    
    let original = program.funclets.clone();
    let mut initial_location = ast::RemoteNodeId
        { funclet_id : 0, node_id : 0 };
    let starting_index = program.funclets.get_next_id();
    let mut context = 
        SchedulingContext{
            program,
            new_schedules : HashMap::new(),
            location : initial_location,
            resolved_schedules : HashMap::new(),
            resolved_values : HashMap::new()
        };
    let mut unresolved = true; // see if there are any nodes left to resolve
    while unresolved {
        unresolved = false;
        for funclet in original.iter() {
            context.location.funclet_id = *funclet.0;
            unresolved = explicate_funclet(
                funclet.1, &mut context) || unresolved;
        }
    }
    cleanup_partials(&mut context);
    construct_pipeline(&mut context);
    // debug_funclets(&program);
}