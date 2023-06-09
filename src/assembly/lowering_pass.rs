use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, StorageTypeId, TypeId,
};
use crate::assembly::context::Context;
use crate::assembly::context::FuncletLocation;
use crate::assembly::context::LocationNames;
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

// Utility stuff

pub fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => unreachable!("Unimplemented Hole"),
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

pub fn remote_conversion(remote: &ast::RemoteNodeId, context: &Context) -> ir::RemoteNodeId {
    remote_location_conversion(
        &LocationNames {
            funclet_name: remote.funclet_name.clone().unwrap(),
            node_name: remote.node_name.clone().unwrap(),
        },
        context,
    )
}

pub fn remote_location_conversion(remote: &LocationNames, context: &Context) -> ir::RemoteNodeId {
    ir::RemoteNodeId {
        funclet_id: context
            .funclet_indices
            .get_funclet(&remote.funclet_name.0)
            .unwrap()
            .clone(),
        node_id: context
            .remote_node_id(&remote.funclet_name, &remote.node_name)
            .clone(),
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

fn ir_external_gpu_resource(
    d: &ast::ExternalGpuFunctionResourceBinding,
    input_args: &Vec<Option<NodeId>>,
    output_args: &Vec<Option<NodeId>>,
    context: &mut Context,
) -> ffi::GpuKernelResourceBinding {
    fn local_name(
        val: &NodeId,
        input_args: &Vec<Option<NodeId>>,
        output_args: &Vec<Option<NodeId>>,
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

            // Very silly
            let input_path = Path::new(&binding_info.shader_module);
            assert!(binding_info.shader_module.ends_with(".wgsl"));
            let mut input_file = match File::open(&input_path) {
                Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
                Ok(file) => file,
            };
            let mut content = String::new();
            match input_file.read_to_string(&mut content) {
                Err(why) => panic!("Couldn't read file: {}", why),
                Ok(_) => (),
            };
            ffi::ExternalFunction::GpuKernel(ffi::GpuKernel {
                name: external.name.clone(),
                input_types: input_types.into_boxed_slice(),
                output_types: output_types.into_boxed_slice(),
                entry_point: binding_info.entry_point.clone(),
                resource_bindings: resource_bindings.into_boxed_slice(),
                shader_module_content: ffi::ShaderModuleContent::Wgsl(content),
            })
        }
    }
}

fn ir_native_interface(program: &ast::Program, context: &mut Context) -> ffi::NativeInterface {
    let mut types = StableVec::new();
    let mut external_functions = StableVec::new();

    for declaration in &program.declarations {
        match declaration {
            ast::Declaration::TypeDecl(ast::TypeDecl::FFI(t)) => {
                types.add(ffi_to_ffi(t.clone(), context));
            }
            ast::Declaration::ExternalFunction(external) => {
                external_functions.add(ir_external(external, context));
            }
            _ => {}
        }
    }

    ffi::NativeInterface {
        types,
        external_functions,
        effects: StableVec::new(), // todo: add
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
                ast::LocalTypeInfo::Slot {
                    storage_type,
                    queue_stage,
                    queue_place,
                } => ir::Type::Slot {
                    storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                    queue_stage: queue_stage.clone(),
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Fence { queue_place } => ir::Type::Fence {
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Buffer {
                    storage_place,
                    static_layout_opt,
                } => ir::Type::Buffer {
                    storage_place: storage_place.clone(),
                    static_layout_opt: static_layout_opt.clone(),
                },
                ast::LocalTypeInfo::Event { place } => ir::Type::Event {
                    place: place.clone(),
                },
                ast::LocalTypeInfo::BufferSpace => ir::Type::BufferSpace,
                ast::LocalTypeInfo::SchedulingJoin {} => ir::Type::SchedulingJoin {},
            })
        }
        ast::TypeDecl::FFI(name) => None,
    }
}

fn ir_node(node: &ast::NamedNode, context: &mut Context) -> Option<ir::Node> {
    match &node.node {
        ast::Node::None => Some(ir::Node::None),
        ast::Node::Phi { index } => Some(ir::Node::Phi {
            index: reject_hole(index.as_ref()).clone(),
        }),
        ast::Node::ExtractResult { node_id, index } => Some(ir::Node::ExtractResult {
            node_id: context.node_id(reject_hole(node_id.as_ref())),
            index: reject_hole(index.as_ref()).clone(),
        }),
        ast::Node::Constant { value, type_id } => {
            let unwrapped_value = reject_hole(value.clone());
            let unwrapped_type = reject_hole(type_id.clone());
            let parsed_value = match &unwrapped_type {
                ast::TypeId::Local(_) => {
                    panic!("Cannot have a local type constant {:?}", type_id)
                }
                ast::TypeId::FFI(t) => match t {
                    FFIType::U64 => ir::Constant::U64(unwrapped_value.parse::<u64>().unwrap()),
                    FFIType::I32 => ir::Constant::I32(unwrapped_value.parse::<i32>().unwrap()),
                    FFIType::I64 => ir::Constant::I64(unwrapped_value.parse::<i64>().unwrap()),
                    _ => panic!("Unsupported constant type {:?}", type_id),
                },
            };
            Some(ir::Node::Constant {
                value: parsed_value,
                type_id: context.loc_type_id(&unwrapped_type),
            })
        }
        ast::Node::CallValueFunction {
            function_id,
            arguments,
        } => {
            let name = reject_hole(function_id.clone());
            let mapped_arguments: Vec<ir::NodeId> = reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect();
            let function_id = context.funclet_indices.get_funclet(&name.0).unwrap();
            Some(ir::Node::CallValueFunction {
                function_id: context
                    .function_classes
                    .get(&FunctionClassId(name.0.clone()))
                    .unwrap(),
                arguments: mapped_arguments.into_boxed_slice(),
            })
        }
        ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => Some(ir::Node::Select {
            condition: context.node_id(reject_hole(condition.as_ref())),
            true_case: context.node_id(reject_hole(true_case.as_ref())),
            false_case: context.node_id(reject_hole(false_case.as_ref())),
        }),
        ast::Node::CallExternalCpu {
            external_function_id,
            arguments,
        } => unreachable!(), // unification of calls
        ast::Node::CallExternalGpuCompute {
            external_function_id,
            dimensions,
            arguments,
        } => unreachable!(), // unification of calls
        ast::Node::AllocTemporary {
            place,
            storage_type,
            operation,
        } => Some(ir::Node::AllocTemporary {
            place: reject_hole(place.clone()),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            operation: remote_conversion(reject_hole(operation.as_ref()), context),
        }),
        ast::Node::UnboundSlot {
            place,
            storage_type,
            operation,
        } => Some(ir::Node::UnboundSlot {
            place: reject_hole(place.clone()),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            operation: remote_conversion(reject_hole(operation.as_ref()), context),
        }),
        ast::Node::Drop { node } => Some(ir::Node::Drop {
            node: context.node_id(reject_hole(node.as_ref())),
        }),
        ast::Node::StaticAllocFromStaticBuffer {
            buffer,
            place,
            storage_type,
            operation,
        } => Some(ir::Node::StaticAllocFromStaticBuffer {
            buffer: context.node_id(reject_hole(buffer.as_ref())),
            place: reject_hole(place.clone()),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            operation: remote_conversion(reject_hole(operation.as_ref()), context),
        }),
        ast::Node::EncodeDo {
            place,
            operation,
            inputs,
            outputs,
        } => Some(ir::Node::EncodeDo {
            place: reject_hole(place.clone()),
            operation: remote_conversion(reject_hole(operation.as_ref()), context),
            inputs: reject_hole(inputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            outputs: reject_hole(outputs.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        ast::Node::EncodeCopy {
            place,
            input,
            output,
        } => Some(ir::Node::EncodeCopy {
            place: reject_hole(place.clone()),
            input: context.node_id(reject_hole(input.as_ref())),
            output: context.node_id(reject_hole(output.as_ref())),
        }),
        ast::Node::Submit { place, event } => Some(ir::Node::Submit {
            place: reject_hole(place.clone()),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        ast::Node::EncodeFence { place, event } => Some(ir::Node::EncodeFence {
            place: reject_hole(place.clone()),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        ast::Node::SyncFence {
            place,
            fence,
            event,
        } => Some(ir::Node::SyncFence {
            place: reject_hole(place.clone()),
            fence: context.node_id(reject_hole(fence.as_ref())),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        ast::Node::InlineJoin {
            funclet,
            captures,
            continuation,
        } => Some(ir::Node::InlineJoin {
            funclet: context
                .funclet_indices
                .get_funclet(&reject_hole(funclet.as_ref()).0)
                .unwrap()
                .clone(),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        }),
        ast::Node::SerializedJoin {
            funclet,
            captures,
            continuation,
        } => Some(ir::Node::SerializedJoin {
            funclet: context
                .funclet_indices
                .get_funclet(&reject_hole(funclet.as_ref()).0)
                .unwrap()
                .clone(),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        }),
        ast::Node::DefaultJoin => Some(ir::Node::DefaultJoin),
        ast::Node::SubmissionEvent {
            here_place,
            there_place,
            local_past,
        } => Some(ir::Node::SubmissionEvent {
            here_place: reject_hole(here_place.clone()),
            there_place: reject_hole(there_place.clone()),
            local_past: context.node_id(reject_hole(local_past.as_ref())),
        }),
        ast::Node::SynchronizationEvent {
            here_place,
            there_place,
            local_past,
            remote_local_past,
        } => Some(ir::Node::SynchronizationEvent {
            here_place: reject_hole(here_place.clone()),
            there_place: reject_hole(there_place.clone()),
            local_past: context.node_id(reject_hole(local_past.as_ref())),
            remote_local_past: context.node_id(reject_hole(remote_local_past.as_ref())),
        }),
        ast::Node::SeparatedLinearSpace { place, space } => Some(ir::Node::SeparatedLinearSpace {
            place: reject_hole(place.clone()),
            space: context.node_id(reject_hole(space.as_ref())),
        }),
        ast::Node::MergedLinearSpace { place, spaces } => Some(ir::Node::MergedLinearSpace {
            place: reject_hole(place.clone()),
            spaces: reject_hole(spaces.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
    }
}

fn ir_tail_edge(tail: &ast::TailEdge, context: &mut Context) -> Option<ir::TailEdge> {
    match tail {
        ast::TailEdge::Return { return_values } => Some(ir::TailEdge::Return {
            return_values: reject_hole(return_values.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        ast::TailEdge::Yield {
            external_function_id,
            yielded_nodes,
            next_funclet,
            continuation_join,
            arguments,
        } => Some(ir::TailEdge::Yield {
            external_function_id: ffi::ExternalFunctionId(
                context
                    .funclet_indices
                    .get_funclet(&reject_hole(external_function_id.as_ref()).0)
                    .unwrap()
                    .clone(),
            ),
            yielded_nodes: reject_hole(yielded_nodes.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            next_funclet: context
                .funclet_indices
                .get_funclet(&reject_hole(next_funclet.as_ref()).0)
                .unwrap()
                .clone(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        ast::TailEdge::Jump { join, arguments } => Some(ir::TailEdge::Jump {
            join: context
                .funclet_indices
                .get_funclet(&reject_hole(join.as_ref()).0)
                .unwrap()
                .clone(),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        ast::TailEdge::ScheduleCall {
            value_operation,
            callee_funclet_id,
            callee_arguments,
            continuation_join,
        } => Some(ir::TailEdge::ScheduleCall {
            value_operation: remote_conversion(reject_hole(value_operation.as_ref()), context),
            callee_funclet_id: context
                .funclet_indices
                .get_funclet(&reject_hole(callee_funclet_id.as_ref()).0)
                .unwrap()
                .clone(),
            callee_arguments: reject_hole(callee_arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        }),
        ast::TailEdge::ScheduleSelect {
            value_operation,
            condition,
            callee_funclet_ids,
            callee_arguments,
            continuation_join,
        } => Some(ir::TailEdge::ScheduleSelect {
            value_operation: remote_conversion(reject_hole(value_operation.as_ref()), context),
            condition: context.node_id(reject_hole(condition.as_ref())),
            callee_funclet_ids: reject_hole(callee_funclet_ids.as_ref())
                .iter()
                .map(|n| {
                    context
                        .funclet_indices
                        .get_funclet(&reject_hole(n.as_ref()).0)
                        .unwrap()
                        .clone()
                })
                .collect(),
            callee_arguments: reject_hole(callee_arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        }),
        ast::TailEdge::DynamicAllocFromBuffer {
            buffer,
            arguments,
            dynamic_allocation_size_slots,
            success_funclet_id,
            failure_funclet_id,
            continuation_join,
        } => Some(ir::TailEdge::DynamicAllocFromBuffer {
            buffer: context.node_id(reject_hole(buffer.as_ref())),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            dynamic_allocation_size_slots: reject_hole(dynamic_allocation_size_slots.as_ref())
                .iter()
                .map(|o| reject_hole(o.as_ref()).as_ref().map(|n| context.node_id(n)))
                .collect(),
            success_funclet_id: context
                .funclet_indices
                .get_funclet(&reject_hole(success_funclet_id.as_ref()).0)
                .unwrap()
                .clone(),
            failure_funclet_id: context
                .funclet_indices
                .get_funclet(&reject_hole(failure_funclet_id.as_ref()).0)
                .unwrap()
                .clone(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        }),
    }
}

fn map_tag(tag: &ast::Tag, context: &mut Context) -> Option<(Hole<FuncletId>, ir::Tag)> {
    match tag {
        ast::Tag::None => None,
        ast::Tag::Node(ast::RemoteNodeId {
            funclet_name,
            node_name,
        }) => Some((
            funclet_name.clone(),
            ir::Tag::Node {
                node_id: context.remote_node_id(
                    reject_hole(funclet_name.as_ref()),
                    reject_hole(node_name.as_ref()),
                ),
            },
        )),
        ast::Tag::Input(ast::RemoteNodeId {
            funclet_name,
            node_name,
        }) => Some((
            funclet_name.clone(),
            ir::Tag::Input {
                index: context.remote_node_id(
                    reject_hole(funclet_name.as_ref()),
                    reject_hole(node_name.as_ref()),
                ),
            },
        )),
        ast::Tag::Output(ast::RemoteNodeId {
            funclet_name,
            node_name,
        }) => Some((
            funclet_name.clone(),
            ir::Tag::Output {
                index: context.remote_node_id(
                    reject_hole(funclet_name.as_ref()),
                    reject_hole(node_name.as_ref()),
                ),
            },
        )),
        ast::Tag::Halt(ast::RemoteNodeId {
            funclet_name,
            node_name,
        }) => Some((
            funclet_name.clone(),
            ir::Tag::Halt {
                index: context.remote_node_id(
                    reject_hole(funclet_name.as_ref()),
                    reject_hole(node_name.as_ref()),
                ),
            },
        )),
    }
}

fn ir_schedule_binding(
    funclet_header: &ast::FuncletHeader,
    implicit_tags: &Option<(ast::Tag, ast::Tag)>,
    value: &Option<FuncletId>,
    timeline: &Option<FuncletId>,
    spatial: &Option<FuncletId>,
    context: &mut Context,
) -> ir::FuncletSpecBinding {
    #[derive(Debug)]
    struct TagSet {
        value: ir::Tag,
        spatial: ir::Tag,
        timeline: ir::Tag,
    }

    fn gen_tags(
        tags: &Vec<ast::Tag>,
        value: &Option<FuncletId>,
        timeline: &Option<FuncletId>,
        spatial: &Option<FuncletId>,
        context: &mut Context,
    ) -> TagSet {
        let mut result = TagSet {
            value: ir::Tag::None,
            spatial: ir::Tag::None,
            timeline: ir::Tag::None,
        };
        for tag in tags {
            let data = map_tag(tag, context);
            match data {
                None => {}
                Some((funclet_name, new_tag)) => {
                    let fnid = reject_hole(funclet_name.clone());
                    if fnid == value.clone().unwrap_or(FuncletId("".to_string())) {
                        result.value = new_tag;
                    } else if fnid == spatial.clone().unwrap_or(FuncletId("".to_string())) {
                        result.spatial = new_tag;
                    } else if fnid == timeline.clone().unwrap_or(FuncletId("".to_string())) {
                        result.timeline = new_tag;
                    } else {
                        panic!("Unknown tag funclet id {:?}", funclet_name);
                    }
                }
            };
        }
        result
    }

    // probably a better way to do this, but whatever
    let mut value_in = ir::Tag::None;
    let mut value_out = ir::Tag::None;
    let mut spatial_in = ir::Tag::None;
    let mut spatial_out = ir::Tag::None;
    let mut timeline_in = ir::Tag::None;
    let mut timeline_out = ir::Tag::None;

    match implicit_tags {
        None => {}
        Some((in_tag, out_tag)) => {
            let in_tags = gen_tags(&vec![in_tag.clone()], value, timeline, spatial, context);
            let out_tags = gen_tags(&vec![out_tag.clone()], value, timeline, spatial, context);
            value_in = in_tags.value;
            value_out = out_tags.value;
            spatial_in = in_tags.spatial;
            spatial_out = out_tags.spatial;
            timeline_in = in_tags.timeline;
            timeline_out = out_tags.timeline;
        }
    }

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
        let tags = gen_tags(&arg.tags, value, timeline, spatial, context);
        input_tags.value_tags.push(tags.value.clone());
        input_tags.spatial_tags.push(tags.spatial.clone());
        input_tags.timeline_tags.push(tags.timeline.clone());
    }

    for arg in &funclet_header.ret {
        let tags = gen_tags(&arg.tags, value, timeline, spatial, context);
        output_tags.value_tags.push(tags.value.clone());
        output_tags.spatial_tags.push(tags.spatial.clone());
        output_tags.timeline_tags.push(tags.timeline.clone());
    }

    let mut implicit_in_tag = ir::Tag::None;
    let mut implicit_out_tag = ir::Tag::None;

    match implicit_tags {
        None => {}
        Some((in_tag, out_tag)) => {
            implicit_in_tag = map_tag(in_tag, context).unwrap_or((None, ir::Tag::None)).1;
            implicit_out_tag = map_tag(out_tag, context).unwrap_or((None, ir::Tag::None)).1;
        }
    }

    ir::FuncletSpecBinding::ScheduleExplicit {
        value: ir::FuncletSpec {
            funclet_id_opt: value
                .clone()
                .map(|f| context.funclet_indices.get_funclet(&f.0).unwrap()),
            input_tags: input_tags.value_tags.into_boxed_slice(),
            output_tags: output_tags.value_tags.into_boxed_slice(),
            implicit_in_tag: ir::Tag::None,
            implicit_out_tag: ir::Tag::None,
        },
        spatial: ir::FuncletSpec {
            funclet_id_opt: spatial
                .clone()
                .map(|f| context.funclet_indices.get_funclet(&f.0).unwrap()),
            input_tags: input_tags.spatial_tags.into_boxed_slice(),
            output_tags: output_tags.spatial_tags.into_boxed_slice(),
            implicit_in_tag: ir::Tag::None,
            implicit_out_tag: ir::Tag::None,
        },
        timeline: ir::FuncletSpec {
            // assume implicit is timeline for now?
            funclet_id_opt: timeline
                .clone()
                .map(|f| context.funclet_indices.get_funclet(&f.0).unwrap()),
            input_tags: input_tags.timeline_tags.into_boxed_slice(),
            output_tags: output_tags.timeline_tags.into_boxed_slice(),
            implicit_in_tag,
            implicit_out_tag,
        },
    }
}

fn ir_spec_binding(
    funclet_header: &ast::FuncletHeader,
    context: &mut Context,
) -> ir::FuncletSpecBinding {
    match &funclet_header.binding {
        ast::FuncletBinding::None => ir::FuncletSpecBinding::None,
        ast::FuncletBinding::ValueBinding(ast::FunctionClassBinding {
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
            value,
            timeline,
            spatial,
        }) => ir_schedule_binding(
            funclet_header,
            implicit_tags,
            value,
            timeline,
            spatial,
            context,
        ),
    }
}

fn ir_funclet(funclet: &ast::Funclet, context: &mut Context) -> ir::Funclet {
    context.location.funclet_name = funclet.header.name.clone();
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
        match command.as_ref().unwrap() {
            ast::Command::Node(node) => {
                context.location.node_name = node.name.clone();
                nodes.push(ir_node(&node, context).unwrap());
            }
            ast::Command::TailEdge(tail) => {
                if tail_edge.is_some() {
                    panic!("More than one tail edge in {:?}", funclet.header.name);
                }
                tail_edge = Some(ir_tail_edge(tail, context).unwrap());
            }
        }
    }

    ir::Funclet {
        kind: funclet.kind.clone(),
        spec_binding: ir_spec_binding(&funclet.header, context),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        tail_edge: tail_edge.unwrap(), // actually safe, oddly enough
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
                ast::FuncletBinding::ValueBinding(binding) => {
                    let current_id = context
                        .funclet_indices
                        .get_funclet(&f.header.name.0)
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
                _ => {}
            },
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
            .unwrap()
            .clone(),
        effect_id_opt: None,
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
    let explicated_program = explication::explicate(program);
    let mut context = Context::new(&explicated_program);
    frontend::Definition {
        version: ir_version(&explicated_program.version, &mut context),
        program: ir_program(&explicated_program, &mut context),
    }
}
