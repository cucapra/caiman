
use std::any::Any;
use std::collections::HashMap;
use crate::{ast, frontend, ir};
use crate::ir::{ffi, FuncletKind};
use crate::assembly::{context, parser};
use crate::ast::{DictValue, FFIType};
use crate::assembly::context::{Context, FuncletLocation};

// for reading GPU stuff
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::stable_vec::StableVec;

// Utility

fn ffi_to_ffi(value : ast::FFIType, context : &mut Context) -> ffi::Type {
    fn box_map(b : Box<[ast::FFIType]>, context : &mut Context) -> Box<[ffi::TypeId]> {
        b.iter().map(|x| ffi::TypeId(*context.ffi_type_id(x))).collect()
    }
    fn type_id(element_type : Box<ast::FFIType>, context : &mut Context) -> ffi::TypeId {
        ffi::TypeId(*context.ffi_type_id(element_type.as_ref()))
    }
    match value {
        FFIType::F32 => ffi::Type::F32,
        FFIType::F64 => ffi::Type::F64,
        FFIType::U8 => ffi::Type::U8,
        FFIType::U16 => ffi::Type::U16,
        FFIType::U32 => ffi::Type::U32,
        FFIType::U64 => ffi::Type::U64,
        FFIType::USize => ffi::Type::USize,
        FFIType::I8 => ffi::Type::I8,
        FFIType::I16 => ffi::Type::I16,
        FFIType::I32 => ffi::Type::I32,
        FFIType::I64 => ffi::Type::I64,
        FFIType::Array { element_type, length } => ffi::Type::Array {
            element_type: type_id(element_type, context),
            length
        },
        FFIType::ErasedLengthArray(element_type) =>
            ffi::Type::ErasedLengthArray { element_type: type_id(element_type, context) },
        FFIType::Struct { fields,
            byte_alignment,
            byte_size } => todo!(),
        FFIType::Tuple(element_types) => ffi::Type::Tuple {
            fields: box_map(element_types.into_boxed_slice(), context),
        },
        FFIType::ConstRef(element_type) =>
            ffi::Type::ConstRef { element_type: type_id(element_type, context) },
        FFIType::MutRef(element_type) =>
            ffi::Type::MutRef { element_type: type_id(element_type, context) },
        FFIType::ConstSlice(element_type) =>
            ffi::Type::ConstSlice { element_type: type_id(element_type, context) },
        FFIType::MutSlice(element_type) =>
            ffi::Type::MutSlice { element_type: type_id(element_type, context) },
        FFIType::GpuBufferRef(element_type) =>
            ffi::Type::GpuBufferRef { element_type: type_id(element_type, context) },
        FFIType::GpuBufferSlice(element_type) =>
            ffi::Type::GpuBufferSlice { element_type: type_id(element_type, context) },
        FFIType::GpuBufferAllocator => ffi::Type::GpuBufferAllocator
    }
}

fn as_key(k : &str) -> ast::Value {
    ast::Value::ID(k.to_string())
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

fn remote_conversion(remote : &ast::RemoteNodeId, context : &mut Context) -> ir::RemoteNodeId {
    context.remote_id(remote.funclet_id.clone(), remote.node_id.clone())
}

fn value_string(d : &ast::DictValue, context : &mut Context) -> String {
    let v = as_value(d.clone());
    match v {
        ast::Value::ID(s) => s.clone(),
        _ => panic!(format!("Expected id got {:?}", v))
    }
}

fn value_num(d : &ast::DictValue, context : &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        ast::Value::Num(n) => n.clone(),
        _ => panic!(format!("Expected num got {:?}", v))
    }
}

fn value_function_loc(d : &ast::DictValue, context : &mut Context) -> ir::RemoteNodeId {
    let v = as_value(d.clone());
    match v {
        ast::Value::FunctionLoc(remote) => ir::RemoteNodeId {
            funclet_id : *context.local_funclet_id(remote.funclet_id.clone()),
            node_id : *context.local_funclet_id(remote.node_id.clone())
        },
        _ => panic!(format!("Expected function location got {:?}", v))
    }
}

fn value_var_name(d : &ast::DictValue, context : &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        ast::Value::VarName(s) => *context.node_id(s.clone()),
        _ => panic!(format!("Expected variable name got {:?}", v))
    }
}

fn value_funclet_name(d : &ast::DictValue, context : &mut Context) -> context::FuncletLocation {
    let v = as_value(d.clone());
    match v {
        ast::Value::FnName(s) => context.funclet_id(s.clone()).clone(),
        _ => panic!(format!("Expected funclet name got {:?}", v))
    }
}

fn value_type(d : &ast::DictValue, context : &mut Context) -> context::Location {
    let v = as_value(d.clone());
    match v {
        ast::Value::Type(t) => match t {
            ast::Type::FFI(typ) => context::Location::FFI(
                *context.ffi_type_id(&typ)),
            ast::Type::Local(name) => context::Location::Local(
                *context.local_type_id(name.clone()))
        }
        _ => panic!(format!("Expected type got {:?}", v))
    }
}

fn value_place(d : &ast::DictValue, context : &mut Context) -> ir::Place {
    let v = as_value(d.clone());
    match v {
        ast::Value::Place(p) => p.clone(),
        _ => panic!(format!("Expected place got {:?}", v))
    }
}

fn value_stage(d : &ast::DictValue, context : &mut Context) -> ir::ResourceQueueStage {
    let v = as_value(d.clone());
    match v {
        ast::Value::Stage(s) => s.clone(),
        _ => panic!(format!("Expected stage got {:?}", v))
    }
}

// This all feels very dumb
fn value_core_tag(v : ast::TagCore, context : &mut Context) -> ir::ValueTag {
    match v {
        ast::TagCore::None => ir::ValueTag::None,
        ast::TagCore::Operation(r) => ir::ValueTag::Operation {
            remote_node_id:  remote_conversion(&r, context)
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

fn timeline_core_tag(v : ast::TagCore, context : &mut Context) -> ir::TimelineTag {
    match v {
        ast::TagCore::None => ir::TimelineTag::None,
        ast::TagCore::Operation(r) => ir::TimelineTag::Operation {
            remote_node_id:  remote_conversion(&r, context)
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

fn spatial_core_tag(v : ast::TagCore, context : &mut Context) -> ir::SpatialTag {
    match v {
        ast::TagCore::None => ir::SpatialTag::None,
        ast::TagCore::Operation(r) => ir::SpatialTag::Operation {
            remote_node_id:  remote_conversion(&r, context)
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

fn value_value_tag(t : &ast::ValueTag, context : &mut Context) -> ir::ValueTag {
    match t {
        ast::ValueTag::Core(c) => value_core_tag(c.clone(), context),
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
}

fn value_dict_value_tag(d : &ast::DictValue, context : &mut Context) -> ir::ValueTag {
    let v = as_value(d.clone());
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::ValueTag(v) => {
                value_value_tag(&v, context)
            },
            _ => panic!(format!("Expected value tag got {:?}", d))
        },
        _ => panic!(format!("Expected tag got {:?}", d))
    }
}

fn value_timeline_tag(t : &ast::TimelineTag, context : &mut Context) -> ir::TimelineTag {
    match t {
        ast::TimelineTag::Core(c) => timeline_core_tag(c.clone(), context)
    }
}

fn value_dict_timeline_tag(d : &ast::DictValue, context : &mut Context) -> ir::TimelineTag {
    let v = as_value(d.clone());
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::TimelineTag(t) => value_timeline_tag(&t, context),
            _ => panic!(format!("Expected timeline tag got {:?}", d))
        },
        _ => panic!(format!("Expected tag got {:?}", d))
    }
}

fn value_spatial_tag(t : &ast::SpatialTag, context : &mut Context) -> ir::SpatialTag {
    match t {
        ast::SpatialTag::Core(c) => spatial_core_tag(c.clone(), context)
    }
}

fn value_dict_spatial_tag(d : &ast::DictValue, context : &mut Context) -> ir::SpatialTag {
    let v = as_value(d.clone());
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::SpatialTag(t) => value_spatial_tag(&t, context),
            _ => panic!(format!("Expected spatial tag got {:?}", d))
        },
        _ => panic!(format!("Expected tag got {:?}", d))
    }
}

fn value_slot_info(d : &ast::DictValue, context : &mut Context) -> ir::SlotInfo {
    let v = as_value(d.clone());
    match v {
        ast::Value::SlotInfo(s) => ir::SlotInfo {
            value_tag: value_value_tag(&s.value_tag, context),
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_fence_info(d : &ast::DictValue, context : &mut Context) -> ir::FenceInfo {
    let v = as_value(d.clone());
    match v {
        ast::Value::FenceInfo(s) => ir::FenceInfo {
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_buffer_info(d : &ast::DictValue, context : &mut Context) -> ir::BufferInfo {
    let v = as_value(d.clone());
    match v {
        ast::Value::BufferInfo(s) => ir::BufferInfo {
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!(format!("Expected tag got {:?}", v))
    }
}

fn value_list<T>(v : &ast::DictValue, f : fn(&ast::DictValue, &mut Context) -> T,
                context : &mut Context) -> HashMap<usize, T> {
    let lst = as_list(v.clone());
    let mut result = HashMap::new();
    let index = 0;
    for value in lst.iter() {
        result.insert(index, f(value, context));
    }
    result
}

fn value_index_var_dict<T>(v : &ast::DictValue, f : fn(&ast::DictValue, &mut Context) -> T,
                context : &mut Context) -> HashMap<usize, T> {
    let d = as_dict(v.clone());
    let mut result = HashMap::new();
    for pair in d.iter() {
        let index = value_var_name(&ast::DictValue::Raw(pair.0.clone()), context);
        result.insert(index, f(&pair.1.clone(), context));
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
        input_types.push(ffi::TypeId(*context.ffi_type_id(name)))
    }
    for name in &external.output_types {
        output_types.push(ffi::TypeId(*context.ffi_type_id(name)))
    }

    ffi::ExternalCpuFunction {
        name: external.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
    }
}

fn ir_external_gpu_resource(d : &ast::UncheckedDict, arg_names : &Vec<String>, context : &mut Context)
    -> ffi::ExternalGpuFunctionResourceBinding {
    fn local_name(d : &ast::DictValue, arg_names : &Vec<String>) -> usize {
        let v = as_value(d.clone());
        match v {
            ast::Value::VarName(n) => {
                let mut index = 0;
                for arg in arg_names {
                    if n == *arg {
                        return index
                    }
                    index += 1;
                }
                panic!(format!("Unknown GPU variable {:?}", n))
            }
            _ => panic!(format!("Invalid argument {:?}", d))
        }
    }
    let group = value_num(d.get(&as_key("group")).unwrap(), context);
    let binding = value_num(d.get(&as_key("binding")).unwrap(), context);
    let mut input = None;
    if d.contains_key(&as_key("input")) {
        input = Some(local_name(d.get(&as_key("input")).unwrap(), arg_names));
    }
    let mut output = None;
    if d.contains_key(&as_key("output")) {
        output = Some(local_name(d.get(&as_key("output")).unwrap(), arg_names));
    }
    ffi::ExternalGpuFunctionResourceBinding { group, binding, input, output }
}

fn ir_external_gpu(external : &ast::ExternalGpuFunction, context : &mut Context)
    -> ffi::ExternalGpuFunction {
    let mut input_types = Vec::new();
    let mut arg_names = Vec::new();
    let mut output_types = Vec::new();
    let mut resource_bindings = Vec::new();

    for name in &external.input_args {
        input_types.push(ffi::TypeId(*context.ffi_type_id(&name.0)));
        arg_names.push(name.1.clone());
    }
    for name in &external.output_types {
        output_types.push(ffi::TypeId(*context.ffi_type_id(name)))
    }
    for resource in &external.resource_bindings {
        resource_bindings.push(ir_external_gpu_resource(resource, &arg_names, context));
    }

    // Very silly
    let input_path = Path::new(&external.shader_module);
    assert!(external.shader_module.ends_with(".wgsl"));
    let mut input_file = match File::open(& input_path)
    {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file
    };
    let mut content = String::new();
    match input_file.read_to_string(&mut content)
    {
        Err(why) => panic!("Couldn't read file: {}", why),
        Ok(_) => ()
    };

    ffi::ExternalGpuFunction {
        name: external.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        entry_point: external.entry_point.clone(),
        resource_bindings : resource_bindings.into_boxed_slice(),
        shader_module_content : ffi::ShaderModuleContent::Wgsl(content),
    }
}

fn ir_native_interface(program : &ast::Program, context : &mut Context) -> ffi::NativeInterface {
    let mut types = StableVec::new();
    let mut external_cpu_functions = StableVec::new();
    let mut external_gpu_functions = StableVec::new();

    for typ in &program.types {
        match typ {
            ast::TypeDecl::FFI(t) => {
                types.add(ffi_to_ffi(t.clone(), context));
            }
            _ => {}
        }
    }

    for def in &program.funclets {
        match def {
            ast::FuncletDef::ExternalCPU(external) => {
                external_cpu_functions.add(ir_external_cpu(external, context));
            },
            ast::FuncletDef::ExternalGPU(external) => {
                external_gpu_functions.add(ir_external_gpu(external, context));
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

fn ir_types(types : &Vec<ast::TypeDecl>, context : &mut Context) -> StableVec<ir::Type> {
    let mut result = StableVec::new();
    for type_decl in types {
        let new_type = match type_decl {
            ast::TypeDecl::Local(typ) => {
                match typ.event
                { // only supported custom types atm
                    true => {
                    ir::Type::Event {
                        place: value_place(
                            typ.data.get(&as_key("place")).unwrap(), context),
                    }
                },
                    false => {
                    ir::Type::Slot {
                        storage_type: ffi::TypeId(value_type(
                            typ.data.get(&as_key("type")).unwrap(), context).unpack()),
                        queue_stage: value_stage(typ.data.get(&as_key("stage")).unwrap(), context),
                        queue_place: value_place(typ.data.get(&as_key("place")).unwrap(), context),
                    }
                }
                }
            }
            ast::TypeDecl::FFI(name) => {
                ir::Type::NativeValue {
                    storage_type: ffi::TypeId(*context.ffi_type_id(name))
                }
            }
        };
        result.add(new_type);
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
            unimplemented!() // arbitrary trick for unification of calls
        },
        ast::Node::Select { condition, true_case, false_case } => {
            ir::Node::Select {
                condition: *context.node_id(condition.clone()),
                true_case: *context.node_id(true_case.clone()),
                false_case: *context.node_id(false_case.clone()),
            }
        },
        ast::Node::CallExternalCpu { external_function_id, arguments } => {
            let name = external_function_id.clone();
            let external_function_id = *match context.funclet_id(name.clone()) {
                FuncletLocation::Local(_) => panic!(format!("Cannot directly call local funclet {}", name)),
                FuncletLocation::ValueFun(i) => i,
                FuncletLocation::CpuFun(i) => i,
                FuncletLocation::GpuFun(i) => i
            };
            ir::Node::CallExternalCpu {
                external_function_id,
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
                outputs: outputs.iter().map(|n| *context.node_id(n.clone())).collect(),
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

fn ir_tail_edge(tail : &ast::TailEdge, context : &mut Context) -> ir::TailEdge {
    match tail {
        ast::TailEdge::Return { var } => {
            ir::TailEdge::Return { return_values: Box::new([*context.node_id(var.clone())]) }
        }
    }
}

fn ir_funclet(funclet : &ast::Funclet, context : &mut Context) -> ir::Funclet {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut nodes = Vec::new();

    let mut index = 0;
    for input_type in &funclet.header.args {
        match input_type.0.clone() { // adding the phi node
            None => {},
            Some(s) => { nodes.push(ir::Node::Phi { index } ) }
        };
        input_types.push(*context.loc_type_id(input_type.1.clone()))
    }

    output_types.push(*context.loc_type_id(funclet.header.ret.clone()));

    for node in &funclet.commands {
        nodes.push(ir_node(node, context));
    }

    let tail_edge = ir_tail_edge(&funclet.tail_edge, context);

    ir::Funclet {
        kind: funclet.kind.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        tail_edge // actually safe, oddly enough
    }
}

fn ir_funclets(funclets : &ast::FuncletDefs, context : &mut Context) -> StableVec<ir::Funclet> {
    let mut result = StableVec::new();
    for def in funclets {
        match def {
            ast::FuncletDef::Local(f) => {
                context.update_local_funclet(f.header.name.clone());
                result.add(ir_funclet(f, context)); },
            _ => {}
        }
    }
    context.clear_local_funclet();
    result
}

fn ir_value_function(function : &ast::ValueFunction, context : &mut Context) -> ir::ValueFunction {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut default_funclet_id = None;

    for typ in &function.input_types {
        input_types.push(*context.loc_type_id(typ.clone()));
    }
    for typ in &function.output_types {
        output_types.push(*context.loc_type_id(typ.clone()));
    }
    if function.allowed_funclets.len() > 0 {
        let name = function.allowed_funclets.get(0).unwrap();
        let index = match context.funclet_id(name.clone()) {
            context::FuncletLocation::Local(i) => i,
            _ => panic!(format!("Non-local funclet used for value function {}", name))
        };
        default_funclet_id = Some(*index);
    }

    ir::ValueFunction {
        name: function.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        default_funclet_id,
    }
}

fn ir_value_functions(funclets : &ast::FuncletDefs, context : &mut Context) -> StableVec<ir::ValueFunction> {
    let mut result = StableVec::new();
    for def in funclets {
        match def {
            ast::FuncletDef::ValueFunction(f) => {
                result.add(ir_value_function(f, context)); },
            _ => {}
        }
    }
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

fn ir_scheduling_extra(d: &ast::UncheckedDict, context : &mut Context) -> ir::SchedulingFuncletExtra {
    let index = match value_funclet_name(d.get(&as_key("value")).unwrap(), context) {
        context::FuncletLocation::Local(n) => n,
        context::FuncletLocation::GpuFun(n) => n,
        context::FuncletLocation::CpuFun(n) => n,
        context::FuncletLocation::ValueFun(n) => n
    };
    ir::SchedulingFuncletExtra {
        value_funclet_id: index,
        input_slots: value_index_var_dict(d.get(&as_key("input_slots")).unwrap(), value_slot_info, context),
        output_slots: value_index_var_dict(d.get(&as_key("output_slots")).unwrap(), value_slot_info, context),
        input_fences: value_index_var_dict(d.get(&as_key("input_fences")).unwrap(), value_fence_info, context),
        output_fences: value_index_var_dict(d.get(&as_key("output_fences")).unwrap(), value_fence_info, context),
        input_buffers: value_index_var_dict(d.get(&as_key("input_buffers")).unwrap(), value_buffer_info, context),
        output_buffers: value_index_var_dict(d.get(&as_key("output_buffers")).unwrap(), value_buffer_info, context),
        in_timeline_tag: value_dict_timeline_tag(d.get(&as_key("in_timeline_tag")).unwrap(), context),
        out_timeline_tag: value_dict_timeline_tag(d.get(&as_key("out_timeline_tag")).unwrap(), context),
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
                                    ir_scheduling_extra(&extra.data, context)
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
        value_functions: ir_value_functions(&program.funclets, context),
        pipelines: ir_pipelines(&program.pipelines, context),
        value_funclet_extras: ir_value_extras(&program.funclets, &program.extras, context),
        scheduling_funclet_extras: ir_scheduling_extras(&program.funclets, &program.extras, context),
    }
}

pub fn transform(program : ast::Program, context : &mut Context) -> frontend::Definition {
    frontend::Definition {
        version: ir_version(&program.version, context),
        program: ir_program(program, context),
    }
}