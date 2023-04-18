use crate::assembly::explication_context::Context;
use crate::assembly::explication_explicator;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_context::FuncletLocation;
use crate::ir::ffi;
use crate::{assembly_ast, assembly_context, frontend, ir};
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
            .map(|x| ffi::TypeId(context.inner.ffi_type_id(x)))
            .collect()
    }
    fn type_id(element_type: Box<FFIType>, context: &mut Context) -> ffi::TypeId {
        ffi::TypeId(context.inner.ffi_type_id(element_type.as_ref()))
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

pub fn as_key(k: &str) -> assembly_ast::Value {
    assembly_ast::Value::ID(k.to_string())
}

pub fn as_value(value: assembly_ast::DictValue) -> assembly_ast::Value {
    match value {
        assembly_ast::DictValue::Raw(v) => v,
        _ => panic!("Expected raw value got {:?}", value),
    }
}

pub fn as_list(value: assembly_ast::DictValue) -> Vec<assembly_ast::DictValue> {
    match value {
        assembly_ast::DictValue::List(v) => v,
        _ => panic!("Expected list got {:?}", value),
    }
}

pub fn as_dict(value: assembly_ast::DictValue) -> assembly_ast::UncheckedDict {
    match value {
        assembly_ast::DictValue::Dict(d) => d,
        _ => panic!("Expected dict got {:?}", value),
    }
}

pub fn remote_conversion(
    remote: &assembly_ast::RemoteNodeId,
    context: &mut Context,
) -> ir::RemoteNodeId {
    context
        .inner
        .remote_id(remote.funclet_id.clone(), remote.node_id.clone())
}

pub fn value_string(d: &assembly_ast::DictValue, _: &mut Context) -> String {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::ID(s) => s.clone(),
        _ => panic!("Expected id got {:?}", v),
    }
}

pub fn value_num(d: &assembly_ast::DictValue, _: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Num(n) => n.clone(),
        _ => panic!("Expected num got {:?}", v),
    }
}

pub fn value_function_loc(d: &assembly_ast::DictValue, context: &mut Context) -> ir::RemoteNodeId {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::FunctionLoc(remote) => ir::RemoteNodeId {
            funclet_id: context
                .inner
                .local_funclet_id(remote.funclet_id.clone())
                .clone(),
            node_id: context
                .inner
                .local_funclet_id(remote.node_id.clone())
                .clone(),
        },
        _ => panic!("Expected function location got {:?}", v),
    }
}

pub fn value_var_name(d: &assembly_ast::DictValue, ret: bool, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::VarName(s) => {
            if ret {
                context.inner.return_id(s.clone())
            } else {
                context.inner.node_id(s.clone())
            }
        }
        _ => panic!("Expected variable name got {:?}", v),
    }
}

pub fn value_funclet_name(d: &assembly_ast::DictValue, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::FnName(s) => context.inner.funclet_id(&s).clone(),
        _ => panic!("Expected funclet name got {:?}", v),
    }
}

pub fn value_funclet_raw_id(d: &assembly_ast::DictValue, context: &mut Context) -> usize {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::FnName(s) => context.inner.funclet_id(&s).clone(),
        _ => panic!("Expected funclet name got {:?}", v),
    }
}

pub fn value_type(
    d: &assembly_ast::DictValue,
    context: &mut Context,
) -> assembly_context::Location {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Type(t) => match t {
            assembly_ast::Type::FFI(typ) => {
                assembly_context::Location::FFI(context.inner.ffi_type_id(&typ))
            }
            assembly_ast::Type::Local(name) => {
                assembly_context::Location::Local(context.inner.local_type_id(&name))
            }
        },
        _ => panic!("Expected type got {:?}", v),
    }
}

pub fn value_place(d: &assembly_ast::DictValue, _: &mut Context) -> ir::Place {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Place(p) => p.clone(),
        _ => panic!("Expected place got {:?}", v),
    }
}

pub fn value_stage(d: &assembly_ast::DictValue, _: &mut Context) -> ir::ResourceQueueStage {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Stage(s) => s.clone(),
        _ => panic!("Expected stage got {:?}", v),
    }
}

// This all feels very dumb
pub fn value_core_tag(v: assembly_ast::TagCore, context: &mut Context) -> ir::ValueTag {
    match v {
        assembly_ast::TagCore::None => ir::ValueTag::None,
        assembly_ast::TagCore::Operation(r) => ir::ValueTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly_ast::TagCore::Input(r) => ir::ValueTag::Input {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        assembly_ast::TagCore::Output(r) => ir::ValueTag::Output {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

pub fn timeline_core_tag(v: assembly_ast::TagCore, context: &mut Context) -> ir::TimelineTag {
    match v {
        assembly_ast::TagCore::None => ir::TimelineTag::None,
        assembly_ast::TagCore::Operation(r) => ir::TimelineTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly_ast::TagCore::Input(r) => ir::TimelineTag::Input {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        assembly_ast::TagCore::Output(r) => ir::TimelineTag::Output {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

pub fn spatial_core_tag(v: assembly_ast::TagCore, context: &mut Context) -> ir::SpatialTag {
    match v {
        assembly_ast::TagCore::None => ir::SpatialTag::None,
        assembly_ast::TagCore::Operation(r) => ir::SpatialTag::Operation {
            remote_node_id: remote_conversion(&r, context),
        },
        assembly_ast::TagCore::Input(r) => ir::SpatialTag::Input {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        assembly_ast::TagCore::Output(r) => ir::SpatialTag::Output {
            funclet_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
    }
}

pub fn value_value_tag(t: &assembly_ast::ValueTag, context: &mut Context) -> ir::ValueTag {
    match t {
        assembly_ast::ValueTag::Core(c) => value_core_tag(c.clone(), context),
        assembly_ast::ValueTag::FunctionInput(r) => ir::ValueTag::FunctionInput {
            function_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        assembly_ast::ValueTag::FunctionOutput(r) => ir::ValueTag::FunctionOutput {
            function_id: context.inner.local_funclet_id(r.funclet_id.clone()).clone(),
            index: context
                .inner
                .remote_node_id(r.funclet_id.clone(), r.node_id.clone()),
        },
        assembly_ast::ValueTag::Halt(n) => ir::ValueTag::Halt {
            index: context.inner.node_id(n.clone()),
        },
    }
}

pub fn value_dict_value_tag(d: &assembly_ast::DictValue, context: &mut Context) -> ir::ValueTag {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Tag(t) => match t {
            assembly_ast::Tag::ValueTag(v) => value_value_tag(&v, context),
            _ => panic!("Expected value tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_timeline_tag(t: &assembly_ast::TimelineTag, context: &mut Context) -> ir::TimelineTag {
    match t {
        assembly_ast::TimelineTag::Core(c) => timeline_core_tag(c.clone(), context),
    }
}

pub fn value_dict_timeline_tag(
    d: &assembly_ast::DictValue,
    context: &mut Context,
) -> ir::TimelineTag {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Tag(t) => match t {
            assembly_ast::Tag::TimelineTag(t) => value_timeline_tag(&t, context),
            _ => panic!("Expected timeline tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_spatial_tag(t: &assembly_ast::SpatialTag, context: &mut Context) -> ir::SpatialTag {
    match t {
        assembly_ast::SpatialTag::Core(c) => spatial_core_tag(c.clone(), context),
    }
}

pub fn value_dict_spatial_tag(
    d: &assembly_ast::DictValue,
    context: &mut Context,
) -> ir::SpatialTag {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::Tag(t) => match t {
            assembly_ast::Tag::SpatialTag(t) => value_spatial_tag(&t, context),
            _ => panic!("Expected spatial tag got {:?}", d),
        },
        _ => panic!("Expected tag got {:?}", d),
    }
}

pub fn value_slot_info(d: &assembly_ast::DictValue, context: &mut Context) -> ir::SlotInfo {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::SlotInfo(s) => ir::SlotInfo {
            value_tag: value_value_tag(&s.value_tag, context),
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_fence_info(d: &assembly_ast::DictValue, context: &mut Context) -> ir::FenceInfo {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::FenceInfo(s) => ir::FenceInfo {
            timeline_tag: value_timeline_tag(&s.timeline_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_buffer_info(d: &assembly_ast::DictValue, context: &mut Context) -> ir::BufferInfo {
    let v = as_value(d.clone());
    match v {
        assembly_ast::Value::BufferInfo(s) => ir::BufferInfo {
            spatial_tag: value_spatial_tag(&s.spatial_tag, context),
        },
        _ => panic!("Expected tag got {:?}", v),
    }
}

pub fn value_list<T>(
    v: &assembly_ast::DictValue,
    f: fn(&assembly_ast::DictValue, &mut Context) -> T,
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
    v: &assembly_ast::DictValue,
    f: fn(&assembly_ast::DictValue, &mut Context) -> T,
    ret: bool, // read remote or not
    context: &mut Context,
) -> HashMap<usize, T> {
    let d = as_dict(v.clone());
    let mut result = HashMap::new();
    for pair in d.iter() {
        let index = value_var_name(&assembly_ast::DictValue::Raw(pair.0.clone()), ret, context);
        result.insert(index, f(&pair.1.clone(), context));
    }
    result
}
