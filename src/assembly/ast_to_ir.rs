
use std::any::Any;
use std::collections::HashMap;
use crate::{ir, frontend};
use crate::ir::{ffi, FuncletKind};
use crate::arena::Arena;
use crate::assembly::{ast, context, parser};
use crate::assembly::ast::DictValue;
use crate::assembly::context::Context;

// Utility

fn contains_value(dict : &ast::UncheckedDict, key : &ast::Value) -> bool {
    for pair in dict.iter() {
        if *key == pair.key {
            return true
        }
    }
    false
}

fn contains(dict : &ast::UncheckedDict, key : &str) -> bool {
    contains_value(dict, &ast::Value::ID(key.to_string()))
}

fn remote_conversion(remote : &ast::RemoteNodeId, context : &mut Context) -> ir::RemoteNodeId {
    context.remote_id(remote.funclet_id.clone(), remote.node_id.clone())
}

fn lookup_value(dict : &ast::UncheckedDict, key : &ast::Value) -> ast::DictValue {
    for pair in dict.iter() {
        if *key == pair.key {
            return pair.value.clone()
        }
    }
    panic!(format!("Missing dictionary element {:?}", key));
}

fn lookup_full(dict : &ast::UncheckedDict, key : &str) -> ast::DictValue {
    lookup_value(dict, &ast::Value::ID(key.to_string()))
}

fn as_value(value : ast::DictValue) -> ast::Value {
    match value {
        ast::DictValue::Raw(v) => v,
        _ => panic!(format!("Expected raw value got {:?}", value))
    }
}

fn as_list(value : ast::DictValue) -> Vec<DictValue> {
    match value {
        ast::DictValue::List(v) => v,
        _ => panic!(format!("Expected list got {:?}", value))
    }
}

fn as_dict(value : ast::DictValue) -> ast::UncheckedDict {
    match value {
        ast::DictValue::Dict(d) => d,
        _ => panic!(format!("Expected dict got {:?}", value))
    }
}

fn lookup(dict : &ast::UncheckedDict, key : &str) -> ast::Value {
    as_value(lookup_full(dict, key))
}

/*
	ID(String),
    FunctionLoc(RemoteNodeId),
    VarName(String),
    FnName(String),
    Type(Type),
    Place(ir::Place),
    Stage(ir::ResourceQueueStage),
    Tag(Tag),
 */

fn value_string(v : &ast::Value, context : &mut Context) -> String {
    match v {
        ast::Value::ID(s) => s.clone(),
        _ => panic!(format!("Expected id got {:?}", v))
    }
}

fn value_function_loc(v : &ast::Value, context : &mut Context) -> ir::RemoteNodeId {
    match v {
        ast::Value::FunctionLoc(remote) => ir::RemoteNodeId {
            funclet_id : *context.local_funclet_id(remote.funclet_id.clone()),
            node_id : *context.local_funclet_id(remote.node_id.clone())
        },
        _ => panic!(format!("Expected function location got {:?}", v))
    }
}

fn value_var_name(v : &ast::Value, context : &mut Context) -> usize {
    match v {
        ast::Value::VarName(s) => *context.node_id(s.clone()),
        _ => panic!(format!("Expected variable name got {:?}", v))
    }
}

fn value_funclet_name(v : &ast::Value, context : &mut Context) -> context::Location {
    match v {
        ast::Value::FnName(s) => context.funclet_id(s.clone()),
        _ => panic!(format!("Expected funclet name got {:?}", v))
    }
}

fn value_type(v : &ast::Value, context : &mut Context) -> context::Location {
    match v {
        ast::Value::Type(t) => match t {
            ast::Type::FFI(name) => context::Location::FFI(
                *context.ffi_type_id(name.clone())),
            ast::Type::Local(name) => context::Location::Local(
                *context.local_type_id(name.clone()))
        }
        _ => panic!(format!("Expected type got {:?}", v))
    }
}

fn value_place(v : &ast::Value, context : &mut Context) -> ir::Place {
    match v {
        ast::Value::Place(p) => p.clone(),
        _ => panic!(format!("Expected place got {:?}", v))
    }
}

fn value_stage(v : &ast::Value, context : &mut Context) -> ir::ResourceQueueStage {
    match v {
        ast::Value::Stage(s) => s.clone(),
        _ => panic!(format!("Expected stage got {:?}", v))
    }
}

// This all feels very dumb
fn value_core_tag(v : &ast::TagCore, context : &mut Context) -> ir::ValueTag {
    match v {
        ast::TagCore::None => ir::ValueTag::None,
        ast::TagCore::Operation(r) => ir::ValueTag::Operation {
            remote_node_id:  remote_conversion(r, context)
        },
        ast::TagCore::Input(r) => ir::ValueTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        ast::TagCore::Output(r) => ir::ValueTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

fn timeline_core_tag(v : &ast::TagCore, context : &mut Context) -> ir::TimelineTag {
    match v {
        ast::TagCore::None => ir::TimelineTag::None,
        ast::TagCore::Operation(r) => ir::TimelineTag::Operation {
            remote_node_id:  remote_conversion(r, context)
        },
        ast::TagCore::Input(r) => ir::TimelineTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        ast::TagCore::Output(r) => ir::TimelineTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

fn spatial_core_tag(v : &ast::TagCore, context : &mut Context) -> ir::SpatialTag {
    match v {
        ast::TagCore::None => ir::SpatialTag::None,
        ast::TagCore::Operation(r) => ir::SpatialTag::Operation {
            remote_node_id:  remote_conversion(r, context)
        },
        ast::TagCore::Input(r) => ir::SpatialTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        ast::TagCore::Output(r) => ir::SpatialTag::Input {
            funclet_id : *context.local_funclet_id(r.funclet_id.clone()),
            index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

fn value_value_tag(v : &ast::Value, context : &mut Context) -> ir::ValueTag {
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::ValueTag(v) => {
                match v {
                    ast::ValueTag::Core(c) => value_core_tag(c, context),
                    ast::ValueTag::FunctionInput(r) => ir::ValueTag::FunctionInput {
                        function_id : *context.local_funclet_id(r.funclet_id.clone()),
                        index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
                    },
                    ast::ValueTag::FunctionOutput(r) => ir::ValueTag::FunctionOutput {
                        function_id : *context.local_funclet_id(r.funclet_id.clone()),
                        index: *context.remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
                    },
                    ast::ValueTag::Halt(n) => ir::ValueTag::Halt {
                        index: *context.node_id(n.clone())
                    }
                }
            },
            _ => panic!(format!("Expected value tag got {:?}", v))
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_timeline_tag(v : &ast::Value, context : &mut Context) -> ir::TimelineTag {
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::TimelineTag(t) => match t {
                ast::TimelineTag::Core(c) => timeline_core_tag(c, context)
            },
            _ => panic!(format!("Expected timeline tag got {:?}", v))
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_spatial_tag(v : &ast::Value, context : &mut Context) -> ir::SpatialTag {
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::SpatialTag(t) => match t {
                ast::SpatialTag::Core(c) => spatial_core_tag(c, context)
            },
            _ => panic!(format!("Expected spatial tag got {:?}", v))
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_list<T>(v : ast::DictValue, f : fn(&ast::DictValue, &mut Context) -> T,
                context : &mut Context) -> HashMap<usize, T> {
    let lst = as_list(v);
        let mut result = HashMap::new();
        let index = 0;
        for value in lst.iter() {
            result.insert(index, f(value, context));
        }
        result
}

fn value_index_var_dict<T>(v : ast::DictValue, f : fn(&ast::DictValue, &mut Context) -> T,
                context : &mut Context) -> HashMap<usize, T> {
    let d = as_dict(v);
    let mut result = HashMap::new();
    for pair in d.iter() {
        let index = value_var_name(&pair.key.clone(), context);
        result.insert(index, f(&pair.value.clone(), context));
    };
    result
}

// Translation

fn ir_version(version : &ast::Version, context : &mut Context) -> (u32, u32, u32) {
    (version.major, version.minor, version.detailed)
}

fn ir_external_cpu(external : &ast::ExternalCpuFunction, context : &mut Context)
-> ffi::ExternalCpuFunction {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();

    for name in &external.input_types {
        input_types.push(ffi::TypeId(*context.ffi_type_id(name.clone())))
    }
    for name in &external.output_types {
        output_types.push(ffi::TypeId(*context.ffi_type_id(name.clone())))
    }

    ffi::ExternalCpuFunction {
        name: external.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
    }
}

fn ir_native_interface(program : &ast::Program, context : &mut Context) -> ffi::NativeInterface {
    let mut types = Arena::new();
    let mut external_cpu_functions = Arena::new();
    let mut external_gpu_functions = Arena::new();

    for typ in &program.types {
        match typ {
            ast::TypeDecl::FFI(t) => {
                types.create(parser::read_ffi_raw(t.clone(), context));
            }
            _ => {}
        }
    }

    for def in &program.funclets {
        match def {
            ast::FuncletDef::ExternalCPU(external) => {
                external_cpu_functions.create(ir_external_cpu(external, context));
            },
            ast::FuncletDef::ExternalGPU(external) => {
                todo!()
            },
            _ => {}
        }
    }

    ffi::NativeInterface {
        types,
        external_cpu_functions,
        external_gpu_functions
    }
}

fn ir_types(types : &Vec<ast::TypeDecl>, context : &mut Context) -> Arena<ir::Type> {
    let mut result = Arena::new();
    for type_decl in types {
        let new_type = match type_decl {
            ast::TypeDecl::Local(typ) => {
                match typ.event
                { // only supported custom types atm
                    true => {
                    ir::Type::Event {
                        place: value_place(&lookup(&typ.data, "place"), context),
                    }
                },
                    false => {
                    ir::Type::Slot {
                        storage_type: ffi::TypeId(value_type(
                            &lookup(&typ.data, "type"), context).unpack()),
                        queue_stage: value_stage(&lookup(&typ.data, "stage"), context),
                        queue_place: value_place(&lookup(&typ.data, "place"), context),
                    }
                }
                }
            }
            ast::TypeDecl::FFI(name) => {
                ir::Type::NativeValue {
                    storage_type: ffi::TypeId(*context.ffi_type_id(name.clone()))
                }
            }
        };
        result.create(new_type);
    }
    result
}

fn ir_node(node : &ast::Node, context : &mut Context) -> ir::Node {
    match node {
        ast::Node::None => ir::Node::None,
        ast::Node::Phi { index } => ir::Node::Phi { index: *index },
        ast::Node::ExtractResult { node_id , index } => {
            ir::Node::ExtractResult {
                node_id: *context.node_id(node_id.clone()),
                index: *index,
            }
        },
        ast::Node::ConstantInteger { value, type_id } => {
            ir::Node::ConstantInteger {
                value: *value,
                type_id: *context.loc_type_id(type_id.clone()),
            }
        },
        ast::Node::ConstantUnsignedInteger { value, type_id } => {
            ir::Node::ConstantUnsignedInteger {
                value: *value,
                type_id: *context.loc_type_id(type_id.clone()),
            }
        },
        ast::Node::CallValueFunction { function_id, arguments } => {
            todo!()
        },
        ast::Node::Select { condition, true_case, false_case } => {
            ir::Node::Select {
                condition: *context.node_id(condition.clone()),
                true_case: *context.node_id(true_case.clone()),
                false_case: *context.node_id(false_case.clone()),
            }
        },
        ast::Node::CallExternalCpu { external_function_id, arguments } => {
            ir::Node::CallExternalCpu {
                external_function_id: *context.cpu_funclet_id(external_function_id.clone()),
                arguments: arguments.iter().map(|n| *context.node_id(n.clone())).collect(),
            }
        },
        ast::Node::CallExternalGpuCompute { external_function_id,
            dimensions, arguments } => {
            ir::Node::CallExternalGpuCompute {
                external_function_id: *context.gpu_funclet_id(external_function_id.clone()),
                dimensions: dimensions.iter().map(|n| *context.node_id(n.clone())).collect(),
                arguments: arguments.iter().map(|n| *context.node_id(n.clone())).collect(),
            }
        },
        ast::Node::AllocTemporary { place, storage_type, operation } => {
            ir::Node::AllocTemporary {
                place: place.clone(),
                storage_type: ffi::TypeId(*context.loc_type_id(storage_type.clone())),
                operation: remote_conversion(operation, context),
            }
        },
        ast::Node::UnboundSlot { place, storage_type, operation } => {
            ir::Node::UnboundSlot {
                place: place.clone(),
                storage_type: ffi::TypeId(*context.loc_type_id(storage_type.clone())),
                operation: remote_conversion(operation, context),
            }
        },
        ast::Node::Drop { node } => {
            ir::Node::Drop {
                node: *context.node_id(node.clone()),
            }
        },
        ast::Node::StaticAllocFromStaticBuffer { buffer, place,
            storage_type, operation } => {
            ir::Node::StaticAllocFromStaticBuffer {
                buffer: *context.node_id(buffer.clone()),
                place: place.clone(),
                storage_type: ffi::TypeId(*context.loc_type_id(storage_type.clone())),
                operation: remote_conversion(operation, context),
            }
        },
        ast::Node::EncodeDo { place, operation,
            inputs, outputs } => {
            ir::Node::EncodeDo {
                place : place.clone(),
                operation: remote_conversion(operation, context),
                inputs: inputs.iter().map(|n| *context.node_id(n.clone())).collect(),
                outputs: inputs.iter().map(|n| *context.node_id(n.clone())).collect(),
            }
        },
        ast::Node::EncodeCopy { place, input, output } => {
            ir::Node::EncodeCopy {
                place: place.clone(),
                input: *context.node_id(input.clone()),
                output: *context.node_id(output.clone()),
            }
        },
        ast::Node::Submit { place, event } => {
            ir::Node::Submit {
                place: place.clone(),
                event: remote_conversion(event, context)
            }
        },
        ast::Node::EncodeFence { place, event } => {
            ir::Node::EncodeFence {
                place: place.clone(),
                event: remote_conversion(event, context)
            }
        },
        ast::Node::SyncFence { place, fence, event } => {
            ir::Node::SyncFence {
                place: place.clone(),
                fence: *context.node_id(fence.clone()),
                event: remote_conversion(event, context)
            }
        },
        ast::Node::InlineJoin { funclet, captures, continuation } => {
            ir::Node::InlineJoin {
                funclet : *context.local_funclet_id(funclet.clone()),
                captures: captures.iter().map(|n| *context.node_id(n.clone())).collect(),
                continuation: *context.node_id(continuation.clone())
            }
        },
        ast::Node::SerializedJoin { funclet, captures, continuation } => {
            ir::Node::SerializedJoin {
                funclet : *context.local_funclet_id(funclet.clone()),
                captures: captures.iter().map(|n| *context.node_id(n.clone())).collect(),
                continuation: *context.node_id(continuation.clone())
            }
        },
        ast::Node::DefaultJoin => { ir::Node::DefaultJoin },
        ast::Node::SubmissionEvent { here_place, there_place, local_past } => {
            ir::Node::SubmissionEvent {
                here_place : here_place.clone(),
                there_place : there_place.clone(),
                local_past : *context.node_id(local_past.clone()),
            }
        },
        ast::Node::SynchronizationEvent { here_place, there_place,
            local_past, remote_local_past } => {
            ir::Node::SynchronizationEvent {
                here_place : here_place.clone(),
                there_place : there_place.clone(),
                local_past : *context.node_id(local_past.clone()),
                remote_local_past : *context.node_id(remote_local_past.clone())
            }
        },
        ast::Node::SeparatedLinearSpace { place, space } => {
            ir::Node::SeparatedLinearSpace {
                place : place.clone(),
                space : *context.node_id(space.clone())
            }
        },
        ast::Node::MergedLinearSpace { place, spaces } => {
            ir::Node::MergedLinearSpace {
                place : place.clone(),
                spaces : spaces.iter().map(|n| *context.node_id(n.clone())).collect()
            }
        },
    }
}

fn ir_tail_edge(tail : &ast::TailCommand, context : &mut Context) -> ir::TailEdge {
    match tail {
        ast::TailCommand::Return { var } => {
            ir::TailEdge::Return { return_values: Box::new([*context.node_id(var.clone())]) }
        }
    }
}

fn ir_funclet(funclet : &ast::Funclet, context : &mut Context) -> ir::Funclet {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut nodes = Vec::new();
    let mut tail_edge = None;

    for input_type in &funclet.header.args {
        input_types.push(*context.loc_type_id(input_type.clone()))
    }

    output_types.push(*context.loc_type_id(funclet.header.ret.clone()));

    let mut index = 0;
    if funclet.commands.len() == 0 {
        panic!(format!("Empty funclet {:?}", funclet.header.name));
    }
    let l = funclet.commands.len() - 1;
    for command in &funclet.commands {
        match command {
            ast::Command::IRNode(n) => {
                if index < l {
                    nodes.push(ir_node(n, context));
                } else {
                    panic!(format!("Last command must be a tail edge in {:?}", funclet.header.name));
                }
            },
            ast::Command::Tail(t) => {
                if index == l {
                    tail_edge = Some(ir_tail_edge(t, context));
                } else {
                    panic!(format!("Unexpected tail edge before the last command in {:?}",
                                   funclet.header.name));
                }
            }
        }

        index += 1;
    }

    ir::Funclet {
        kind: funclet.kind.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        tail_edge: tail_edge.unwrap(), // actually safe, oddly enough
    }
}

fn ir_funclets(funclets : &ast::FuncletDefs, context : &mut Context) -> Arena<ir::Funclet> {
    let mut result = Arena::new();
    for def in funclets {
        match def {
            ast::FuncletDef::Local(f) => {
                context.update_local_funclet(f.header.name.clone());
                result.create(ir_funclet(f, context)); },
            _ => {}
        }
    }
    context.clear_local_funclet();
    result
}

fn ir_pipelines(pipelines : &ast::Pipelines, context : &mut Context) -> Vec<ir::Pipeline> {
    let mut result = Vec::new();
    for pipeline in pipelines.iter() {
        let new_pipeline = ir::Pipeline {
            name: pipeline.name.clone(),
            entry_funclet: *context.local_funclet_id(pipeline.funclet.clone()),
            yield_points: Default::default(),
        };
        result.push(new_pipeline);
    }
    result
}

fn ir_value_extra(extra: &ast::UncheckedDict, context : &mut Context) -> ir::ValueFuncletExtra {
    todo!()
}

fn ir_value_extras(funclets : &ast::FuncletDefs, extras : &ast::Extras, context : &mut Context)
-> HashMap<ir::FuncletId, ir::ValueFuncletExtra> {
    let mut result = HashMap::new();
    for funclet in funclets {
        match funclet {
            ast::FuncletDef::Local(f) => {
                match f.kind {
                    ir::FuncletKind::Value => {
                        let name = f.header.name.clone();
                        for extra in extras {
                            if extra.name == name {
                                context.update_local_funclet(extra.name.clone());
                                let index = context.local_funclet_id(extra.name.clone());
                                if result.contains_key(index) {
                                    panic!(format!("Duplicate extras for {:?}", name));
                                }
                                result.insert(*index, ir_value_extra(&extra.data, context));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    result
}

fn ir_slot_info(v : &ast::DictValue, context : &mut Context) -> ir::SlotInfo {
    let d = &as_dict(v.clone());
    let mut value_tag = ir::ValueTag::None;
    let mut timeline_tag = ir::TimelineTag::None;
    let mut spatial_tag = ir::SpatialTag::None;
    if contains(d, "value") {
        value_tag = value_value_tag(&lookup(d, "value"), context);
    }
    if contains(d, "timeline") {
        timeline_tag = value_timeline_tag(&lookup(d, "timeline"), context);
    }
    if contains(d, "spatial") {
        spatial_tag = value_spatial_tag(&lookup(d, "spatial"), context);
    }
    ir::SlotInfo { value_tag, timeline_tag, spatial_tag }
}

fn fence_info(v : &ast::DictValue, context : &mut Context) -> ir::FenceInfo {
    let d = &as_dict(v.clone());
    let timeline_tag = value_timeline_tag(&lookup(d, "timeline"), context);
    ir::FenceInfo { timeline_tag }
}

fn buffer_info(v : &ast::DictValue, context : &mut Context) -> ir::BufferInfo {
    let d = &as_dict(v.clone());
    let spatial_tag = value_spatial_tag(&lookup(d, "spatial"), context);
    ir::BufferInfo { spatial_tag }
}

fn ir_scheduling_extra(id : &usize, d: &ast::UncheckedDict, context : &mut Context) -> ir::SchedulingFuncletExtra {
    ir::SchedulingFuncletExtra {
        value_funclet_id: *id,
        input_slots: value_index_var_dict(lookup_full(d, "input_slots"), ir_slot_info, context),
        output_slots: value_list(lookup_full(d, "output_slots"), ir_slot_info, context),
        input_fences: value_index_var_dict(lookup_full(d, "input_fences"), fence_info, context),
        output_fences: value_index_var_dict(lookup_full(d, "output_fences"), fence_info, context),
        input_buffers: value_index_var_dict(lookup_full(d, "input_buffers"), buffer_info, context),
        output_buffers: value_index_var_dict(lookup_full(d, "output_buffers"), buffer_info, context),
        in_timeline_tag: value_timeline_tag(&lookup(d, "in_timeline_tag"), context),
        out_timeline_tag: value_timeline_tag(&lookup(d, "out_timeline_tag"), context),
    }
}

fn ir_scheduling_extras(funclets : &ast::FuncletDefs, extras : &ast::Extras, context : &mut Context)
-> HashMap<ir::FuncletId, ir::SchedulingFuncletExtra> {
    // duplicating some code...but it's annoying to fix and I'm lazy
    let mut result = HashMap::new();
    for funclet in funclets {
        match funclet {
            ast::FuncletDef::Local(f) => {
                match f.kind {
                    ir::FuncletKind::ScheduleExplicit => {
                        let name = f.header.name.clone();
                        for extra in extras {
                            if extra.name == name {
                                context.update_local_funclet(extra.name.clone());
                                let index = context.local_funclet_id(extra.name.clone());
                                if result.contains_key(index) {
                                    panic!(format!("Duplicate extras for {:?}", name));
                                }
                                result.insert(
                                    *index,
                                    ir_scheduling_extra(&index.clone(), &extra.data, context)
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    result
}

fn ir_program(program : ast::Program, context : &mut Context) -> ir::Program {
    ir::Program {
        native_interface: ir_native_interface(&program, context),
        types: ir_types(&program.types, context),
        funclets: ir_funclets(&program.funclets, context),
        value_functions: Arena::new(),
        pipelines: ir_pipelines(&program.pipelines, context),
        value_funclet_extras: ir_value_extras(&program.funclets, &program.extras, context),
        scheduling_funclet_extras: ir_scheduling_extras(&program.funclets, &program.extras, context),
    }
}

pub fn ast_to_ir(program : ast::Program, context : &mut Context) -> frontend::Definition {
    frontend::Definition {
        version: ir_version(&program.version, context),
        program: ir_program(program, context),
    }
}