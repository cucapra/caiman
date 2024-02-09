use crate::assembly::ast;
use crate::assembly::context::Context;
use crate::assembly::context::LocationNames;
use crate::assembly::parser;
use crate::explication::expir;
use crate::explication::Hole;
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

pub fn ffi_to_ffi(value: ast::FFIType, context: &mut Context) -> ffi::Type {
    fn box_map(b: Box<[ast::FFIType]>, context: &mut Context) -> Box<[ffi::TypeId]> {
        b.iter()
            .map(|x| ffi::TypeId(context.ffi_type_id(x)))
            .collect()
    }
    fn type_id(element_type: Box<ast::FFIType>, context: &mut Context) -> ffi::TypeId {
        ffi::TypeId(context.ffi_type_id(element_type.as_ref()))
    }
    match value {
        ast::FFIType::F32 => ffi::Type::F32,
        ast::FFIType::F64 => ffi::Type::F64,
        ast::FFIType::U8 => ffi::Type::U8,
        ast::FFIType::U16 => ffi::Type::U16,
        ast::FFIType::U32 => ffi::Type::U32,
        ast::FFIType::U64 => ffi::Type::U64,
        ast::FFIType::USize => ffi::Type::USize,
        ast::FFIType::I8 => ffi::Type::I8,
        ast::FFIType::I16 => ffi::Type::I16,
        ast::FFIType::I32 => ffi::Type::I32,
        ast::FFIType::I64 => ffi::Type::I64,
        ast::FFIType::Array {
            element_type,
            length,
        } => ffi::Type::Array {
            element_type: type_id(element_type, context),
            length,
        },
        ast::FFIType::ErasedLengthArray(element_type) => ffi::Type::ErasedLengthArray {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::Struct {
            fields,
            byte_alignment,
            byte_size,
        } => todo!(),
        ast::FFIType::Tuple(element_types) => ffi::Type::Tuple {
            fields: box_map(element_types.into_boxed_slice(), context),
        },
        ast::FFIType::ConstRef(element_type) => ffi::Type::ConstRef {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::MutRef(element_type) => ffi::Type::MutRef {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::ConstSlice(element_type) => ffi::Type::ConstSlice {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::MutSlice(element_type) => ffi::Type::MutSlice {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::GpuBufferRef(element_type) => ffi::Type::GpuBufferRef {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::GpuBufferSlice(element_type) => ffi::Type::GpuBufferSlice {
            element_type: type_id(element_type, context),
        },
        ast::FFIType::GpuBufferAllocator => ffi::Type::GpuBufferAllocator,
        ast::FFIType::CpuBufferAllocator => ffi::Type::CpuBufferAllocator,
        ast::FFIType::CpuBufferRef(element_type) => ffi::Type::CpuBufferRef {
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

pub fn ir_quotient_node(quot: &ast::Quotient, context: &Context) -> Hole<expir::Quotient> {
    fn get_node(remote_id: &ast::RemoteNodeId, context: &Context) -> Hole<usize> {
        match (remote_id.funclet.as_ref(), remote_id.node.as_ref()) {
            (Some(funclet_id), Some(node_id)) => Some(context.remote_node_id(funclet_id, node_id)),
            _ => None,
        }
    }
    match quot {
        ast::Quotient::None => Some(expir::Quotient::None),
        ast::Quotient::Node(r) => r
            .as_ref()
            .and_then(|v| get_node(v, context).map(|n| expir::Quotient::Node { node_id: n })),
        ast::Quotient::Input(r) => r
            .as_ref()
            .and_then(|v| get_node(v, context).map(|n| expir::Quotient::Input { index: n })),
        ast::Quotient::Output(r) => r
            .as_ref()
            .and_then(|v| get_node(v, context).map(|n| expir::Quotient::Output { index: n })),
    }
}

fn ir_tag(tag: &Hole<ast::Tag>, context: &mut Context) -> Hole<expir::Tag> {
    tag.as_ref().map(|t|
        expir::Tag {
            quot: ir_quotient_node(&t.quot, context),
            flow: t.flow.clone(),
        }
    )
}

fn quotient_funclet(quot: &Hole<ast::Quotient>, context: &mut Context) -> Hole<ast::FuncletId> {
    quot.and_then(|q| match q {
        ast::Quotient::None => None,
        ast::Quotient::Node(r) => r.as_ref().cloned().map(|v| v.funclet),
        ast::Quotient::Input(r) => r.as_ref().cloned().map(|v| v.funclet),
        ast::Quotient::Output(r) => r.as_ref().cloned().map(|v| v.funclet),
    })
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

fn ir_type_decl(type_decl: &ast::TypeDecl, context: &mut Context) -> Option<expir::Type> {
    match type_decl {
        ast::TypeDecl::Local(typ) => {
            Some(match &typ.data {
                // only supported custom types atm
                ast::LocalTypeInfo::NativeValue { storage_type } => expir::Type::NativeValue {
                    storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                },
                ast::LocalTypeInfo::Ref {
                    storage_type,
                    storage_place,
                } => expir::Type::Ref {
                    storage_type: ffi::TypeId(context.loc_type_id(&storage_type)),
                    storage_place: storage_place.clone(),
                },
                ast::LocalTypeInfo::Fence { queue_place } => expir::Type::Fence {
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Buffer {
                    storage_place,
                    static_layout_opt,
                } => expir::Type::Buffer {
                    storage_place: storage_place.clone(),
                    static_layout_opt: static_layout_opt.clone(),
                },
                ast::LocalTypeInfo::Encoder { queue_place } => expir::Type::Encoder {
                    queue_place: queue_place.clone(),
                },
                ast::LocalTypeInfo::Event {} => expir::Type::Event {},
                ast::LocalTypeInfo::BufferSpace => expir::Type::BufferSpace,
            })
        }
        ast::TypeDecl::FFI(name) => None,
    }
}

fn ir_non_constant_node(node: &ast::Node, context: &Context) -> expir::Node {
    // macro
    todo!()
}

fn ir_node(node: &ast::Node, context: &Context) -> expir::Node {
    match &node {
        ast::Node::Constant { value, type_id } => {
            // we don't allow holes for constants
            let unwrapped_value = value.unwrap().clone();
            let unwrapped_type = type_id.unwrap().clone();
            let parsed_value = match &unwrapped_type {
                ast::TypeId::Local(name) => match context.native_type_map.get(name) {
                    None => panic!("{:?} must have a direct FFI storage type", type_id),
                    Some(t) => match t {
                        ast::FFIType::U64 => {
                            expir::Constant::U64(unwrapped_value.parse::<u64>().unwrap())
                        }
                        ast::FFIType::I32 => {
                            expir::Constant::I32(unwrapped_value.parse::<i32>().unwrap())
                        }
                        ast::FFIType::I64 => {
                            expir::Constant::I64(unwrapped_value.parse::<i64>().unwrap())
                        }
                        _ => panic!("Unsupported constant type {:?}", type_id),
                    },
                },
                ast::TypeId::FFI(_) => panic!("Cannot directly type a constant with an ffi type"),
            };
            expir::Node::Constant {
                value: parsed_value,
                type_id: context.loc_type_id(&unwrapped_type),
            }
        }
        _ => ir_non_constant_node(node, context),
    }
}

fn ir_tail_edge(tail: &ast::TailEdge, context: &mut Context) -> expir::TailEdge {
    match tail {
        ast::TailEdge::DebugHole { inputs } => expir::TailEdge::DebugHole {
            inputs: inputs.as_ref().map(|n| context.node_id(n).collect()),
        },
        ast::TailEdge::Return { return_values } => expir::TailEdge::Return {
            return_values: return_values.as_ref().map(|v| {
                v.iter()
                    .map(|n| n.as_ref().map(|id| context.node_id(id)))
                    .collect()
            }),
        },
        ast::TailEdge::Jump { join, arguments } => expir::TailEdge::Jump {
            join: context.funclet_indices.get_funclet(join.as_ref()),
            arguments: arguments
                .as_ref()
                .iter()
                .map(|n| context.node_id(n.as_ref()))
                .collect(),
        },
        ast::TailEdge::ScheduleCall {
            value_operation,
            timeline_operation,
            spatial_operation,
            callee_funclet_id,
            callee_arguments,
            continuation_join,
        } => expir::TailEdge::ScheduleCall {
            value_operation: ir_quotient_node(value_operation.as_ref(), context),
            timeline_operation: ir_quotient_node(timeline_operation.as_ref(), context),
            spatial_operation: ir_quotient_node(spatial_operation.as_ref(), context),
            callee_funclet_id: context
                .funclet_indices
                .get_funclet(&callee_funclet_id.as_ref().0)
                .clone(),
            callee_arguments: callee_arguments
                .as_ref()
                .iter()
                .map(|n| context.node_id(n.as_ref()))
                .collect(),
            continuation_join: context.node_id(continuation_join.as_ref()),
        },
        ast::TailEdge::ScheduleSelect {
            value_operation,
            timeline_operation,
            spatial_operation,
            condition,
            callee_funclet_ids,
            callee_arguments,
            continuation_join,
        } => expir::TailEdge::ScheduleSelect {
            value_operation: ir_quotient_node(reject_hole(value_operation.as_ref()), context),
            timeline_operation: ir_quotient_node(reject_hole(timeline_operation.as_ref()), context),
            spatial_operation: ir_quotient_node(reject_hole(spatial_operation.as_ref()), context),
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
        },
        ast::TailEdge::ScheduleCallYield {
            value_operation,
            timeline_operation,
            spatial_operation,
            external_function_id,
            yielded_nodes,
            continuation_join,
        } => expir::TailEdge::ScheduleCallYield {
            value_operation: ir_quotient_node(reject_hole(value_operation.as_ref()), context),
            timeline_operation: ir_quotient_node(reject_hole(timeline_operation.as_ref()), context),
            spatial_operation: ir_quotient_node(reject_hole(spatial_operation.as_ref()), context),
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
            continuation_join: context.node_id(reject_hole(continuation_join.as_ref())),
        },
    }
}

// updates the location in the context value funclet
fn ir_schedule_binding(
    funclet_header: &ast::FuncletHeader,
    implicit_tags: &Option<(ast::Tag, ast::Tag)>,
    value: &Option<ast::FuncletId>,
    timeline: &Option<ast::FuncletId>,
    spatial: &Option<ast::FuncletId>,
    context: &mut Context,
) -> expir::FuncletSpecBinding {
    #[derive(Debug)]
    struct TagSet {
        value: expir::Tag,
        spatial: expir::Tag,
        timeline: expir::Tag,
    }

    fn gen_tags(
        tags: &Vec<ast::Tag>,
        value: &Option<ast::FuncletId>,
        timeline: &Option<ast::FuncletId>,
        spatial: &Option<ast::FuncletId>,
        context: &mut Context,
    ) -> TagSet {
        let mut result = TagSet {
            value: Default::default(),
            spatial: Default::default(),
            timeline: Default::default(),
        };
        for tag in tags {
            let new_tag = ir_tag(tag, context);
            let data = quotient_funclet(&tag.quot, context);
            match data {
                None => {}
                Some(fnid) => {
                    if fnid == value.clone().unwrap_or(ast::FuncletId("".to_string())) {
                        result.value = new_tag;
                    } else if fnid == spatial.clone().unwrap_or(ast::FuncletId("".to_string())) {
                        result.spatial = new_tag;
                    } else if fnid == timeline.clone().unwrap_or(ast::FuncletId("".to_string())) {
                        result.timeline = new_tag;
                    } else {
                        panic!("Unknown tag funclet id {:?}", fnid);
                    }
                }
            };
        }
        result
    }

    // probably a better way to do this, but whatever
    let mut value_in = Default::default();
    let mut value_out = Default::default();
    let mut spatial_in = Default::default();
    let mut spatial_out = Default::default();
    let mut timeline_in = Default::default();
    let mut timeline_out = Default::default();

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
        value_tags: Vec<expir::Tag>,
        spatial_tags: Vec<expir::Tag>,
        timeline_tags: Vec<expir::Tag>,
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

    let mut implicit_in_tag = Default::default();
    let mut implicit_out_tag = Default::default();

    match implicit_tags {
        None => {}
        Some((in_tag, out_tag)) => {
            implicit_in_tag = ir_tag(in_tag, context);
            implicit_out_tag = ir_tag(out_tag, context);
        }
    }

    expir::FuncletSpecBinding::ScheduleExplicit {
        value: expir::FuncletSpec {
            funclet_id_opt: value
                .clone()
                .map(|f| context.funclet_indices.get_funclet(&f.0).unwrap()),
            input_tags: input_tags.value_tags.into_boxed_slice(),
            output_tags: output_tags.value_tags.into_boxed_slice(),
            implicit_in_tag: Default::default(),
            implicit_out_tag: Default::default(),
        },
        spatial: expir::FuncletSpec {
            funclet_id_opt: spatial
                .clone()
                .map(|f| context.funclet_indices.get_funclet(&f.0).unwrap()),
            input_tags: input_tags.spatial_tags.into_boxed_slice(),
            output_tags: output_tags.spatial_tags.into_boxed_slice(),
            implicit_in_tag: Default::default(),
            implicit_out_tag: Default::default(),
        },
        timeline: expir::FuncletSpec {
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
) -> expir::FuncletSpecBinding {
    match &funclet_header.binding {
        ast::FuncletBinding::None => expir::FuncletSpecBinding::None,
        ast::FuncletBinding::ValueBinding(ast::FunctionClassBinding {
            default,
            function_class,
        }) => {
            let value_function_id_opt =
                Some(context.function_classes.get(&function_class).unwrap());
            expir::FuncletSpecBinding::Value {
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

fn ir_funclet(funclet: &ast::Funclet, context: &mut Context) -> expir::Funclet {
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
        context.location.node_name = command.name.clone();
        match &command.command {
            ast::Command::Node(node) => {
                nodes.push(ir_node(&node, context));
            }
            ast::Command::TailEdge(tail) => {
                if tail_edge.is_some() {
                    panic!("More than one tail edge in {:?}", funclet.header.name);
                }
                tail_edge = Some(ir_tail_edge(tail, context));
            }
            _ => {
                unreachable!("Unresolved Hole {:?}", command);
            }
        }
    }

    expir::Funclet {
        kind: funclet.kind.clone(),
        spec_binding,
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
) -> expir::FunctionClass {
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
                    if binding.function_class == function.name {
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
                        context.funclet_indices.get_funclet(&f.name).unwrap(),
                    ));
                }
            }
            _ => {}
        }
    }

    expir::FunctionClass {
        name_opt: Some(function.name.0.clone()),
        input_types: input_types.into_boxed_slice(),
        output_types: output_types.into_boxed_slice(),
        default_funclet_id,
        external_function_ids,
    }
}

fn ir_pipeline(pipeline: &ast::Pipeline, context: &mut Context) -> expir::Pipeline {
    match context.funclet_indices.get_funclet(&pipeline.funclet.0) {
        Some(entry_funclet) => expir::Pipeline {
            name: pipeline.name.clone(),
            entry_funclet: entry_funclet.clone(),
            effect_id_opt: None,
        },
        None => panic!(
            "Unknown funclet name {} in pipeline {}",
            &pipeline.funclet.0, &pipeline.name
        ),
    }
}

fn ir_program(program: &ast::Program, context: &mut Context) -> expir::Program {
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

    expir::Program {
        native_interface,
        types,
        funclets,
        function_classes,
        pipelines,
    }
}

pub fn lower(mut program: ast::Program) -> frontend::ExplicationDefinition {
    // should probably handle errors with a result, future problem though
    give_names(&mut program);
    let mut context = Context::new(&program);
    frontend::ExplicationDefinition {
        version: ir_version(&program.version, &mut context),
        program: ir_program(&program, &mut context),
    }
}
