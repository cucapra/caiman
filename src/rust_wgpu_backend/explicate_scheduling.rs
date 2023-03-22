use crate::ir;
use std::{collections::{HashMap}, any, hash::Hash};
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
    remote_update : Vec<ir::RemoteNodeId>,
    allocated : HashMap<usize, HashMap<usize, usize>>
}

#[derive(Debug)]
struct SchedulingContext<'a> {
    program : &'a mut ir::Program,
    new_schedules : HashMap<usize, ScheduleBlob>,
    location : ir::RemoteNodeId,
    // scheduled node maps
    resolved_schedules : HashMap<ir::RemoteNodeId, ResolvedScheduleNode>,
    resolved_values : HashMap<usize, ResolvedValueNode>
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
    // todo: extend?
    let mut input_slots_default = empty_slot();
    information.input_slots.insert(0, input_slots_default);
    information.output_slots.insert(0, empty_slot());
    information
}

fn new_blob(value_index : &usize) -> ScheduleBlob {
    let schedule = new_partial_schedule();
    let information = new_information(value_index);
    let timeline = new_partial_timeline();
    ScheduleBlob {
        schedule,
        information,
        timeline,
        remote_update: Vec::new(),
        allocated: HashMap::new()
    }
}

fn get_blob<'a>(value_index : &usize,
                context : &'a mut SchedulingContext) -> &'a ScheduleBlob {
    context.new_schedules.entry(*value_index).
        or_insert(new_blob(value_index))
}

fn get_blob_mut<'a>(value_index : &usize,
                    context : &'a mut SchedulingContext) -> &'a mut ScheduleBlob {
    context.new_schedules.entry(*value_index).
        or_insert(new_blob(value_index))
}

fn get_current_blob<'a>(context : &'a mut SchedulingContext)
                        -> &'a ScheduleBlob {
    let value_index = context.location.funclet_id;
    get_blob(&value_index, context)
}

fn get_current_blob_mut<'a>(context : &'a mut SchedulingContext)
                            -> &'a mut ScheduleBlob {
    let value_index = context.location.funclet_id;
    get_blob_mut(&value_index, context)
}

fn get_funclet<'a>(index : &usize, context : &'a SchedulingContext)
                   -> &'a ir::Funclet {
    context.program.funclets.get(*index).unwrap()
}

fn get_current_funclet<'a>(context : &'a SchedulingContext)
                           -> &'a ir::Funclet {
    let index = context.location.funclet_id;
    get_funclet(&index, context)
}

fn get_external<'a>(external_function_id: &usize,
                    context: &'a SchedulingContext) -> &'a ffi::ExternalCpuFunction {
    context.program.native_interface.external_cpu_functions.
        get(*external_function_id).unwrap()
}

fn get_current_allocated<'a>(node_id : &usize, argument : &usize,
                             context : &'a mut SchedulingContext) -> Option<&'a usize> {
    let blob = get_current_blob_mut(context);
    match blob.allocated.get(node_id) {
        None => { None }
        Some(map) => { map.get(argument) }
    }
}

fn update_current_allocated(node_id : &usize, index : &usize,
                            context : &mut SchedulingContext) {
    // Assumes we _haven't_ added this node yet
    let blob = get_current_blob_mut(context);
    let current_node = blob.schedule.core.nodes.len();
    let entry = blob.allocated.entry(*node_id).
        or_insert(HashMap::new());
    entry.insert(*index, current_node);
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
    let mut return_values = Box::new([0]);
    if node_count > 0 {
        return_values[0] = node_count-1;
    }
    ir::TailEdge::Return {
        return_values  // zero-indexing
    }
}

fn build_funclet(core : PartialData, kind : ir::FuncletKind,
                 context : &mut SchedulingContext) -> ir::Funclet {
    let default = default_tail(&core, context);
    ir::Funclet {
        kind,
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
    let schedule_id = context.program.funclets.add(new_schedule);
    let timeline_id = context.program.funclets.add(new_timeline);
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
    let remote = ir::RemoteNodeId {
        funclet_id : context.location.funclet_id,
        node_id : context.location.node_id
    };
    let node = &get_current_funclet(context).nodes[*node_id];
    match node {
        ir::Node::CallExternalCpu {
            external_function_id,
            arguments } => {
            let output = get_external(external_function_id,
                                      context).output_types[*index];
            let node = ir::Node::AllocTemporary {
                place: ir::Place::Local,
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
    let new_id = ir::RemoteNodeId {
        funclet_id: context.location.funclet_id,
        node_id: context.location.node_id
    };
    let mut nodes = Vec::new();
    let ret_index = get_current_blob(context).schedule.core.nodes.len();
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
        outputs: Box::new([ret_index])
    });
    let resolved = ResolvedScheduleNode {};
    add_schedule_node(resolved, nodes, context);
    true
}

fn explicate_value_function(function_id : &usize, arguments : &Box<[usize]>,
                            context : &mut SchedulingContext) -> bool {
    // let id = context.program.funclets.get_next_id();
    // let tail_edge = ir::TailEdge::ScheduleCall {
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
        None => { ir::Place::Local }
        Some(_) => { todo!() }
    };
    let node = ir::Node::EncodeDo {
        place,
        operation: context.location.clone(),
        inputs: arguments.clone(),
        outputs: allocations.into_boxed_slice()
    };
    let resolved = ResolvedScheduleNode {};
    add_schedule_node(resolved, vec![node], context);
    true
}

fn explicate_node(node : &ir::Node,
                  context : &mut SchedulingContext) -> bool {
    match node {
        ir::Node::ExtractResult { node_id,
            index } => explicate_extract_result(
            node_id,
            index,
            context),
        ir::Node::Constant { value,
                type_id } => explicate_constant(type_id, context),
        ir::Node::CallValueFunction { function_id,
            arguments } =>
            explicate_value_function(function_id, arguments, context),
        ir::Node::Select { condition, true_case,
            false_case } =>
            explicate_select(condition, true_case, false_case, context),
        ir::Node::CallExternalCpu { external_function_id,
            arguments } =>
            explicate_external(
                external_function_id,
                None,
                arguments,
                context),
        ir::Node::CallExternalGpuCompute { external_function_id,
            dimensions, arguments } =>
            explicate_external(
                external_function_id,
                Some(dimensions),
                arguments,
                context),
        _ => true
    }
}

fn explicate_nodes(nodes : &Box<[ir::Node]>,
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

fn should_explicate(program : &mut ir::Program) -> bool {
    let mut any_pipelines = false;
    let mut all_value_pipelines = true;
    for pipeline in program.pipelines.iter() {
        any_pipelines = true;
        match program.funclets.get(pipeline.entry_funclet) {
            None => { panic!(format!("Undefined funclet {}",
                                     pipeline.entry_funclet)); }
            Some(funclet) => {
                match funclet.kind {
                    ir::FuncletKind::Value => {}, // dumb, but whatever
                    _ => { all_value_pipelines = false; }
                }
            }
        }
    }
    any_pipelines && all_value_pipelines
}

pub fn explicate_scheduling(program : &mut ir::Program)
{
    // only explicate if there are pipelines to explicate (value funclet pipes)
    if !should_explicate(program) {
        return
    }

    let original = program.funclets.clone();
    let mut initial_location = ir::RemoteNodeId
    { funclet_id : 0, node_id : 0 };
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
            context.location.funclet_id = funclet.0;
            unresolved = explicate_funclet(
                funclet.1, &mut context) || unresolved;
        }
    }
    cleanup_partials(&mut context);
    construct_pipeline(&mut context);
    panic!("Please don't explicate yet!")
}