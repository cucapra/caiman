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
    ExternalFunctionId, FFIType, FuncletId, NodeId, RemoteNodeId, StorageTypeId, TypeId,
    ValueFunctionId,
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

// dumb hack
fn create_map<F, T, U>(f: &'static F) -> Box<dyn Fn(T) -> U>
where
    F: Fn(T) -> U,
{
    Box::new(move |x| f(x))
}

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
    fn EOI(_input: Node) -> ParseResult<()> {
        Ok(())
    }

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

    fn name_hole(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => Some(name),
            [hole] => None
        ))
    }

    fn meta_name(input: Node) -> ParseResult<String> {
        CaimanAssemblyParser::name(input)
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
        Ok(match_nodes!(input.into_children();
            [meta_name(funclet_name), name(node_name)] => ast::RemoteNodeId {
                funclet_name: Some(FuncletId(funclet_name)),
                node_name: Some(NodeId(node_name))
            }
        ))
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

    fn constant_raw(input: Node) -> ParseResult<ast::Node> {
        match_nodes!(input.into_children();
        [n, ffi_type(type_id)] => {
            n
                .as_str()
                .parse::<String>()
                .map_err(|e| n.error(e))
                .map(|num| ast::Node::Constant {
                    type_id: TypeId::FFI(type_id),
                    value: num,
                })
        })
    }

    fn constant(input: Node) -> ParseResult<Hole<ast::Node>> {
        Ok(match_nodes!(input.into_children();
            [constant_raw(n)] => Some(n),
            [hole] => None
        ))
    }

    // weirdly, this seems like the best way to do this with pest_consume for now?
    fn tag_op(input: Node) -> ParseResult<Box<dyn Fn(Hole<ast::RemoteNodeId>) -> ast::Tag>> {
        fn box_up<F>(f: &'static F) -> Box<dyn Fn(Hole<ast::RemoteNodeId>) -> ast::Tag>
        where
            F: Fn(Hole<ast::RemoteNodeId>) -> ast::Tag,
        {
            Box::new(move |x| f(x))
        }

        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "node" => Ok(create_map(&ast::Tag::Node)),
                "input" => Ok(create_map(&ast::Tag::Input)),
                "output" => Ok(create_map(&ast::Tag::Output)),
                "halt" => Ok(create_map(&ast::Tag::Halt)),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn tag(input: Node) -> ParseResult<ast::Tag> {
        Ok(match_nodes!(input.into_children();
            [none] => ast::Tag::None,
            [tag_op(op), funclet_loc_hole(loc)] => op(loc),
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
            [name_type_separator(name), place(storage_place), n(alignment_bits), n(byte_size)] =>
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
            [name_type_separator(name)] => ast::TypeDecl::Local(ast::LocalType {
                name,
                data: ast::LocalTypeInfo::SchedulingJoin {}
            })
        ))
    }

    fn buffer_space_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name)] => ast::TypeDecl::Local(ast::LocalType {
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

    fn impl_box(input: Node) -> ParseResult<FuncletId> {
        Ok(match_nodes!(input.into_children();
            [impl_sep, name(name)] => FuncletId(name)
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
                ast::ExternalArgument { name: Some(name), ffi_type },
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
            [external_loc(loc), impl_box(imp), name(name), external_args(input_args),
                external_ret(output_types), external_body(body)] => {
                    let kind_result = match loc {
                        (Cpu, true) => Ok(ast::ExternalFunctionKind::CPUPure),
                        (Cpu, false) => Ok(ast::ExternalFunctionKind::CPUEffect),
                        (Gpu, false) => {
                            reject_hole(body, error.clone())
                            .map(|v| ast::ExternalFunctionKind::GPU(v))
                        }
                        _ => Err(error.clone()),
                    };
                    kind_result.map(|kind| ast::ExternalFunction {
                        kind,
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
            [function_class_sep, name(name),
            function_class_args(input_types), function_class_ret(output_types)] =>
                Ok(ast::FunctionClass {
                    name,
                    input_types,
                    output_types
                })
        )
    }

    // some duplication, but it's annoying to fix...
    fn schedule_box_value(input: Node) -> ParseResult<Option<(String, String)>> {
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] => Some((meta_name, name)),
            [] => None
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
            [impl_box(imp), value_funclet(mut value)] => {
                value.header.binding = ast::FuncletBinding::ValueBinding(imp);
                Ok(value)
            },
            [schedule_box(schedule), mut schedule_funclet] => {
                let mut local_map = schedule_funclet.user_data().binding_info.clone();
                local_map.replace(Some(schedule));
                let mut result = CaimanAssemblyParser::schedule_funclet(schedule_funclet);
                local_map.replace(None);
                result
            },
            [timeline_funclet(funclet)] => Ok(funclet),
            [spatial_funclet(funclet)] => Ok(funclet),
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
            [funclet_header(header), schedule_command(commands)..] => ast::Funclet {
                kind: ir::FuncletKind::Spatial,
                header,
                commands: commands.collect(),
            }
        ))
    }

    fn node_list(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn node_box_raw(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn node_box(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn return_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn return_args(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn return_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn yield_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn yield_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn jump_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn jump_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_call_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_call_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_select_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_nodes(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_box_raw(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_box(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_select_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn dynamic_alloc_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_option_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_option_nodes(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_option_box_raw(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_option_box(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn dynamic_alloc_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_edge(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn phi_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn constant_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn extract_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn extract_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn call_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn call_args(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn call_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn select_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn select_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn value_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn alloc_temporary_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_args(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_params(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_call(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn create_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn drop_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn alloc_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn alloc_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_copy_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_copy_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn submit_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_fence_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_fence_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_fence_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn inline_join_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn inline_join_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn serialized_join_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn serialized_join_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn default_join_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn submission_node(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn timeline_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn spatial_node(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn pipeline_sep(input: Node) -> ParseResult<()> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn pipeline(input: Node) -> ParseResult<ast::Pipeline> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn program(input: Node) -> ParseResult<ast::Program> {
        Ok(match_nodes!(input.into_children();
            [version(version), declaration(declarations).., EOI(_)] => ast::Program {
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
