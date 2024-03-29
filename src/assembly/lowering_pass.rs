use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::NodeId;
use crate::assembly::context;
use crate::assembly::context::reject_hole;
use crate::assembly::context::Context;
use crate::assembly::explication;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::{BTreeSet, HashMap};

// for reading GPU stuff
use crate::stable_vec::StableVec;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::context::reject_opt;

pub fn undefined<T>(h: Hole<T>) -> T {
    match h {
        Hole::Filled(v) => v,
        Hole::Empty => panic!(""),
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

// Translation

fn ir_version(version: &ast::Version, _: &mut Context) -> (u32, u32, u32) {
    let result = (
        u32::try_from(version.major).unwrap(),
        u32::try_from(version.minor).unwrap(),
        u32::try_from(version.detailed).unwrap(),
    );
    assert_eq!(result, (0, 0, 2));
    result
}

fn ir_tag(tag: &ast::Tag, context: &mut Context) -> ir::Tag {
    ir::Tag {
        quot: context.remote_node_id(&reject_hole(tag.quot.clone())),
        flow: tag.flow.clone(),
    }
}

fn ir_external_gpu_resource(
    d: &ast::ExternalGpuFunctionResourceBinding,
    input_args: &Vec<Option<ast::NodeId>>,
    output_args: &Vec<Option<ast::NodeId>>,
    context: &mut Context,
) -> ffi::GpuKernelResourceBinding {
    fn local_name(
        val: &ast::NodeId,
        input_args: &Vec<Option<ast::NodeId>>,
        output_args: &Vec<Option<ast::NodeId>>,
    ) -> usize {
        let mut index = 0;
        for arg in input_args {
            if Some(val) == arg.as_ref() {
                return index;
            }
            index += 1;
        }
        let mut index = 0;
        for arg in output_args {
            if Some(val) == arg.as_ref() {
                return index;
            }
            index += 1;
        }
        panic!("Unknown GPU variable {:?}", val)
    }
    let group = d.group.clone();
    let binding = d.binding.clone();
    let input = d
        .input
        .as_ref()
        .map(|x| local_name(&x, input_args, output_args));
    let output = d
        .output
        .as_ref()
        .map(|x| local_name(&x, input_args, output_args));
    ffi::GpuKernelResourceBinding {
        group,
        binding,
        input,
        output,
    }
}

fn ir_external(external: &ast::ExternalFunction, context: &mut Context) -> ffi::ExternalFunction {
    let mut input_types = Vec::new();
    let mut input_args = Vec::new();
    let mut output_args = Vec::new();
    let mut output_types = Vec::new();

    for arg in &external.input_args {
        input_types.push(ffi::TypeId(context.ffi_type_id(&arg.ffi_type)));
        input_args.push(arg.name.clone());
    }
    for arg in &external.output_types {
        output_types.push(ffi::TypeId(context.ffi_type_id(&arg.ffi_type)));
        let arg_name = arg.name.clone();
        output_args.push(arg_name);
    }

    match &external.kind {
        ast::ExternalFunctionKind::CPUEffect => {
            ffi::ExternalFunction::CpuEffectfulOperation(ffi::CpuEffectfulOperation {
                name: external.name.clone(),
                input_types: input_types.into_boxed_slice(),
                output_types: output_types.into_boxed_slice(),
            })
        }
        ast::ExternalFunctionKind::CPUPure => {
            ffi::ExternalFunction::CpuPureOperation(ffi::CpuPureOperation {
                name: external.name.clone(),
                input_types: input_types.into_boxed_slice(),
                output_types: output_types.into_boxed_slice(),
            })
        }
        ast::ExternalFunctionKind::GPU(binding_info) => {
            let mut resource_bindings = Vec::new();

            for resource in &binding_info.resource_bindings {
                resource_bindings.push(ir_external_gpu_resource(
                    resource,
                    &input_args,
                    &output_args,
                    context,
                ));
            }

            let input_path = Path::new(&binding_info.shader_module);
            let program_path = Path::new(&context.path);
            let extension = input_path.extension().unwrap();
            let mut input_file = match File::open(program_path.join(input_path)) {
                Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
                Ok(file) => file,
            };
            let mut content = String::new();
            match input_file.read_to_string(&mut content) {
                Err(why) => panic!("Couldn't read file: {}", why),
                Ok(_) => (),
            };
            let shader_module = if extension == "comp" {
                match crate::shadergen::ShaderModule::from_glsl(content.as_str()) {
                    Err(why) => panic!("Couldn't parse as glsl: {}", why),
                    Ok(sm) => sm,
                }
            } else if extension == "wgsl" {
                match crate::shadergen::ShaderModule::from_wgsl(content.as_str()) {
                    Err(why) => panic!("Couldn't parse as glsl: {}", why),
                    Ok(sm) => sm,
                }
            } else {
                panic!(
                    "Unsupported extension for filename {:?}, .wgsl or .comp expected",
                    input_path
                )
            };
            ffi::ExternalFunction::GpuKernel(ffi::GpuKernel {
                name: external.name.clone(),
                input_types: input_types.into_boxed_slice(),
                output_types: output_types.into_boxed_slice(),
                dimensionality: binding_info.dimensionality,
                entry_point: binding_info.entry_point.clone(),
                resource_bindings: resource_bindings.into_boxed_slice(),
                shader_module,
            })
        }
    }
}

fn ir_native_interface(program: &ast::Program, context: &mut Context) -> ffi::NativeInterface {
    let mut types = StableVec::new();
    let mut external_functions = StableVec::new();
    let mut effects = StableVec::new();

    for declaration in &program.declarations {
        match declaration {
            ast::Declaration::TypeDecl(ast::TypeDecl::FFI(t)) => {
                types.add(ffi_to_ffi(t.clone(), context));
            }
            ast::Declaration::ExternalFunction(external) => {
                external_functions.add(ir_external(external, context));
            }
            ast::Declaration::Effect(effect) => {
                effects.add(ir_effect(effect, context));
            }
            _ => {}
        }
    }

    ffi::NativeInterface {
        types,
        external_functions,
        effects,
    }
}

fn ir_type_decl(type_decl: &ast::TypeDecl, context: &mut Context) -> Option<ir::Type> {
    match type_decl {
        ast::TypeDecl::Local(typ) => {
            Some(match &typ.data {
                // only supported custom types atm
                ast::LocalTypeInfo::NativeValue { storage_type } => ir::Type::NativeValue {
                    storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                },
                ast::LocalTypeInfo::Ref {
                    storage_type,
                    storage_place,
                    buffer_flags,
                } => ir::Type::Ref {
                    storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                    storage_place: storage_place.clone(),
                    buffer_flags: buffer_flags.clone(),
                },
                ast::LocalTypeInfo::Fence { queue_place } => ir::Type::Fence {
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Buffer {
                    storage_place,
                    static_layout_opt,
                    flags,
                } => ir::Type::Buffer {
                    storage_place: storage_place.clone(),
                    static_layout_opt: static_layout_opt.clone(),
                    flags: flags.clone(),
                },
                ast::LocalTypeInfo::Encoder { queue_place } => ir::Type::Encoder {
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Event {} => ir::Type::Event {},
                ast::LocalTypeInfo::BufferSpace => ir::Type::BufferSpace,
            })
        }
        ast::TypeDecl::FFI(name) => None,
    }
}

fn ir_node(node: &ast::NamedNode, context: &mut Context) -> ir::Node {
    match &node.node {
        ast::Node::None => ir::Node::None,
        ast::Node::Phi { index } => ir::Node::Phi {
            index: reject_hole(index.as_ref()).clone(),
        },
        ast::Node::ExtractResult { node_id, index } => ir::Node::ExtractResult {
            node_id: context.node_id(reject_hole(node_id.as_ref())),
            index: reject_hole(index.as_ref()).clone(),
        },
        ast::Node::Constant { value, type_id } => {
            let unwrapped_value = reject_hole(value.clone());
            let unwrapped_type = reject_hole(type_id.clone());
            let parsed_value = match &unwrapped_type {
                ast::TypeId::Local(name) => match context.native_type_map.get(name) {
                    None => panic!("{:?} must have a direct FFI storage type", type_id),
                    Some(t) => match t {
                        FFIType::U64 => ir::Constant::U64(unwrapped_value.parse::<u64>().unwrap()),
                        FFIType::I32 => ir::Constant::I32(unwrapped_value.parse::<i32>().unwrap()),
                        FFIType::I64 => ir::Constant::I64(unwrapped_value.parse::<i64>().unwrap()),
                        _ => panic!("Unsupported constant type {:?}", type_id),
                    },
                },
                ast::TypeId::FFI(_) => panic!("Cannot directly type a constant with an ffi type"),
            };
            ir::Node::Constant {
                value: parsed_value,
                type_id: context.loc_type_id(&unwrapped_type),
            }
        }
        ast::Node::CallFunctionClass {
            function_id,
            arguments,
        } => {
            let name = reject_hole(function_id.clone());
            let mapped_arguments: Vec<ir::NodeId> = reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect();
            ir::Node::CallFunctionClass {
                function_id: context
                    .function_classes
                    .get(&ast::FunctionClassId(name.0.clone()))
                    .unwrap_or_else(|| {
                        panic!(
                            "Function class {:?} not found in {:?}",
                            name.0, context.function_classes
                        )
                    }),
                arguments: mapped_arguments.into_boxed_slice(),
            }
        }
        ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => ir::Node::Select {
            condition: context.node_id(reject_hole(condition.as_ref())),
            true_case: context.node_id(reject_hole(true_case.as_ref())),
            false_case: context.node_id(reject_hole(false_case.as_ref())),
        },
        ast::Node::AllocTemporary {
            place,
            storage_type,
            buffer_flags,
        } => ir::Node::AllocTemporary {
            place: reject_hole(place.clone()),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            buffer_flags: reject_hole(buffer_flags.clone()),
        },
        ast::Node::Drop { node } => ir::Node::Drop {
            node: context.node_id(reject_hole(node.as_ref())),
        },
        ast::Node::StaticSubAlloc {
            node,
            place,
            storage_type,
        } => ir::Node::StaticSubAlloc {
            node: context.node_id(reject_hole(node.as_ref())),
            place: reject_hole(place.as_ref()).clone(),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
        },
        ast::Node::StaticSplit {
            spatial_operation,
            node,
            sizes,
            place,
        } => ir::Node::StaticSplit {
            spatial_operation: context.remote_node_id(reject_hole(spatial_operation.as_ref())),
            node: context.node_id(reject_hole(node.as_ref())),
            sizes: reject_hole(sizes.as_ref())
                .iter()
                .map(|n| reject_hole(n.as_ref()).clone())
                .collect(),
            place: reject_hole(place.as_ref()).clone(),
        },
        ast::Node::StaticMerge {
            spatial_operation,
            nodes,
            place,
        } => ir::Node::StaticMerge {
            spatial_operation: context.remote_node_id(reject_hole(spatial_operation.as_ref())),
            nodes: reject_hole(nodes.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            place: reject_hole(place.as_ref()).clone(),
        },
        ast::Node::ReadRef {
            storage_type,
            source,
        } => ir::Node::ReadRef {
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            source: context.node_id(reject_hole(source.as_ref())),
        },
        ast::Node::BorrowRef {
            storage_type,
            source,
        } => ir::Node::BorrowRef {
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            source: context.node_id(reject_hole(source.as_ref())),
        },
        ast::Node::WriteRef {
            storage_type,
            destination,
            source,
        } => ir::Node::WriteRef {
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            destination: context.node_id(reject_hole(destination.as_ref())),
            source: context.node_id(reject_hole(source.as_ref())),
        },
        ast::Node::LocalDoBuiltin {
            operation,
            inputs,
            outputs,
        } => ir::Node::LocalDoBuiltin {
            operation: context.remote_node_id(reject_hole(operation.as_ref())),
            inputs: reject_hole(inputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            outputs: reject_hole(outputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::LocalDoExternal {
            operation,
            external_function_id,
            inputs,
            outputs,
        } => ir::Node::LocalDoExternal {
            operation: context.remote_node_id(reject_hole(operation.as_ref())),
            external_function_id: reject_opt(
                context
                    .funclet_indices
                    .get_funclet(reject_opt(
                        external_function_id.as_ref().opt().map(|x| &x.0),
                    ))
                    .opt()
                    .map(ffi::ExternalFunctionId),
            ),
            inputs: reject_hole(inputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            outputs: reject_hole(outputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::LocalCopy { input, output } => ir::Node::LocalCopy {
            input: context.node_id(reject_hole(input.as_ref())),
            output: context.node_id(reject_hole(output.as_ref())),
        },
        ast::Node::BeginEncoding {
            place,
            event,
            encoded,
            fences,
        } => ir::Node::BeginEncoding {
            place: reject_hole(place.as_ref()).clone(),
            event: context.remote_node_id(reject_hole(event.as_ref())),
            encoded: reject_hole(encoded.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            fences: reject_hole(fences.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::EncodeDoExternal {
            encoder,
            operation,
            external_function_id,
            inputs,
            outputs,
        } => ir::Node::EncodeDoExternal {
            encoder: context.node_id(reject_hole(encoder.as_ref())),
            operation: context.remote_node_id(reject_hole(operation.as_ref())),
            external_function_id: reject_opt(
                context
                    .funclet_indices
                    .get_funclet(reject_opt(
                        external_function_id.as_ref().opt().map(|x| &x.0),
                    ))
                    .opt()
                    .map(ffi::ExternalFunctionId),
            ),
            inputs: reject_hole(inputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            outputs: reject_hole(outputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::EncodeCopy {
            encoder,
            input,
            output,
        } => ir::Node::EncodeCopy {
            encoder: context.node_id(reject_hole(encoder.as_ref())),
            input: context.node_id(reject_hole(input.as_ref())),
            output: context.node_id(reject_hole(output.as_ref())),
        },
        ast::Node::Submit { encoder, event } => ir::Node::Submit {
            encoder: context.node_id(reject_hole(encoder.as_ref())),
            event: context.remote_node_id(reject_hole(event.as_ref())),
        },
        ast::Node::SyncFence { fence, event } => ir::Node::SyncFence {
            fence: context.node_id(reject_hole(fence.as_ref())),
            event: context.remote_node_id(reject_hole(event.as_ref())),
        },
        ast::Node::InlineJoin {
            funclet,
            captures,
            continuation,
        } => ir::Node::InlineJoin {
            funclet: context
                .funclet_indices
                .require_funclet(&reject_hole(funclet.as_ref()).0),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        },
        ast::Node::SerializedJoin {
            funclet,
            captures,
            continuation,
        } => ir::Node::SerializedJoin {
            funclet: context
                .funclet_indices
                .require_funclet(&reject_hole(funclet.as_ref()).0),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        },
        ast::Node::DefaultJoin => ir::Node::DefaultJoin {},
        ast::Node::PromiseCaptures {
            count,
            continuation,
        } => ir::Node::PromiseCaptures {
            count: reject_hole(count.as_ref()).clone(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        },
        ast::Node::FulfillCaptures {
            continuation,
            haves,
            needs,
        } => ir::Node::FulfillCaptures {
            continuation: context.node_id(reject_hole(continuation.as_ref())),
            haves: reject_hole(haves.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            needs: reject_hole(needs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::EncodingEvent {
            local_past,
            remote_local_pasts,
        } => ir::Node::EncodingEvent {
            local_past: context.node_id(reject_hole(local_past.as_ref())),
            remote_local_pasts: reject_hole(remote_local_pasts.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::Node::SubmissionEvent { local_past } => ir::Node::SubmissionEvent {
            local_past: context.node_id(reject_hole(local_past.as_ref())),
        },
        ast::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => ir::Node::SynchronizationEvent {
            local_past: context.node_id(reject_hole(local_past.as_ref())),
            remote_local_past: context.node_id(reject_hole(remote_local_past.as_ref())),
        },
        ast::Node::SeparatedBufferSpaces { count, space } => ir::Node::SeparatedBufferSpaces {
            count: reject_hole(count.as_ref()).clone(),
            space: context.node_id(reject_hole(space.as_ref())),
        },
    }
}

fn ir_tail_edge(tail: &ast::TailEdge, context: &mut Context) -> ir::TailEdge {
    match tail {
        ast::TailEdge::DebugHole { inputs } => ir::TailEdge::DebugHole {
            inputs: inputs.iter().map(|n| context.node_id(n)).collect(),
        },
        ast::TailEdge::Return { return_values } => ir::TailEdge::Return {
            return_values: reject_hole(return_values.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::TailEdge::Jump { join, arguments } => ir::TailEdge::Jump {
            join: context.node_id(reject_hole(join.as_ref())),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        },
        ast::TailEdge::ScheduleCall {
            operations,
            callee_funclet_id,
            callee_arguments,
            continuation_join,
        } => {
            let operation_set = context.operational_lookup(reject_hole(operations.as_ref()));
            ir::TailEdge::ScheduleCall {
                value_operation: reject_hole(operation_set.value),
                timeline_operation: reject_hole(operation_set.timeline),
                spatial_operation: reject_hole(operation_set.spatial),
                callee_funclet_id: context
                    .funclet_indices
                    .get_funclet(&reject_hole(callee_funclet_id.as_ref()).0)
                    .opt()
                    .unwrap()
                    .clone(),
                callee_arguments: reject_hole(callee_arguments.as_ref())
                    .iter()
                    .map(|n| context.node_id(reject_hole(n.as_ref())))
                    .collect(),
                continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
            }
        }
        ast::TailEdge::ScheduleSelect {
            operations,
            condition,
            callee_funclet_ids,
            callee_arguments,
            continuation_join,
        } => {
            let operation_set = context.operational_lookup(reject_hole(operations.as_ref()));
            ir::TailEdge::ScheduleSelect {
                value_operation: reject_hole(operation_set.value),
                timeline_operation: reject_hole(operation_set.timeline),
                spatial_operation: reject_hole(operation_set.spatial),
                condition: context.node_id(reject_hole(condition.as_ref())),
                callee_funclet_ids: reject_hole(callee_funclet_ids.as_ref())
                    .iter()
                    .map(|n| {
                        context
                            .funclet_indices
                            .get_funclet(&reject_hole(n.as_ref()).0)
                            .opt()
                            .unwrap()
                            .clone()
                    })
                    .collect(),
                callee_arguments: reject_hole(callee_arguments.as_ref())
                    .iter()
                    .map(|n| context.node_id(reject_hole(n.as_ref())))
                    .collect(),
                continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
            }
        }
        ast::TailEdge::ScheduleCallYield {
            operations,
            external_function_id,
            yielded_nodes,
            continuation_join,
        } => {
            let operation_set = context.operational_lookup(reject_hole(operations.as_ref()));
            ir::TailEdge::ScheduleCallYield {
                value_operation: reject_hole(operation_set.value),
                timeline_operation: reject_hole(operation_set.timeline),
                spatial_operation: reject_hole(operation_set.spatial),
                external_function_id: ffi::ExternalFunctionId(
                    context
                        .funclet_indices
                        .get_funclet(&reject_hole(external_function_id.as_ref()).0)
                        .opt()
                        .unwrap()
                        .clone(),
                ),
                yielded_nodes: reject_hole(yielded_nodes.as_ref())
                    .iter()
                    .map(|n| context.node_id(reject_hole(n.as_ref())))
                    .collect(),
                continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
            }
        }
    }
}

// updates the location in the context value funclet
fn ir_schedule_binding(
    funclet_header: &ast::FuncletHeader,
    implicit_tags: &(ast::Tag, ast::Tag),
    meta_map: &ast::MetaMapping,
    context: &mut Context,
) -> ir::FuncletSpecBinding {
    context.set_meta_map(meta_map.clone());

    let default_tag = ir::Tag {
        quot: ir::Quotient::None,
        flow: ir::Flow::Usable,
    };

    struct TagBindings {
        value_tags: Vec<ir::Tag>,
        spatial_tags: Vec<ir::Tag>,
        timeline_tags: Vec<ir::Tag>,
    }

    let mut input_tags = TagBindings {
        value_tags: Vec::new(),
        spatial_tags: Vec::new(),
        timeline_tags: Vec::new(),
    };
    let mut output_tags = TagBindings {
        value_tags: Vec::new(),
        spatial_tags: Vec::new(),
        timeline_tags: Vec::new(),
    };

    for arg in &funclet_header.args {
        let tags = context.tag_lookup(&arg.tags.iter().map(|t| Hole::Filled(t.clone())).collect());
        input_tags
            .value_tags
            .push(tags.value.opt().unwrap_or(default_tag).clone());
        input_tags
            .spatial_tags
            .push(tags.spatial.opt().unwrap_or(default_tag).clone());
        input_tags
            .timeline_tags
            .push(tags.timeline.opt().unwrap_or(default_tag).clone());
    }

    for ret in &funclet_header.ret {
        let tags = context.tag_lookup(&ret.tags.iter().map(|t| Hole::Filled(t.clone())).collect());
        output_tags
            .value_tags
            .push(tags.value.opt().unwrap_or(default_tag).clone());
        output_tags
            .spatial_tags
            .push(tags.spatial.opt().unwrap_or(default_tag).clone());
        output_tags
            .timeline_tags
            .push(tags.timeline.opt().unwrap_or(default_tag).clone());
    }

    let implicit_in_tag = ir_tag(&implicit_tags.0, context);
    let implicit_out_tag = ir_tag(&implicit_tags.1, context);

    ir::FuncletSpecBinding::ScheduleExplicit {
        value: ir::FuncletSpec {
            funclet_id_opt: context
                .funclet_indices
                .get_funclet(&meta_map.value.1 .0)
                .into(),
            input_tags: input_tags.value_tags.into_boxed_slice(),
            output_tags: output_tags.value_tags.into_boxed_slice(),
            implicit_in_tag: Default::default(),
            implicit_out_tag: Default::default(),
        },
        timeline: ir::FuncletSpec {
            // assume implicit is timeline for now?
            funclet_id_opt: context
                .funclet_indices
                .get_funclet(&meta_map.timeline.1 .0)
                .into(),
            input_tags: input_tags.timeline_tags.into_boxed_slice(),
            output_tags: output_tags.timeline_tags.into_boxed_slice(),
            implicit_in_tag,
            implicit_out_tag,
        },
        spatial: ir::FuncletSpec {
            funclet_id_opt: context
                .funclet_indices
                .get_funclet(&meta_map.spatial.1 .0)
                .into(),
            input_tags: input_tags.spatial_tags.into_boxed_slice(),
            output_tags: output_tags.spatial_tags.into_boxed_slice(),
            implicit_in_tag: Default::default(),
            implicit_out_tag: Default::default(),
        },
    }
}

fn ir_spec_binding(
    funclet_header: &ast::FuncletHeader,
    context: &mut Context,
) -> ir::FuncletSpecBinding {
    match &funclet_header.binding {
        ast::FuncletBinding::None => ir::FuncletSpecBinding::None,
        ast::FuncletBinding::SpecBinding(ast::FunctionClassBinding {
            default,
            function_class,
        }) => {
            let value_function_id_opt =
                Some(context.function_classes.get(&function_class).unwrap());
            ir::FuncletSpecBinding::Value {
                value_function_id_opt,
            }
        }
        ast::FuncletBinding::ScheduleBinding(ast::ScheduleBinding {
            implicit_tags,
            meta_map,
        }) => ir_schedule_binding(funclet_header, implicit_tags, meta_map, context),
    }
}

fn ir_funclet(funclet: &ast::Funclet, context: &mut Context) -> ir::Funclet {
    context.location.funclet_name = funclet.header.name.clone();
    // note that this is stateful, updates the value_funclet in context potentially
    let spec_binding = ir_spec_binding(&funclet.header, context);
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut nodes = Vec::new();
    let mut tail_edge = None;

    for arg in &funclet.header.args {
        input_types.push(context.loc_type_id(&arg.typ));
    }

    for arg in &funclet.header.ret {
        output_types.push(context.loc_type_id(&arg.typ));
    }

    for command in &funclet.commands {
        match reject_hole(command.as_ref()) {
            ast::Command::Node(node) => {
                context.location.node_name = node.name.clone();
                nodes.push(ir_node(&node, context));
            }
            ast::Command::TailEdge(tail) => {
                if tail_edge.is_some() {
                    panic!("More than one tail edge in {:?}", funclet.header.name);
                }
                tail_edge = Some(ir_tail_edge(tail, context));
            }
        }
    }

    // help avoid reuse issues
    context.reset_meta_map();

    ir::Funclet {
        kind: funclet.kind.clone(),
        spec_binding,
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        tail_edge: tail_edge.unwrap(), // actually safe, oddly enough
    }
}

fn ir_effect(declaration: &ast::EffectDeclaration, context: &mut Context) -> ffi::Effect {
    match &declaration.effect {
        ast::Effect::Unrestricted => ffi::Effect::Unrestricted,
        ast::Effect::FullyConnected {
            effectful_function_ids,
        } => ffi::Effect::FullyConnected {
            effectful_function_ids: effectful_function_ids
                .iter()
                .map(|fid| context.external_lookup(fid))
                .collect(),
        },
    }
}

fn ir_function_class(
    declarations: &Vec<ast::Declaration>,
    function: &ast::FunctionClass,
    context: &mut Context,
) -> ir::FunctionClass {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut default_funclet_id = None;
    let mut external_function_ids = BTreeSet::new();

    for typ in &function.input_types {
        input_types.push(context.loc_type_id(&typ));
    }
    for typ in &function.output_types {
        output_types.push(context.loc_type_id(&typ));
    }

    // not efficient, but whatever
    for declaration in declarations {
        match declaration {
            ast::Declaration::Funclet(f) => match &f.header.binding {
                ast::FuncletBinding::SpecBinding(binding) => {
                    if binding.function_class == function.name {
                        let current_id = context
                            .funclet_indices
                            .get_funclet(&f.header.name.0)
                            .opt()
                            .unwrap();
                        if binding.default {
                            default_funclet_id = match default_funclet_id {
                                None => Some(current_id),
                                Some(_) => {
                                    panic!("Duplicate default ids for {:?}", function.name.clone())
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            ast::Declaration::ExternalFunction(f) => {
                if f.value_function_binding.function_class == function.name {
                    if f.value_function_binding.default {
                        panic!(
                            "{:?} uses default, which is unsupported for external functions",
                            f.name
                        )
                    }
                    external_function_ids.insert(ffi::ExternalFunctionId(
                        context.funclet_indices.get_funclet(&f.name).opt().unwrap(),
                    ));
                }
            }
            _ => {}
        }
    }

    ir::FunctionClass {
        name_opt: Some(function.name.0.clone()),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        default_funclet_id,
        external_function_ids,
    }
}

fn ir_pipeline(pipeline: &ast::Pipeline, context: &mut Context) -> ir::Pipeline {
    ir::Pipeline {
        name: pipeline.name.clone(),
        entry_funclet: context
            .funclet_indices
            .get_funclet(&pipeline.funclet.0)
            .opt()
            .unwrap()
            .clone(),
        effect_id_opt: pipeline.effect.as_ref().map(|e| context.effect_lookup(e)),
    }
}

fn ir_program(program: &ast::Program, context: &mut Context) -> ir::Program {
    let native_interface = ir_native_interface(&program, context);
    let mut types = StableVec::new();
    let mut funclets = StableVec::new();
    let mut function_classes = StableVec::new();
    let mut pipelines = Vec::new();

    for declaration in &program.declarations {
        match declaration {
            ast::Declaration::TypeDecl(t) => match ir_type_decl(t, context) {
                Some(typ) => {
                    types.add(typ);
                }
                None => {}
            },
            ast::Declaration::ExternalFunction(e) => {
                // nothing to do cause in the native_interface
                // some duplicate looping, but whatever
            }
            ast::Declaration::FunctionClass(c) => {
                function_classes.add(ir_function_class(&program.declarations, c, context));
            }
            ast::Declaration::Funclet(f) => {
                funclets.add(ir_funclet(f, context));
            }
            ast::Declaration::Pipeline(p) => {
                pipelines.push(ir_pipeline(p, context));
            }
            ast::Declaration::Effect(effect) => {}
        }
    }

    ir::Program {
        native_interface,
        types,
        funclets,
        function_classes,
        pipelines,
    }
}

pub fn lower(mut program: ast::Program) -> frontend::Definition {
    // should probably handle errors with a result, future problem though
    explication::explicate(&mut program);
    let mut context = Context::new(&program);
    // dbg!(&context);
    frontend::Definition {
        version: ir_version(&program.version, &mut context),
        program: ir_program(&program, &mut context),
    }
}
