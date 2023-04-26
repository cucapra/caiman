use crate::assembly::explication_context::Context;
use crate::assembly::explication_context::FuncletLocation;
use crate::assembly::explication_explicator;
use crate::assembly::explication_util::*;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_ast::{
    ExternalCpuFunctionId, ExternalGpuFunctionId, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::ir::ffi;
use crate::{assembly_ast, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

// for reading GPU stuff
use crate::stable_vec::StableVec;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Translation

fn ir_version(version: &assembly_ast::Version, _: &mut Context) -> (u32, u32, u32) {
    (version.major, version.minor, version.detailed)
}

fn ir_external_cpu(
    external: &assembly_ast::ExternalCpuFunction,
    context: &mut Context,
) -> ffi::ExternalCpuFunction {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();

    for name in &external.input_types {
        input_types.push(ffi::TypeId(context.ffi_type_id(name)))
    }
    for name in &external.output_types {
        output_types.push(ffi::TypeId(context.ffi_type_id(name)))
    }

    ffi::ExternalCpuFunction {
        name: external.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
    }
}

fn ir_external_gpu_resource(
    d: &assembly_ast::ExternalGpuFunctionResourceBinding,
    input_args: &Vec<String>,
    output_args: &Vec<String>,
    context: &mut Context,
) -> ffi::ExternalGpuFunctionResourceBinding {
    fn local_name(val: &String, input_args: &Vec<String>, output_args: &Vec<String>) -> usize {
        let mut index = 0;
        for arg in input_args {
            if val == arg {
                return index;
            }
            index += 1;
        }
        let mut index = 0;
        for arg in output_args {
            if val == arg {
                return index;
            }
            index += 1;
        }
        panic!("Unknown GPU variable {:?}", val)
    }
    let group = local_name(&d.group, input_args, output_args);
    let binding = local_name(&d.binding, input_args, output_args);
    let input = d.input.as_ref().map(|x| local_name(&x, input_args, output_args));
    let output = d.output.as_ref().map(|x| local_name(&x, input_args, output_args));
    ffi::ExternalGpuFunctionResourceBinding {
        group,
        binding,
        input,
        output,
    }
}

fn ir_external_gpu(
    external: &assembly_ast::ExternalGpuFunction,
    context: &mut Context,
) -> ffi::ExternalGpuFunction {
    let mut input_types = Vec::new();
    let mut input_args = Vec::new();
    let mut output_args = Vec::new();
    let mut output_types = Vec::new();
    let mut resource_bindings = Vec::new();

    for arg in &external.input_args {
        input_types.push(ffi::TypeId(context.ffi_type_id(&arg.0)));
        input_args.push(arg.1.clone());
    }
    for arg in &external.output_types {
        output_types.push(ffi::TypeId(context.ffi_type_id(&arg.0)));
        let arg_name = arg.1.clone();
        for iarg in &input_args {
            if arg_name == *iarg {
                panic!("Duplicate input and output name {}", iarg);
            }
        }
        output_args.push(arg_name);
    }
    for resource in &external.resource_bindings {
        resource_bindings.push(ir_external_gpu_resource(
            resource,
            &input_args,
            &output_args,
            context,
        ));
    }

    // Very silly
    let input_path = Path::new(&external.shader_module);
    assert!(external.shader_module.ends_with(".wgsl"));
    let mut input_file = match File::open(&input_path) {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };
    let mut content = String::new();
    match input_file.read_to_string(&mut content) {
        Err(why) => panic!("Couldn't read file: {}", why),
        Ok(_) => (),
    };

    ffi::ExternalGpuFunction {
        name: external.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        entry_point: external.entry_point.clone(),
        resource_bindings: resource_bindings.into_boxed_slice(),
        shader_module_content: ffi::ShaderModuleContent::Wgsl(content),
    }
}

fn ir_native_interface(
    program: &assembly_ast::Program,
    context: &mut Context,
) -> ffi::NativeInterface {
    let mut types = StableVec::new();
    let mut external_cpu_functions = StableVec::new();
    let mut external_gpu_functions = StableVec::new();

    for typ in &program.types {
        match typ {
            assembly_ast::TypeDecl::FFI(t) => {
                types.add(ffi_to_ffi(t.clone(), context));
            }
            _ => {}
        }
    }

    for def in &program.funclets {
        match def {
            assembly_ast::FuncletDef::ExternalCPU(external) => {
                external_cpu_functions.add(ir_external_cpu(external, context));
            }
            assembly_ast::FuncletDef::ExternalGPU(external) => {
                external_gpu_functions.add(ir_external_gpu(external, context));
            }
            _ => {}
        }
    }

    ffi::NativeInterface {
        types,
        external_cpu_functions,
        external_gpu_functions,
    }
}

fn ir_types(types: &Vec<assembly_ast::TypeDecl>, context: &mut Context) -> StableVec<ir::Type> {
    let mut result = StableVec::new();
    for type_decl in types {
        let new_type = match type_decl {
            assembly_ast::TypeDecl::Local(typ) => {
                Some(match &typ.data {
                    // only supported custom types atm
                    assembly_ast::LocalTypeInfo::NativeValue { storage_type } => {
                        ir::Type::NativeValue {
                            storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                        }
                    }
                    assembly_ast::LocalTypeInfo::Slot {
                        storage_type,
                        queue_stage,
                        queue_place,
                    } => ir::Type::Slot {
                        storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                        queue_stage: queue_stage.clone(),
                        queue_place: queue_place.clone(),
                    },
                    assembly_ast::LocalTypeInfo::Fence { queue_place } => ir::Type::Fence {
                        queue_place: queue_place.clone(),
                    },
                    assembly_ast::LocalTypeInfo::Buffer {
                        storage_place,
                        static_layout_opt,
                    } => ir::Type::Buffer {
                        storage_place: storage_place.clone(),
                        static_layout_opt: static_layout_opt.clone(),
                    },
                    assembly_ast::LocalTypeInfo::Event { place } => ir::Type::Event {
                        place: place.clone(),
                    },
                    assembly_ast::LocalTypeInfo::BufferSpace => ir::Type::BufferSpace,
                    assembly_ast::LocalTypeInfo::SchedulingJoin {} => ir::Type::SchedulingJoin {},
                })
            }
            assembly_ast::TypeDecl::FFI(name) => None,
        };
        match new_type {
            Some(t) => {
                result.add(t);
            }
            None => {}
        }
    }
    result
}

fn ir_node(node: &assembly_ast::NamedNode, context: &mut Context) -> Option<ir::Node> {
    match &node.node {
        assembly_ast::Node::None => Some(ir::Node::None),
        assembly_ast::Node::Phi { index } => Some(ir::Node::Phi {
            index: reject_hole(index.as_ref()).clone(),
        }),
        assembly_ast::Node::ExtractResult { node_id, index } => Some(ir::Node::ExtractResult {
            node_id: context.node_id(reject_hole(node_id.as_ref())),
            index: reject_hole(index.as_ref()).clone(),
        }),
        assembly_ast::Node::Constant { value, type_id } => {
            let unwrapped_value = reject_hole(value.clone());
            let unwrapped_type = reject_hole(type_id.clone());
            let parsed_value = match &unwrapped_type {
                assembly_ast::Type::Local(_) => {
                    panic!("Cannot have a custom type constant {:?}", type_id)
                }
                assembly_ast::Type::FFI(t) => match t {
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
        assembly_ast::Node::CallValueFunction {
            function_id,
            arguments,
        } => {
            unreachable!() // arbitrary unification of calls
        }
        assembly_ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => Some(ir::Node::Select {
            condition: context.node_id(reject_hole(condition.as_ref())),
            true_case: context.node_id(reject_hole(true_case.as_ref())),
            false_case: context.node_id(reject_hole(false_case.as_ref())),
        }),
        assembly_ast::Node::CallExternalCpu {
            external_function_id,
            arguments,
        } => {
            let name = reject_hole(external_function_id.clone());
            let mapped_arguments = reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect();
            let id = context.funclet_indices.get(&name).unwrap();
            Some(match context.funclet_indices.get_loc(&name).unwrap() {
                FuncletLocation::Local => {
                    panic!("Cannot directly call local funclet {}", name)
                }
                FuncletLocation::Value => ir::Node::CallValueFunction {
                    function_id: id.clone(),
                    arguments: mapped_arguments,
                },
                FuncletLocation::Cpu => ir::Node::CallExternalCpu {
                    external_function_id: id.clone(),
                    arguments: mapped_arguments,
                },
                FuncletLocation::Gpu => ir::Node::CallExternalGpuCompute {
                    external_function_id: id.clone(),
                    dimensions: Box::new([]), // explicitly empty
                    arguments: mapped_arguments,
                },
            })
        }
        assembly_ast::Node::CallExternalGpuCompute {
            external_function_id,
            dimensions,
            arguments,
        } => Some(ir::Node::CallExternalGpuCompute {
            external_function_id: context
                .funclet_indices
                .get(reject_hole(external_function_id.as_ref()))
                .unwrap(),
            dimensions: reject_hole(dimensions.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        assembly_ast::Node::AllocTemporary {
            place,
            storage_type,
            operation,
        } => explication_explicator::explicate_allocate_temporary(
            place,
            storage_type,
            operation,
            context,
        ),
        assembly_ast::Node::UnboundSlot {
            place,
            storage_type,
            operation,
        } => Some(ir::Node::UnboundSlot {
            place: reject_hole(place.clone()),
            storage_type: ffi::TypeId(context.loc_type_id(reject_hole(storage_type.as_ref()))),
            operation: remote_conversion(reject_hole(operation.as_ref()), context),
        }),
        assembly_ast::Node::Drop { node } => Some(ir::Node::Drop {
            node: context.node_id(reject_hole(node.as_ref())),
        }),
        assembly_ast::Node::StaticAllocFromStaticBuffer {
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
        assembly_ast::Node::EncodeDo {
            place,
            operation,
            inputs,
            outputs,
        } => {
            explication_explicator::explicate_encode_do(place, operation, inputs, outputs, context)
        }
        assembly_ast::Node::EncodeCopy {
            place,
            input,
            output,
        } => Some(ir::Node::EncodeCopy {
            place: reject_hole(place.clone()),
            input: context.node_id(reject_hole(input.as_ref())),
            output: context.node_id(reject_hole(output.as_ref())),
        }),
        assembly_ast::Node::Submit { place, event } => Some(ir::Node::Submit {
            place: reject_hole(place.clone()),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        assembly_ast::Node::EncodeFence { place, event } => Some(ir::Node::EncodeFence {
            place: reject_hole(place.clone()),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        assembly_ast::Node::SyncFence {
            place,
            fence,
            event,
        } => Some(ir::Node::SyncFence {
            place: reject_hole(place.clone()),
            fence: context.node_id(reject_hole(fence.as_ref())),
            event: remote_conversion(reject_hole(event.as_ref()), context),
        }),
        assembly_ast::Node::InlineJoin {
            funclet,
            captures,
            continuation,
        } => Some(ir::Node::InlineJoin {
            funclet: context
                .funclet_indices
                .get(reject_hole(funclet.as_ref()))
                .unwrap()
                .clone(),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        }),
        assembly_ast::Node::SerializedJoin {
            funclet,
            captures,
            continuation,
        } => Some(ir::Node::SerializedJoin {
            funclet: context
                .funclet_indices
                .get(reject_hole(funclet.as_ref()))
                .unwrap()
                .clone(),
            captures: reject_hole(captures.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation: context.node_id(reject_hole(continuation.as_ref())),
        }),
        assembly_ast::Node::DefaultJoin => Some(ir::Node::DefaultJoin),
        assembly_ast::Node::SubmissionEvent {
            here_place,
            there_place,
            local_past,
        } => Some(ir::Node::SubmissionEvent {
            here_place: reject_hole(here_place.clone()),
            there_place: reject_hole(there_place.clone()),
            local_past: context.node_id(reject_hole(local_past.as_ref())),
        }),
        assembly_ast::Node::SynchronizationEvent {
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
        assembly_ast::Node::SeparatedLinearSpace { place, space } => {
            Some(ir::Node::SeparatedLinearSpace {
                place: reject_hole(place.clone()),
                space: context.node_id(reject_hole(space.as_ref())),
            })
        }
        assembly_ast::Node::MergedLinearSpace { place, spaces } => {
            Some(ir::Node::MergedLinearSpace {
                place: reject_hole(place.clone()),
                spaces: reject_hole(spaces.as_ref())
                    .iter()
                    .map(|n| context.node_id(reject_hole(n.as_ref())))
                    .collect(),
            })
        }
    }
}

fn ir_tail_edge(tail: &assembly_ast::TailEdge, context: &mut Context) -> Option<ir::TailEdge> {
    match tail {
        assembly_ast::TailEdge::Return { return_values } => Some(ir::TailEdge::Return {
            return_values: reject_hole(return_values.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        assembly_ast::TailEdge::Yield {
            pipeline_yield_point_id,
            yielded_nodes,
            next_funclet,
            continuation_join,
            arguments,
        } => Some(ir::TailEdge::Yield {
            pipeline_yield_point_id: reject_hole(pipeline_yield_point_id.clone()),
            yielded_nodes: reject_hole(yielded_nodes.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            next_funclet: context
                .funclet_indices
                .get(reject_hole(next_funclet.as_ref()))
                .unwrap()
                .clone(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        assembly_ast::TailEdge::Jump { join, arguments } => Some(ir::TailEdge::Jump {
            join: context.node_id(reject_hole(join.as_ref())),
            arguments: reject_hole(arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
        }),
        assembly_ast::TailEdge::ScheduleCall {
            value_operation,
            callee_funclet_id,
            callee_arguments,
            continuation_join,
        } => Some(ir::TailEdge::ScheduleCall {
            value_operation: remote_conversion(reject_hole(value_operation.as_ref()), context),
            callee_funclet_id: context
                .funclet_indices
                .get(reject_hole(callee_funclet_id.as_ref()))
                .unwrap()
                .clone(),
            callee_arguments: reject_hole(callee_arguments.as_ref())
                .iter()
                .map(|n| context.node_id(reject_hole(n.as_ref())))
                .collect(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        }),
        assembly_ast::TailEdge::ScheduleSelect {
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
                        .get(reject_hole(n.as_ref()))
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
        assembly_ast::TailEdge::DynamicAllocFromBuffer {
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
                .get(reject_hole(success_funclet_id.as_ref()))
                .unwrap()
                .clone(),
            failure_funclet_id: context
                .funclet_indices
                .get(reject_hole(failure_funclet_id.as_ref()))
                .unwrap()
                .clone(),
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        }),
    }
}

fn ir_funclet(funclet: &assembly_ast::Funclet, context: &mut Context) -> Option<ir::Funclet> {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut nodes = Vec::new();

    for (mut index, input_type) in funclet.header.args.iter().enumerate() {
        match input_type.0.clone() {
            // adding the phi node
            None => {}
            Some(s) => nodes.push(ir::Node::Phi { index }),
        };
        input_types.push(context.loc_type_id(&input_type.1));
    }

    for output_type in funclet.header.ret.iter() {
        output_types.push(context.loc_type_id(&output_type.1));
    }

    for command in &funclet.commands {
        let node = &reject_hole(command.as_ref());
        context.location.node_id = node.name.clone();
        nodes.push(ir_node(&node, context).unwrap());
    }

    let tail_edge = ir_tail_edge(reject_hole(funclet.tail_edge.as_ref()), context).unwrap();

    Some(ir::Funclet {
        kind: funclet.kind.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        tail_edge, // actually safe, oddly enough
    })
}

fn ir_funclets(
    funclets: &assembly_ast::FuncletDefs,
    context: &mut Context,
) -> StableVec<ir::Funclet> {
    let mut result = StableVec::new();
    for def in funclets {
        match def {
            assembly_ast::FuncletDef::Local(f) => {
                let name = &f.header.name;
                context.location.funclet_id = name.clone();
                result.add(ir_funclet(f, context).unwrap());
                context.explicate_funclet(name.clone()); // separate operation cause order
            }
            _ => {}
        }
    }
    context.reset_location(); // reset again to catch mistakes
    result
}

fn ir_value_function(
    function: &assembly_ast::ValueFunction,
    context: &mut Context,
) -> ir::ValueFunction {
    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut default_funclet_id = None;

    for typ in &function.input_types {
        input_types.push(context.loc_type_id(&typ));
    }
    for typ in &function.output_types {
        output_types.push(context.loc_type_id(&typ));
    }
    if function.allowed_funclets.len() > 0 {
        let name = function.allowed_funclets.get(0).unwrap();
        let index = context.funclet_indices.get_loc(name).map(|x| match x {
            FuncletLocation::Local => context.funclet_indices.get(name).unwrap(),
            _ => panic!("Non-local funclet used for value function {}", name),
        });
        default_funclet_id = index;
    }

    ir::ValueFunction {
        name: function.name.clone(),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        default_funclet_id,
    }
}

fn ir_value_functions(
    funclets: &assembly_ast::FuncletDefs,
    context: &mut Context,
) -> StableVec<ir::ValueFunction> {
    let mut result = StableVec::new();
    for def in funclets {
        match def {
            assembly_ast::FuncletDef::ValueFunction(f) => {
                result.add(ir_value_function(f, context));
            }
            _ => {}
        }
    }
    result
}

fn ir_pipelines(pipelines: &assembly_ast::Pipelines, context: &mut Context) -> Vec<ir::Pipeline> {
    let mut result = Vec::new();
    for (name, pipeline) in pipelines {
        let new_pipeline = ir::Pipeline {
            name: name.clone(),
            entry_funclet: context.funclet_indices.get(pipeline).unwrap().clone(),
            yield_points: Default::default(),
        };
        result.push(new_pipeline);
    }
    result
}

fn ir_value_extra(_: &assembly_ast::UncheckedDict, _: &mut Context) -> ir::ValueFuncletExtra {
    todo!()
}

fn ir_value_extras(
    funclets: &assembly_ast::FuncletDefs,
    extras: &assembly_ast::Extras,
    context: &mut Context,
) -> HashMap<ir::FuncletId, ir::ValueFuncletExtra> {
    let mut result = HashMap::new();
    for funclet in funclets {
        match funclet {
            assembly_ast::FuncletDef::Local(f) => match f.kind {
                ir::FuncletKind::Value => {
                    let name = &f.header.name;
                    for extra in extras {
                        if extra.0 == name {
                            context.location.funclet_id = extra.0.clone();
                            let index = context.funclet_indices.get(extra.0).unwrap().clone();
                            if result.contains_key(&index) {
                                panic!("Duplicate extras for {:?}", name);
                            }
                            result.insert(index, ir_value_extra(extra.1, context));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    result
}

fn ir_scheduling_extra(
    d: &assembly_ast::UncheckedDict,
    context: &mut Context,
) -> ir::SchedulingFuncletExtra {
    let index = value_funclet_raw_id(d.get(&as_key("value")).unwrap(), context);
    ir::SchedulingFuncletExtra {
        value_funclet_id: index,
        input_slots: value_index_var_dict(
            d.get(&as_key("input_slots")).unwrap(),
            value_slot_info,
            false,
            context,
        ),
        output_slots: value_index_var_dict(
            d.get(&as_key("output_slots")).unwrap(),
            value_slot_info,
            true,
            context,
        ),
        input_fences: value_index_var_dict(
            d.get(&as_key("input_fences")).unwrap(),
            value_fence_info,
            false,
            context,
        ),
        output_fences: value_index_var_dict(
            d.get(&as_key("output_fences")).unwrap(),
            value_fence_info,
            true,
            context,
        ),
        input_buffers: value_index_var_dict(
            d.get(&as_key("input_buffers")).unwrap(),
            value_buffer_info,
            false,
            context,
        ),
        output_buffers: value_index_var_dict(
            d.get(&as_key("output_buffers")).unwrap(),
            value_buffer_info,
            true,
            context,
        ),
        in_timeline_tag: value_dict_timeline_tag(
            d.get(&as_key("in_timeline_tag")).unwrap(),
            context,
        ),
        out_timeline_tag: value_dict_timeline_tag(
            d.get(&as_key("out_timeline_tag")).unwrap(),
            context,
        ),
    }
}

fn setup_extras(extras: &assembly_ast::Extras, context: &mut Context) {
    let mut built_extras = HashMap::new();
    for (name, extra) in extras {
        let index = context.funclet_indices.get(&name);
        context.location.funclet_id = name.clone();
        built_extras.insert(
            name.clone(),
            ir_scheduling_extra(&extra, context),
        );
    }
    context.schedule_extras = built_extras;
}

fn ir_scheduling_extras(
    context: &mut Context,
) -> HashMap<ir::FuncletId, ir::SchedulingFuncletExtra> {
    // duplicating some code...but it's annoying to fix and I'm lazy
    let mut result = HashMap::new();
    let mut schedule_extras = HashMap::new();
    std::mem::swap(&mut schedule_extras, &mut context.schedule_extras);
    // we don't ensure every funclet has an extra here
    // sorta not our problem
    for (name, extra) in schedule_extras.into_iter() {
        result.insert(context.funclet_indices.get(&name).unwrap(), extra);
    }
    result
}

fn ir_program(program: &assembly_ast::Program, context: &mut Context) -> ir::Program {
    setup_extras(&program.extras, context);
    ir::Program {
        native_interface: ir_native_interface(&program, context),
        types: ir_types(&program.types, context),
        funclets: ir_funclets(&program.funclets, context),
        value_functions: ir_value_functions(&program.funclets, context),
        pipelines: ir_pipelines(&program.pipelines, context),
        value_funclet_extras: ir_value_extras(&program.funclets, &program.extras, context),
        scheduling_funclet_extras: ir_scheduling_extras(context),
    }
}

pub fn explicate(mut program: assembly_ast::Program) -> frontend::Definition {
    let mut context = Context::new(&program);
    frontend::Definition {
        version: ir_version(&program.version, &mut context),
        program: ir_program(&program, &mut context),
    }
}
