use pest::iterators::{Pair, Pairs};
use pest_consume::{match_nodes, Error, Parser};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
struct CaimanAssemblyParser;

use crate::{assembly, frontend, ir};
use assembly::ast;
use ast::Hole;
use ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use ir::ffi;

// data structs

#[derive(Clone, Debug)]
struct BindingParseInfo {
    pub value: Option<FuncletId>,
    pub timeline: Option<FuncletId>,
    pub spatial: Option<FuncletId>,
    pub meta_map: HashMap<String, FuncletId>,
}

#[derive(Clone, Debug)]
struct UserData {
    pub binding_info: RefCell<Option<BindingParseInfo>>,
}

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

// helper stuff

fn unexpected(s: String) -> String {
    format!("Unexpected string {}", s)
}

fn error_hole(input: &Node) -> Error<Rule> {
    input.error("Invalid hole")
}

fn reject_hole<T>(h: Hole<T>, error: Error<Rule>) -> ParseResult<T> {
    match h {
        Some(v) => Ok(v),
        None => Err(error),
    }
}

fn clean_string(input: Node) -> ParseResult<String> {
    // remove `'` and `"`
    input
        .as_str()
        .parse::<String>()
        .map_err(|e| input.error(e))
        .map(|s| (s[1..s.len() - 1]).to_string())
}

#[pest_consume::parser]
impl CaimanAssemblyParser {
    // dummy declarations
    // we make them unreachable to highlight they are never called
    // this is done so that if they _are_ called, they should be updated

    fn version_keyword(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn pure_keyword(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn default_keyword(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn impl_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn none(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn hole(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn node_hole(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn function_class_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn return_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn yield_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn jump_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn schedule_call_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn schedule_select_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn dynamic_alloc_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn extract_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn call_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn select_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn inline_join_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn serialized_join_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn value_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn timeline_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn spatial_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn pipeline_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    fn EOI(_input: Node) -> ParseResult<()> {
        unreachable!()
    }

    // real declarations

    fn id(input: Node) -> ParseResult<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }

    fn n(input: Node) -> ParseResult<usize> {
        input.as_str().parse::<usize>().map_err(|e| input.error(e))
    }

    fn str_single(input: Node) -> ParseResult<String> {
        clean_string(input)
    }

    fn str_double(input: Node) -> ParseResult<String> {
        clean_string(input)
    }

    fn str(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [str_single(s)] => s,
            [str_double(s)] => s
        ))
    }

    fn n_hole(input: Node) -> ParseResult<Hole<usize>> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => Some(n),
            [hole] => None
        ))
    }

    fn name(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => s,
            [n(n)] => n.to_string(),
            [throwaway] => "_".to_string()
        ))
    }

    fn name_sep(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => name
        ))
    }

    fn function_name(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => s,
        ))
    }

    fn function_name_sep(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [function_name(name)] => name
        ))
    }

    fn name_hole(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => Some(name),
            [hole] => None
        ))
    }

    fn name_hole_sep(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [name_hole(name)] => name
        ))
    }

    fn meta_name(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => s,
            [throwaway] => "_".to_string()
        ))
    }

    fn meta_name_sep(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [meta_name(name)] => name
        ))
    }

    fn throwaway(input: Node) -> ParseResult<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }

    fn funclet_loc(input: Node) -> ParseResult<RemoteNodeId> {
        Ok(match_nodes!(input.into_children();
            [name(funclet_name), name(node_name)] => ast::RemoteNodeId {
                funclet_name: Some(FuncletId(funclet_name)),
                node_name: Some(NodeId(node_name))
            }
        ))
    }

    fn funclet_loc_sep(input: Node) -> ParseResult<RemoteNodeId> {
        Ok(match_nodes!(input.into_children();
            [funclet_loc(t)] => t
        ))
    }

    fn funclet_loc_hole(input: Node) -> ParseResult<Hole<RemoteNodeId>> {
        Ok(match_nodes!(input.into_children();
            [name_hole(funclet_name), name_hole(node_name)] => Some(ast::RemoteNodeId {
                funclet_name: funclet_name.map(|s| FuncletId(s)),
                node_name: node_name.map(|s| NodeId(s))
            }),
            [hole] => None
        ))
    }

    fn meta_funclet_loc_inner(input: Node) -> ParseResult<RemoteNodeId> {
        let error = input.error("Unknown meta name");
        let meta_map = input
            .user_data()
            .binding_info
            .borrow()
            .clone()
            .unwrap()
            .meta_map;
        match_nodes!(input.into_children();
            [meta_name(meta_name), name(node_name)] =>
                match meta_map.get(&meta_name) {
                        Some(funclet_name) => Ok(ast::RemoteNodeId {
                            funclet_name: Some(funclet_name.clone()),
                            node_name: Some(NodeId(node_name))
                        }),
                        None => Err(error)
                }
        )
    }

    fn meta_funclet_loc(input: Node) -> ParseResult<RemoteNodeId> {
        Ok(match_nodes!(input.into_children();
            [funclet_loc(f)] => f,
            [meta_funclet_loc_inner(f)] => f
        ))
    }

    fn ffi_type_base(input: Node) -> ParseResult<ast::FFIType> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "f32" => Ok(ast::FFIType::F32),
                "f64" => Ok(ast::FFIType::F64),
                "u8" => Ok(ast::FFIType::U8),
                "u16" => Ok(ast::FFIType::U16),
                "u32" => Ok(ast::FFIType::U32),
                "u64" => Ok(ast::FFIType::U64),
                "i8" => Ok(ast::FFIType::I8),
                "i16" => Ok(ast::FFIType::I16),
                "i32" => Ok(ast::FFIType::I32),
                "i64" => Ok(ast::FFIType::I64),
                "usize" => Ok(ast::FFIType::USize),
                "gpu_buffer_allocator" => Ok(ast::FFIType::GpuBufferAllocator),
                "cpu_buffer_allocator" => Ok(ast::FFIType::CpuBufferAllocator),
                _ => Err(input.error(format!("Unknown type name {}", s))),
            })
    }

    fn ffi_array_parameters(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(element_type), n(length)] => ast::FFIType::Array {
                element_type: Box::new(element_type),
                length,
            }
        ))
    }

    fn ffi_parameterized_ref_name(
        input: Node,
    ) -> ParseResult<Box<dyn Fn(ast::FFIType) -> ast::FFIType>> {
        fn box_up<F>(f: &'static F) -> Box<dyn Fn(ast::FFIType) -> ast::FFIType>
        where
            F: Fn(Box<ast::FFIType>) -> ast::FFIType,
        {
            Box::new(move |x| f(Box::new(x)))
        }

        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "erased_length_array" => Ok(box_up(&ast::FFIType::ErasedLengthArray)),
                "const_ref" => Ok(box_up(&ast::FFIType::ConstRef)),
                "mut_ref" => Ok(box_up(&ast::FFIType::MutRef)),
                "const_slice" => Ok(box_up(&ast::FFIType::ConstSlice)),
                "mut_slice" => Ok(box_up(&ast::FFIType::MutSlice)),
                "gpu_buffer_ref" => Ok(box_up(&ast::FFIType::GpuBufferRef)),
                "gpu_buffer_slice" => Ok(box_up(&ast::FFIType::GpuBufferSlice)),
                "cpu_buffer_ref" => Ok(box_up(&ast::FFIType::CpuBufferRef)),
                _ => Err(input.error(format!("Unknown type name {}", s))),
            })
    }

    fn ffi_tuple_parameters(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(f)..] => ast::FFIType::Tuple(f.collect())
        ))
    }

    fn ffi_parameterized_ref(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_parameterized_ref_name(kind), ffi_type(value)] => kind(value)
        ))
    }

    fn ffi_parameterized_type(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_array_parameters(t)] => t,
            [ffi_parameterized_ref(t)] => t,
            [ffi_tuple_parameters(t)] => t
        ))
    }

    fn ffi_type(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_type_base(t)] => t,
            [ffi_parameterized_type(t)] => t
        ))
    }

    fn typ(input: Node) -> ParseResult<ast::TypeId> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(t)] => TypeId::FFI(t),
            [name(name)] => TypeId::Local(name),
        ))
    }

    fn typ_sep(input: Node) -> ParseResult<ast::TypeId> {
        Ok(match_nodes!(input.into_children(); [typ(t)] => t))
    }

    fn type_hole(input: Node) -> ParseResult<Hole<ast::TypeId>> {
        Ok(match_nodes!(input.into_children();
            [typ(t)] => Some(t),
            [hole] => None
        ))
    }

    fn place(input: Node) -> ParseResult<ir::Place> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "local" => Ok(ir::Place::Local),
                "cpu" => Ok(ir::Place::Cpu),
                "gpu" => Ok(ir::Place::Gpu),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn place_hole(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children();
            [place(place)] => Some(place),
            [hole] => None
        ))
    }

    fn stage(input: Node) -> ParseResult<ir::ResourceQueueStage> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "unbound" => Ok(ir::ResourceQueueStage::Unbound),
                "bound" => Ok(ir::ResourceQueueStage::Bound),
                "encoded" => Ok(ir::ResourceQueueStage::Encoded),
                "submitted" => Ok(ir::ResourceQueueStage::Submitted),
                "ready" => Ok(ir::ResourceQueueStage::Ready),
                "dead" => Ok(ir::ResourceQueueStage::Dead),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn stage_hole(input: Node) -> ParseResult<Hole<ir::ResourceQueueStage>> {
        Ok(match_nodes!(input.into_children();
            [stage(stage)] => Some(stage),
            [hole] => None
        ))
    }

    // weirdly, this seems like the best way to do this with pest_consume for now?
    fn tag_op(input: Node) -> ParseResult<Box<dyn Fn(ast::RemoteNodeId) -> ast::Tag>> {
        fn box_up<F>(f: &'static F) -> Box<dyn Fn(ast::RemoteNodeId) -> ast::Tag>
        where
            F: Fn(ast::RemoteNodeId) -> ast::Tag,
        {
            Box::new(move |x| f(x))
        }

        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "node" => Ok(box_up(&ast::Tag::Node)),
                "input" => Ok(box_up(&ast::Tag::Input)),
                "output" => Ok(box_up(&ast::Tag::Output)),
                "halt" => Ok(box_up(&ast::Tag::Halt)),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn tag(input: Node) -> ParseResult<ast::Tag> {
        Ok(match_nodes!(input.into_children();
            [none] => ast::Tag::None,
            [tag_op(op), meta_funclet_loc(loc)] => op(loc),
        ))
    }

    fn version(input: Node) -> ParseResult<ast::Version> {
        Ok(match_nodes!(input.into_children();
            [version_keyword, n(major), n(minor), n(detailed)] => ast::Version {
                major, minor, detailed
            }
        ))
    }

    fn declaration(input: Node) -> ParseResult<ast::Declaration> {
        Ok(match_nodes!(input.into_children();
            [type_decl(x)] => ast::Declaration::TypeDecl(x),
            [external_function(x)] => ast::Declaration::ExternalFunction(x),
            [funclet(x)] => ast::Declaration::Funclet(x),
            [function_class(x)] => ast::Declaration::FunctionClass(x),
            [pipeline(x)] => ast::Declaration::Pipeline(x),
        ))
    }

    fn ffi_type_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(t)] => ast::TypeDecl::FFI(t),
        ))
    }

    fn name_type_separator(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => name
        ))
    }

    fn native_value_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), typ(storage_type)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::NativeValue { storage_type }
                })
        ))
    }

    fn slot_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), typ(storage_type),
            stage(queue_stage), place(queue_place)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Slot {
                        storage_type,
                        queue_stage,
                        queue_place
                }
            })
        ))
    }

    fn fence_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), place(queue_place)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Fence { queue_place }
                })
        ))
    }

    fn buffer_alignment_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name(name), place(storage_place), n(alignment_bits), n(byte_size)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Buffer {
                        storage_place,
                        static_layout_opt: Some(ir::StaticBufferLayout {
                            alignment_bits,
                            byte_size
                        })
                    }
                })
        ))
    }

    fn buffer_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [buffer_alignment_decl(decl)] => decl
        ))
    }

    fn event_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), place(place)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Event { place }
            })
        ))
    }

    fn scheduling_join_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => ast::TypeDecl::Local(ast::LocalType {
                name,
                data: ast::LocalTypeInfo::SchedulingJoin {}
            })
        ))
    }

    fn buffer_space_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => ast::TypeDecl::Local(ast::LocalType {
                name,
                data: ast::LocalTypeInfo::BufferSpace {}
            })
        ))
    }

    fn type_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [ffi_type_decl(t)] => t,
            [native_value_decl(t)] => t,
            [slot_decl(t)] => t,
            [fence_decl(t)] => t,
            [buffer_decl(t)] => t,
            [event_decl(t)] => t,
            [scheduling_join_decl(t)] => t,
            [buffer_space_decl(t)] => t
        ))
    }

    fn impl_box(input: Node) -> ParseResult<(bool, FunctionClassId)> {
        Ok(match_nodes!(input.into_children();
            [impl_sep, function_name(name)] => (false, FunctionClassId(name)),
            [impl_sep, default, function_name(name)] => (true, FunctionClassId(name))
        ))
    }

    fn external_path(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [str(s)] => s
        ))
    }

    fn external_entry_point(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [str(s)] => s
        ))
    }

    fn external_group(input: Node) -> ParseResult<usize> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => n
        ))
    }

    fn external_binding(input: Node) -> ParseResult<usize> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => n
        ))
    }

    fn external_input(input: Node) -> ParseResult<NodeId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => NodeId(name)
        ))
    }

    fn external_output(input: Node) -> ParseResult<NodeId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => NodeId(name)
        ))
    }

    fn external_resource(input: Node) -> ParseResult<ast::ExternalGpuFunctionResourceBinding> {
        // this is dumb, but easier than cleaning it up
        Ok(match_nodes!(input.into_children();
            [external_group(group), external_binding(binding), external_input(input)] =>
                ast::ExternalGpuFunctionResourceBinding {
                    group,
                    binding,
                    input: Some(input),
                    output: None
                },
            [external_group(group), external_binding(binding), external_output(output)] =>
                ast::ExternalGpuFunctionResourceBinding {
                    group,
                    binding,
                    input: None,
                    output: Some(output)
                },
            [external_group(group), external_binding(binding), external_input(input), external_output(output)] =>
                ast::ExternalGpuFunctionResourceBinding {
                    group,
                    binding,
                    input: Some(input),
                    output: Some(output)
                }
        ))
    }

    fn external_body(input: Node) -> ParseResult<Option<ast::ExternalGPUInfo>> {
        Ok(match_nodes!(input.into_children();
            [external_path(shader_module),
                external_entry_point(entry_point),
                external_resource(resources)..] =>
                Some(ast::ExternalGPUInfo {
                    shader_module,
                    entry_point,
                    resource_bindings: resources.collect()
                }),
            [] => None
        ))
    }

    fn external_arg(input: Node) -> ParseResult<ast::ExternalArgument> {
        Ok(match_nodes!(input.into_children();
            [name(name), ffi_type(ffi_type)] =>
                ast::ExternalArgument { name: Some(NodeId(name)), ffi_type },
            [ffi_type(ffi_type)] => ast::ExternalArgument{ name: None, ffi_type }
        ))
    }

    fn external_args(input: Node) -> ParseResult<Vec<ast::ExternalArgument>> {
        Ok(match_nodes!(input.into_children();
            [external_arg(args)..] => args.collect()
        ))
    }

    fn external_loc(input: Node) -> ParseResult<(ir::Place, bool)> {
        Ok(match_nodes!(input.into_children();
            [place(place)] => (place, false),
            [place(place), pure_keyword] => (place, true)
        ))
    }

    fn external_ret(input: Node) -> ParseResult<Vec<ast::ExternalArgument>> {
        Ok(match_nodes!(input.into_children();
            [external_args(args)] => args,
            [ffi_type(ffi_type)] => vec![ast::ExternalArgument { name: None, ffi_type }]
        ))
    }

    fn external_function(input: Node) -> ParseResult<ast::ExternalFunction> {
        let error = input.error("Invalid external, missing information");
        match_nodes!(input.into_children();
            [external_loc(loc),
                impl_box((default, function_class)),
                name(name),
                external_args(input_args),
                external_ret(output_types),
                external_body(body)] => {
                    let kind_result = match loc {
                        (ir::Place::Cpu, true) => Ok(ast::ExternalFunctionKind::CPUPure),
                        (ir::Place::Cpu, false) => Ok(ast::ExternalFunctionKind::CPUEffect),
                        (ir::Place::Gpu, false) => {
                            reject_hole(body, error.clone())
                            .map(|v| ast::ExternalFunctionKind::GPU(v))
                        }
                        _ => Err(error.clone()),
                    };
                    let value_function_binding = ast::FunctionClassBinding {
                        default,
                        function_class
                    };
                    kind_result.map(|kind| ast::ExternalFunction {
                        kind,
                        value_function_binding,
                        name,
                        input_args,
                        output_types,
                    })
                }
        )
    }

    fn function_class_args(input: Node) -> ParseResult<Vec<ast::TypeId>> {
        Ok(match_nodes!(input.into_children();
            [typ(types)..] => types.collect()
        ))
    }

    fn function_class_ret(input: Node) -> ParseResult<Vec<ast::TypeId>> {
        Ok(match_nodes!(input.into_children();
            [function_class_args(args)] => args,
            [typ(typ)] => vec![typ]
        ))
    }

    fn function_class(input: Node) -> ParseResult<ast::FunctionClass> {
        match_nodes!(input.into_children();
            [function_class_sep, function_name(name),
            function_class_args(input_types), function_class_ret(output_types)] =>
                Ok(ast::FunctionClass {
                    name: FunctionClassId(name),
                    input_types,
                    output_types
                })
        )
    }

    // some duplication, but it's annoying to fix...
    fn schedule_box_value(input: Node) -> ParseResult<Option<(String, String)>> {
        // the type is a bit of a lie here, but it reflects the AST better
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] => Some((meta_name, name)),
        ))
    }

    fn schedule_box_timeline(input: Node) -> ParseResult<Option<(String, String)>> {
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] => Some((meta_name, name)),
            [] => None
        ))
    }

    fn schedule_box_spatial(input: Node) -> ParseResult<Option<(String, String)>> {
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] => Some((meta_name, name)),
            [] => None
        ))
    }

    fn schedule_box(input: Node) -> ParseResult<BindingParseInfo> {
        fn build_parse_info(
            val: Option<(String, String)>,
            time: Option<(String, String)>,
            space: Option<(String, String)>,
        ) -> BindingParseInfo {
            fn unpack_pair(
                meta_map: &mut HashMap<String, FuncletId>,
                pair: Option<(String, String)>,
            ) -> Option<FuncletId> {
                match pair {
                    None => None,
                    Some((meta_name, name)) => {
                        let fnid = FuncletId(name);
                        meta_map.insert(meta_name, fnid.clone());
                        Some(fnid)
                    }
                }
            }
            let mut meta_map = HashMap::new();
            let value = unpack_pair(&mut meta_map, val);
            let timeline = unpack_pair(&mut meta_map, time);
            let spatial = unpack_pair(&mut meta_map, space);
            BindingParseInfo {
                value,
                timeline,
                spatial,
                meta_map,
            }
        }
        Ok(match_nodes!(input.into_children();
            [schedule_box_value(val), schedule_box_timeline(time),
            schedule_box_spatial(space)] => build_parse_info(val, time, space)
        ))
    }

    fn funclet(input: Node) -> ParseResult<ast::Funclet> {
        match_nodes!(input.into_children();
            [impl_box((default, function_class)), value_funclet(mut value)] => {
                value.header.binding = ast::FuncletBinding::ValueBinding(
                    ast::FunctionClassBinding {
                        default,
                        function_class
                });
                Ok(value)
            },
            [schedule_box(schedule), mut schedule_funclet] => {
                *schedule_funclet.user_data().binding_info.borrow_mut() = Some(schedule);
                let mut result = CaimanAssemblyParser::schedule_funclet(schedule_funclet);
                result
            },
            [timeline_sep, timeline_funclet(funclet)] => Ok(funclet),
            [spatial_sep, spatial_funclet(funclet)] => Ok(funclet),
        )
    }

    fn funclet_arg(input: Node) -> ParseResult<ast::FuncletArgument> {
        let error = error_hole(&input);
        Ok(match_nodes!(input.into_children();
            [name(name), typ(typ)] =>  ast::FuncletArgument {
                    name: Some(NodeId(name)),
                    typ,
                    tags: vec![]
                },
            [typ(typ)] => ast::FuncletArgument {
                    name: None,
                    typ,
                    tags: vec![]
            },
        ))
    }

    fn funclet_args(input: Node) -> ParseResult<Vec<ast::FuncletArgument>> {
        Ok(match_nodes!(input.into_children();
            [funclet_arg(args)..] => args.collect()
        ))
    }

    fn funclet_return(input: Node) -> ParseResult<Vec<ast::FuncletArgument>> {
        Ok(match_nodes!(input.into_children();
            [funclet_args(args)] => args,
            [typ(typ)] => vec![ast::FuncletArgument {
                    name: None, typ, tags: vec![]
                }]
        ))
    }

    fn funclet_header(input: Node) -> ParseResult<ast::FuncletHeader> {
        Ok(match_nodes!(input.into_children();
            [name(name), funclet_args(args), funclet_return(ret)] =>
                ast::FuncletHeader {
                    name: FuncletId(name),
                    args,
                    ret,
                    binding: ast::FuncletBinding::None
                }
        ))
    }

    fn value_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [name(name), value_node(node)] => Some(ast::Command::Node(ast::NamedNode {
                name: NodeId(name),
                node
            })),
            [tail_edge(tail_edge)] => Some(ast::Command::TailEdge(tail_edge))
        ))
    }

    fn value_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
            [funclet_header(header), value_command(commands)..] => ast::Funclet {
                kind: ir::FuncletKind::Value,
                header,
                commands: commands.collect(),
            }
        ))
    }

    fn timeline_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [name(name), timeline_node(node)] => Some(ast::Command::Node(ast::NamedNode {
                name: NodeId(name),
                node
            })),
            [tail_edge(tail_edge)] => Some(ast::Command::TailEdge(tail_edge)),
            [node_hole] => None
        ))
    }

    fn timeline_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
            [funclet_header(header), timeline_command(commands)..] => ast::Funclet {
                kind: ir::FuncletKind::Timeline,
                header,
                commands: commands.collect(),
            }
        ))
    }

    fn spatial_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [name(name), spatial_node(node)] => Some(ast::Command::Node(ast::NamedNode {
                name: NodeId(name),
                node
            })),
            [tail_edge(tail_edge)] => Some(ast::Command::TailEdge(tail_edge)),
            [node_hole] => None
        ))
    }

    fn spatial_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
            [funclet_header(header), spatial_command(commands)..] => ast::Funclet {
                kind: ir::FuncletKind::Spatial,
                header,
                commands: commands.collect(),
            }
        ))
    }

    fn schedule_typ(input: Node) -> ParseResult<(Vec<ast::Tag>, ast::TypeId)> {
        Ok(match_nodes!(input.into_children();
            [tag(tags).., typ(typ)] => (tags.collect(), typ)
        ))
    }

    fn schedule_arg(input: Node) -> ParseResult<ast::FuncletArgument> {
        Ok(match_nodes!(input.into_children();
            [name(name), schedule_typ(info)] => ast::FuncletArgument {
                name: Some(NodeId(name)),
                typ: info.1,
                tags: info.0
            },
            [schedule_typ(info)] => ast::FuncletArgument {
                name: None,
                typ: info.1,
                tags: info.0
            }
        ))
    }

    fn schedule_args(input: Node) -> ParseResult<Vec<ast::FuncletArgument>> {
        Ok(match_nodes!(input.into_children();
            [schedule_arg(args)..] => args.collect()
        ))
    }

    fn schedule_return(input: Node) -> ParseResult<Vec<ast::FuncletArgument>> {
        Ok(match_nodes!(input.into_children();
            [schedule_args(args)] => args,
            [schedule_typ(info)] => vec![ast::FuncletArgument {
                name: None,
                typ: info.1,
                tags: info.0
            }]
        ))
    }

    fn schedule_header(input: Node) -> ParseResult<ast::FuncletHeader> {
        // requires that UserData be setup properly
        // unwrap with a panic cause this is an internal error if it happens
        let borrow = input.user_data().binding_info.borrow().clone();
        let binding_info = borrow.as_ref().unwrap();
        let value = binding_info.value.clone();
        let timeline = binding_info.timeline.clone();
        let spatial = binding_info.spatial.clone();
        Ok(match_nodes!(input.into_children();
            [name(name), tag(itag), tag(otag), schedule_args(args), schedule_return(ret)] =>
                {
                    ast::FuncletHeader {
                        name: FuncletId(name),
                        args,
                        ret,
                        binding: ast::FuncletBinding::ScheduleBinding(ast::ScheduleBinding {
                            implicit_tags: Some((itag, otag)),
                            value,
                            timeline,
                            spatial
                        })
                    }
                }
        ))
    }

    fn schedule_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [name(name), schedule_node(node)] => Some(ast::Command::Node(ast::NamedNode {
                name: NodeId(name),
                node
            })),
            [tail_edge(tail_edge)] => Some(ast::Command::TailEdge(tail_edge)),
            [node_hole] => None
        ))
    }

    fn schedule_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
            [schedule_header(header), schedule_command(commands)..] => ast::Funclet {
                kind: ir::FuncletKind::ScheduleExplicit,
                header,
                commands: commands.collect(),
            }
        ))
    }

    fn node_list(input: Node) -> ParseResult<Vec<Hole<NodeId>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole(names)..] => names.map(|name| name.map(|s| NodeId(s))).collect()
        ))
    }

    fn node_box(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [node_list(lst)] => Some(lst),
            [hole] => None
        ))
    }

    fn node_call(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [node_list(lst)] => Some(lst),
            [hole] => None
        ))
    }

    fn return_args(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [node_box(names)] => names,
            [name_hole(name)] => name.map(|s| vec![Some(NodeId(s))])
        ))
    }

    fn return_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [return_sep, return_args(return_values)] => ast::TailEdge::Return {
                return_values
            }
        ))
    }

    fn yield_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [yield_sep, name_hole(pipeline_yield_point), node_box(yielded_nodes),
                name_hole_sep(next_funclet), name_hole(continuation_join), node_box(arguments)] =>
                ast::TailEdge::Yield {
                    external_function_id: pipeline_yield_point.map(|s| ExternalFunctionId(s)),
                    yielded_nodes,
                    next_funclet: next_funclet.map(|s| FuncletId(s)),
                    continuation_join: continuation_join.map(|s| NodeId(s)),
                    arguments,
                }
        ))
    }

    fn jump_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [join_sep, name_hole(join), node_box(arguments)] =>
                ast::TailEdge::Jump {
                    join: join.map(|s| FuncletId(s)),
                    arguments
                }
        ))
    }

    fn schedule_call_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [join_sep, funclet_loc_hole(value_operation), name_hole(callee_funclet_id),
                node_box(callee_arguments), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleCall {
                    value_operation,
                    callee_funclet_id: callee_funclet_id.map(|s| FuncletId(s)),
                    callee_arguments,
                    continuation_join: continuation_join.map(|s| NodeId(s))
                }
        ))
    }

    fn schedule_select_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_select_sep,
                funclet_loc_hole(value_operation),
                name_hole(condition),
                node_box(callee_funclet_ids),
                node_box(callee_arguments),
                name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleSelect {
                    value_operation,
                    condition: condition.map(|s| NodeId(s)),
                    callee_funclet_ids: callee_funclet_ids.map(|ids| ids.iter().map(|id|
                        id.as_ref().map(|s| FuncletId(s.0.clone()))).collect()),
                    callee_arguments,
                    continuation_join: continuation_join.map(|s| NodeId(s))
                }
        ))
    }

    fn dynamic_alloc_size_slot(input: Node) -> ParseResult<Hole<Option<NodeId>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole(name)] => name.map(|s| Some(NodeId(s))),
            [none] => Some(None)
        ))
    }

    fn dynamic_alloc_size_slot_list(input: Node) -> ParseResult<Hole<Vec<Hole<Option<NodeId>>>>> {
        Ok(match_nodes!(input.into_children();
            [dynamic_alloc_size_slot(slots)..] => Some(slots.collect()),
            [hole] => None
        ))
    }

    fn dynamic_alloc_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [dynamic_alloc_sep,
                name_hole(buffer),
                node_box(arguments),
                dynamic_alloc_size_slot_list(dynamic_allocation_size_slots),
                name_hole_sep(success_funclet_id),
                name_hole_sep(failure_funclet_id),
                name_hole(continuation_join)] =>
                ast::TailEdge::DynamicAllocFromBuffer {
                    buffer: buffer.map(|s| NodeId(s)),
                    arguments,
                    dynamic_allocation_size_slots,
                    success_funclet_id: success_funclet_id.map(|s| FuncletId(s)),
                    failure_funclet_id: failure_funclet_id.map(|s| FuncletId(s)),
                    continuation_join: continuation_join.map(|s| NodeId(s))
                }
        ))
    }

    fn tail_edge(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [return_node(t)] => t,
            [yield_node(t)] => t,
            [jump_node(t)] => t,
            [schedule_call_node(t)] => t,
            [schedule_select_node(t)] => t,
            [dynamic_alloc_node(t)] => t,
        ))
    }

    fn phi_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [n_hole(index)] => ast::Node::Phi { index }
        ))
    }

    fn constant_node(input: Node) -> ParseResult<ast::Node> {
        match_nodes!(input.into_children();
            [n, ffi_type(type_id)] =>
                n.as_str()
                .parse::<String>()
                .map_err(|e| n.error(e))
                .map(|value| ast::Node::Constant {
                    value: Some(value),
                    type_id: Some(ast::TypeId::FFI(type_id))
                })
        )
    }

    fn extract_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [extract_sep, name(node_id), n(index)] => ast::Node::ExtractResult {
                node_id: Some(NodeId(node_id)),
                index: Some(index)
        }))
    }

    fn call_node(input: Node) -> ParseResult<ast::Node> {
        // will split apart later
        Ok(match_nodes!(input.into_children();
            [call_sep, function_name(external_function_id), node_call(arguments)] =>
                ast::Node::CallValueFunction {
                    function_id: Some(FunctionClassId(external_function_id)),
                    arguments
        }))
    }

    fn select_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [select_sep, name_sep(condition),
                name_sep(true_case), name(false_case)] => ast::Node::Select {
                condition: Some(NodeId(condition)),
                true_case: Some(NodeId(true_case)),
                false_case: Some(NodeId(false_case))
        }))
    }

    fn alloc_temporary_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place), type_hole(storage_type),
                funclet_loc_hole(operation)] => ast::Node::AllocTemporary {
                place,
                storage_type,
                operation
        }))
    }

    fn unbound_slot_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place), type_hole(storage_type),
                funclet_loc_hole(operation)] => ast::Node::UnboundSlot {
                place,
                storage_type,
                operation
        }))
    }

    fn drop_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [name_hole(node)] => ast::Node::Drop {
                node: node.map(|s| NodeId(s))
        }))
    }

    fn alloc_sep(input: Node) -> ParseResult<(Hole<ir::Place>, Hole<TypeId>)> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place), type_hole(typ)] => (place, typ)
        ))
    }

    fn alloc_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [alloc_sep((place, storage_type)), name_hole(buffer),
                funclet_loc_hole(operation)] => ast::Node::StaticAllocFromStaticBuffer {
                buffer: buffer.map(|s| NodeId(s)),
                place,
                storage_type,
                operation
        }))
    }

    fn encode_do_sep(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place)] => place
        ))
    }

    fn encode_do_call(input: Node) -> ParseResult<(Hole<RemoteNodeId>, Hole<Vec<Hole<NodeId>>>)> {
        Ok(match_nodes!(input.into_children();
            [funclet_loc_hole(fnloc), node_call(args)] => (fnloc, args),
            [hole] => (None, None)
        ))
    }

    fn encode_do_ret(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [node_box(nodes)] => nodes,
            [name(name)] => Some(vec![Some(NodeId(name))])
        ))
    }

    fn encode_do_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [encode_do_sep(place), encode_do_call((operation, inputs)),
                encode_do_ret(outputs)] => ast::Node::EncodeDo {
                place,
                operation,
                inputs,
                outputs
        }))
    }

    fn encode_copy_sep(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place)] => place
        ))
    }

    fn encode_copy_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [encode_copy_sep(place), name_hole_sep(input),
                name_hole(output)] => ast::Node::EncodeCopy {
                place,
                input: input.map(|s| NodeId(s)),
                output: output.map(|s| NodeId(s))
        }))
    }

    fn submit_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place), funclet_loc_hole(event)] => ast::Node::Submit {
                place,
                event
        }))
    }

    fn encode_fence_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place), funclet_loc_hole(event)] => ast::Node::EncodeFence {
                place,
                event
        }))
    }

    fn sync_fence_sep(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children();
            [place_hole(place)] => place
        ))
    }

    fn sync_fence_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [sync_fence_sep(place), name_hole_sep(fence),
                funclet_loc_hole(event)] => ast::Node::SyncFence {
                place,
                fence: fence.map(|s| NodeId(s)),
                event
        }))
    }

    fn inline_join_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [inline_join_sep, name_hole(funclet), node_box(captures),
                name_hole(continuation)] => ast::Node::InlineJoin {
                funclet: funclet.map(|s| FuncletId(s)),
                captures,
                continuation: continuation.map(|s| NodeId(s))
        }))
    }

    fn serialized_join_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [serialized_join_sep, name_hole(funclet), node_box(captures),
                name_hole(continuation)] => ast::Node::SerializedJoin {
                funclet: funclet.map(|s| FuncletId(s)),
                captures,
                continuation: continuation.map(|s| NodeId(s))
        }))
    }

    fn default_join_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [] => ast::Node::DefaultJoin {}
        ))
    }

    fn submission_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [place_hole(here_place), place_hole(there_place),
                name_hole(local_past)] => ast::Node::SubmissionEvent {
                here_place,
                there_place,
                local_past: local_past.map(|s| NodeId(s))
        }))
    }

    fn synchronization_sep(input: Node) -> ParseResult<(Hole<ir::Place>, Hole<ir::Place>)> {
        Ok(match_nodes!(input.into_children();
            [place_hole(here_place), place_hole(there_place)] => (here_place, there_place)
        ))
    }

    fn synchronization_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [synchronization_sep((here_place, there_place)),
                name_hole_sep(local_past), name_hole(remote_local_past)]
                => ast::Node::SynchronizationEvent {
                here_place,
                there_place,
                local_past: local_past.map(|s| NodeId(s)),
                remote_local_past: remote_local_past.map(|s| NodeId(s))
        }))
    }

    fn separated_linear_space_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        [place_hole(place), name_hole(space)] => ast::Node::SeparatedLinearSpace {
            place,
            space: space.map(|s| NodeId(s))
        }))
    }

    fn merged_linear_space_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        [place_hole(place), node_box(spaces)] => ast::Node::MergedLinearSpace {
            place,
            spaces
        }))
    }

    fn value_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [phi_node(n)] => n,
            [constant_node(n)] => n,
            [extract_node(n)] => n,
            [call_node(n)] => n,
            [select_node(n)] => n
        ))
    }

    fn schedule_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [phi_node(n)] => n,
            [alloc_temporary_node(n)] => n,
            [unbound_slot_node(n)] => n,
            [encode_do_node(n)] => n,
            [drop_node(n)] => n,
            [alloc_node(n)] => n,
            [encode_copy_node(n)] => n,
            [submit_node(n)] => n,
            [encode_fence_node(n)] => n,
            [sync_fence_node(n)] => n,
            [inline_join_node(n)] => n,
            [serialized_join_node(n)] => n,
            [default_join_node(n)] => n,
        ))
    }

    fn timeline_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [phi_node(n)] => n,
            [synchronization_node(n)] => n,
            [submission_node(n)] => n,
        ))
    }

    fn spatial_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
            [phi_node(n)] => n,
            [separated_linear_space_node(n)] => n,
            [merged_linear_space_node(n)] => n,
        ))
    }

    fn pipeline(input: Node) -> ParseResult<ast::Pipeline> {
        Ok(match_nodes!(input.into_children();
            [pipeline_sep, str(name), name(funclet)] => ast::Pipeline{
                name, funclet: FuncletId(funclet)
            }
        ))
    }

    fn program(input: Node) -> ParseResult<ast::Program> {
        Ok(match_nodes!(input.into_children();
            [version(version), declaration(declarations).., EOI] => ast::Program {
                version,
                declarations: declarations.collect()
            }
        ))
    }
}

pub fn parse(code: &str) -> ParseResult<ast::Program> {
    // necessary to have an empty user data for checking stuff
    let user_data = UserData {
        binding_info: RefCell::new(None),
    };
    let parsed = CaimanAssemblyParser::parse_with_userdata(Rule::program, code, user_data)?;
    CaimanAssemblyParser::program(parsed.single()?)
}
