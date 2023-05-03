use crate::assembly;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalCpuFunction, ExternalGpuFunction, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::assembly::context;
use crate::assembly::context::Context;
use crate::assembly::explication_explicator;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{frontend, ir};
use std::any::Any;
use std::collections::HashMap;

pub fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => panic!("Unimplemented Hole"),
    }
}

pub fn ffi_to_ffi(value: FFIType, context: &mut Context) -> ffi::Type {
    fn box_map(b: Box<[FFIType]>, context: &mut Context) -> Box<[ffi::TypeId]> {
        b.iter()
            .map(|x| ffi::TypeId(context.ffi_type_id(x)))
            .collect()
    }
    fn type_id(element_type: Box<FFIType>, context: &mut Context) -> ffi::TypeId {
        ffi::TypeId(context.ffi_type_id(element_type.as_ref()))
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
        FFIType::Array {
            element_type,
            length,
        } => ffi::Type::Array {
            element_type: type_id(element_type, context),
            length,
        },
        FFIType::ErasedLengthArray(element_type) => ffi::Type::ErasedLengthArray {
            element_type: type_id(element_type, context),
        },
        FFIType::Struct {
            fields,
            byte_alignment,
            byte_size,
        } => todo!(),
        FFIType::Tuple(element_types) => ffi::Type::Tuple {
            fields: box_map(element_types.into_boxed_slice(), context),
        },
        FFIType::ConstRef(element_type) => ffi::Type::ConstRef {
            element_type: type_id(element_type, context),
        },
        FFIType::MutRef(element_type) => ffi::Type::MutRef {
            element_type: type_id(element_type, context),
        },
        FFIType::ConstSlice(element_type) => ffi::Type::ConstSlice {
            element_type: type_id(element_type, context),
        },
        FFIType::MutSlice(element_type) => ffi::Type::MutSlice {
            element_type: type_id(element_type, context),
        },
        FFIType::GpuBufferRef(element_type) => ffi::Type::GpuBufferRef {
            element_type: type_id(element_type, context),
        },
        FFIType::GpuBufferSlice(element_type) => ffi::Type::GpuBufferSlice {
            element_type: type_id(element_type, context),
        },
        FFIType::GpuBufferAllocator => ffi::Type::GpuBufferAllocator,
        FFIType::CpuBufferAllocator => ffi::Type::CpuBufferAllocator,
        FFIType::CpuBufferRef(element_type) => ffi::Type::CpuBufferRef {
            element_type: type_id(element_type, context),
        },
    }
}

pub fn as_key(k: &str) -> assembly::ast::Value {
    assembly::ast::Value::ID(k.to_string())
}

pub fn as_value(value: assembly::ast::DictValue) -> assembly::ast::Value {
    match value {
        assembly::ast::DictValue::Raw(v) => v,
        _ => panic!("Expected raw value got {:?}", value),
    }
}

pub fn as_list(value: assembly::ast::DictValue) -> Vec<assembly::ast::DictValue> {
    match value {
        assembly::ast::DictValue::List(v) => v,
        _ => panic!("Expected list got {:?}", value),
    }
}

pub fn as_dict(value: assembly::ast::DictValue) -> assembly::ast::UncheckedDict {
    match value {
        assembly::ast::DictValue::Dict(d) => d,
        _ => panic!("Expected dict got {:?}", value),
    }
}

pub fn remote_conversion(
    remote: &assembly::ast::RemoteNodeName,
    context: &mut Context,
) -> ir::RemoteNodeId {
    context.remote_id(&remote.funclet_name.clone(), &remote.node_name.clone())
}

pub fn value_string(d: &assembly::ast::DictValue, _: &mut Context) -> String {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::ID(s) => s.clone(),
        _ => panic!("Expected id got {:?}", v),
    }
}

pub fn value_num(d: &assembly::ast::DictValue, _: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Num(n) => n.clone(),
        _ => panic!("Expected num got {:?}", v),
    }
}

pub fn value_function_loc(d: &assembly::ast::DictValue, context: &mut Context) -> ir::RemoteNodeId {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::FunctionLoc(remote) => ir::RemoteNodeId {
            funclet_id: context.funclet_indices.get(&remote.funclet_name).unwrap(),
            node_id: context.funclet_indices.get(&remote.node_name).unwrap(),
        },
        _ => panic!("Expected function location got {:?}", v),
    }
}

pub fn value_var_name(d: &assembly::ast::DictValue, ret: bool, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::VarName(s) => {
            if ret {
                context.return_id(&s)
            } else {
                context.node_id(&s)
            }
        }
        _ => panic!("Expected variable name got {:?}", v),
    }
}

pub fn value_funclet_name(d: &assembly::ast::DictValue, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::FnName(s) => context.funclet_indices.get(&s).unwrap(),
        _ => panic!("Expected funclet name got {:?}", v),
    }
}

pub fn value_funclet_raw_id(d: &assembly::ast::DictValue, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::FnName(s) => context.funclet_indices.get(&s).unwrap(),
        _ => panic!("Expected funclet name got {:?}", v),
    }
}

pub fn value_type(d: &assembly::ast::DictValue, context: &mut Context) -> context::Location {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Type(t) => match t {
            assembly::ast::Type::FFI(typ) => context::Location::FFI(context.ffi_type_id(&typ)),
            assembly::ast::Type::Local(name) => {
                context::Location::Local(context.local_type_id(&name))
            }
        },
        _ => panic!("Expected type got {:?}", v),
    }
}

pub fn value_place(d: &assembly::ast::DictValue, _: &mut Context) -> ir::Place {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Place(p) => p.clone(),
        _ => panic!("Expected place got {:?}", v),
    }
}

pub fn value_stage(d: &assembly::ast::DictValue, _: &mut Context) -> ir::ResourceQueueStage {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Stage(s) => s.clone(),
        _ => panic!("Expected stage got {:?}", v),
    }
}

// This all feels very dumb
pub fn value_core_tag(v: assembly::ast::TagCore, context: &mut Context) -> ir::ValueTag {
    match v {
        assembly::ast::TagCore::None => ir::ValueTag::None,
        assembly::ast::TagCore::Operation(r) => ir::ValueTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly::ast::TagCore::Input(r) => ir::ValueTag::Input {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
        assembly::ast::TagCore::Output(r) => ir::ValueTag::Output {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
    }
}

pub fn timeline_core_tag(v: assembly::ast::TagCore, context: &mut Context) -> ir::TimelineTag {
    match v {
        assembly::ast::TagCore::None => ir::TimelineTag::None,
        assembly::ast::TagCore::Operation(r) => ir::TimelineTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly::ast::TagCore::Input(r) => ir::TimelineTag::Input {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
        assembly::ast::TagCore::Output(r) => ir::TimelineTag::Output {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
    }
}

pub fn spatial_core_tag(v: assembly::ast::TagCore, context: &mut Context) -> ir::SpatialTag {
    match v {
        assembly::ast::TagCore::None => ir::SpatialTag::None,
        assembly::ast::TagCore::Operation(r) => ir::SpatialTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly::ast::TagCore::Input(r) => ir::SpatialTag::Input {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
        assembly::ast::TagCore::Output(r) => ir::SpatialTag::Output {
            funclet_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
    }
}

pub fn value_value_tag(t: &assembly::ast::ValueTag, context: &mut Context) -> ir::ValueTag {
    match t {
        assembly::ast::ValueTag::Core(c) => value_core_tag(c.clone(), context),
        assembly::ast::ValueTag::FunctionInput(r) => ir::ValueTag::FunctionInput {
            function_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
        assembly::ast::ValueTag::FunctionOutput(r) => ir::ValueTag::FunctionOutput {
            function_id: context.funclet_indices.get(&r.funclet_name).unwrap(),
            index: context.remote_node_id(&r.funclet_name, &r.node_name),
        },
        assembly::ast::ValueTag::Halt(n) => ir::ValueTag::Halt {
            index: context.node_id(&n),
        },
    }
}

pub fn value_dict_value_tag(d: &assembly::ast::DictValue, context: &mut Context) -> ir::ValueTag {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Tag(t) => match t {
            assembly::ast::Tag::ValueTag(v) => value_value_tag(&v, context),
            _ => panic!("Expected value tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_timeline_tag(
    t: &assembly::ast::TimelineTag,
    context: &mut Context,
) -> ir::TimelineTag {
    match t {
        assembly::ast::TimelineTag::Core(c) => timeline_core_tag(c.clone(), context),
    }
}

pub fn value_dict_timeline_tag(
    d: &assembly::ast::DictValue,
    context: &mut Context,
) -> ir::TimelineTag {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Tag(t) => match t {
            assembly::ast::Tag::TimelineTag(t) => value_timeline_tag(&t, context),
            _ => panic!("Expected timeline tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_spatial_tag(t: &assembly::ast::SpatialTag, context: &mut Context) -> ir::SpatialTag {
    match t {
        assembly::ast::SpatialTag::Core(c) => spatial_core_tag(c.clone(), context),
    }
}

pub fn value_dict_spatial_tag(
    d: &assembly::ast::DictValue,
    context: &mut Context,
) -> ir::SpatialTag {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::Tag(t) => match t {
            assembly::ast::Tag::SpatialTag(t) => value_spatial_tag(&t, context),
            _ => panic!("Expected spatial tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_slot_info(d: &assembly::ast::DictValue, context: &mut Context) -> ir::SlotInfo {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::SlotInfo(s) => ir::SlotInfo {
            value_tag: value_value_tag(&s.value_tag, context),
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_fence_info(d: &assembly::ast::DictValue, context: &mut Context) -> ir::FenceInfo {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::FenceInfo(s) => ir::FenceInfo {
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_buffer_info(d: &assembly::ast::DictValue, context: &mut Context) -> ir::BufferInfo {
    let v = as_value(d.clone());
    match v {
        assembly::ast::Value::BufferInfo(s) => ir::BufferInfo {
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_list<T>(
    v: &assembly::ast::DictValue,
    f: fn(&assembly::ast::DictValue, &mut Context) -> T,
    context: &mut Context,
) -> HashMap<usize, T> {
    let lst = as_list(v.clone());
    let mut result = HashMap::new();
    let index = 0;
    for value in lst.iter() {
        result.insert(index, f(value, context));
    }
    result
}

pub fn value_index_var_dict<T>(
    v: &assembly::ast::DictValue,
    f: fn(&assembly::ast::DictValue, &mut Context) -> T,
    ret: bool, // read remote or not
    context: &mut Context,
) -> HashMap<usize, T> {
    let d = as_dict(v.clone());
    let mut result = HashMap::new();
    for pair in d.iter() {
        let index = value_var_name(&assembly::ast::DictValue::Raw(pair.0.clone()), ret, context);
        result.insert(index, f(&pair.1.clone(), context));
    }
    result
}

pub fn get_first<'a, T>(v: &'a Vec<T>, test: fn(&T) -> bool) -> Option<&'a T>
where
    T: Sized,
{
    for item in v {
        if test(item) {
            return Some(&item);
        }
    }
    None
}
