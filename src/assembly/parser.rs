use pest::iterators::{Pair, Pairs};
// uncomment to test raw parsing
// use pest::Parser;
// use pest_derive::Parser;
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
struct CaimanAssemblyParser;

use crate::{assembly, frontend, ir};
use assembly::ast;
use crate::explication::Hole;
use ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, MetaId, NodeId, RemoteNodeId,
    StorageTypeId, TypeId,
};
use ir::ffi;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct UserData {}

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

const PHI_QUALIFIER: &str = "_PHI_";

// helper stuff

fn unexpected(s: String) -> String {
    format!("Unexpected string {}", s)
}

fn error_hole(input: &Node) -> Error<Rule> {
    input.error("Invalid hole")
}

fn reject_hole<T>(h: Hole<T>, error: Error<Rule>) -> ParseResult<T> {
    match h {
        Hole::Filled(v) => Ok(v),
        Hole::Empty => Err(error),
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
        // make empty rather than unreachable cause it's an option
        Ok(())
    }
    fn hole(_input: Node) -> ParseResult<()> {
        // make empty rather than unreachable cause it's an option
        Ok(())
    }
    fn node_hole(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn function_class_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn phi_qualifier(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn encoder_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn return_sep(_input: Node) -> ParseResult<()> {
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
    fn schedule_yield_sep(_input: Node) -> ParseResult<()> {
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
    fn alloc_temporary_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn static_sub_alloc_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn static_split_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn static_merge_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn read_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn borrow_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn write_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn local_do_builtin_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn local_do_external_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn local_copy_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn begin_encoding_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn encode_do_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn encode_copy_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn submit_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn sync_fence_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn inline_join_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn serialized_join_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn promise_captures_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn fulfill_captures_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn encoding_event_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn submission_event_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn synchronization_event_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn separated_buffer_space_sep(_input: Node) -> ParseResult<()> {
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
    fn schedule_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn pipeline_effect_sep(_input: Node) -> ParseResult<()> {
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

    fn n_sep(input: Node) -> ParseResult<usize> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => n
        ))
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
            [n(n)] => Hole::Filled(n),
            [hole] => Hole::Empty
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

    fn function_class_name(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => s,
        ))
    }

    fn function_class_name_sep(input: Node) -> ParseResult<String> {
        Ok(match_nodes!(input.into_children();
            [function_class_name(name)] => name
        ))
    }

    fn name_hole(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => Hole::Filled(name),
            [hole] => Hole::Empty
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

    fn ffi_type_sep(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(f)] => f
        ))
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
            [typ(t)] => Hole::Filled(t),
            [hole] => Hole::Empty
        ))
    }

    fn type_hole_sep(input: Node) -> ParseResult<Hole<ast::TypeId>> {
        Ok(match_nodes!(input.into_children(); [type_hole(t)] => t))
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
                _ => unreachable!(),
            })
    }

    fn place_sep(input: Node) -> ParseResult<ir::Place> {
        Ok(match_nodes!(input.into_children(); [place(t)] => t))
    }

    fn place_hole(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children();
            [place(place)] => Hole::Filled(place),
            [hole] => Hole::Empty
        ))
    }

    fn place_hole_sep(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children(); [place_hole(t)] => t))
    }

    fn buffer_flag(input: Node) -> ParseResult<ir::BufferFlags> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "map_read" => Ok(ir::BufferFlags {
                    map_read: true,
                    ..Default::default()
                }),
                "map_write" => Ok(ir::BufferFlags {
                    map_write: true,
                    ..Default::default()
                }),
                "copy_src" => Ok(ir::BufferFlags {
                    copy_src: true,
                    ..Default::default()
                }),
                "copy_dst" => Ok(ir::BufferFlags {
                    copy_dst: true,
                    ..Default::default()
                }),
                "storage" => Ok(ir::BufferFlags {
                    storage: true,
                    ..Default::default()
                }),
                "uniform" => Ok(ir::BufferFlags {
                    uniform: true,
                    ..Default::default()
                }),
                _ => unimplemented!(),
            })
    }

    fn buffer_flags_elements(input: Node) -> ParseResult<ir::BufferFlags> {
        Ok(match_nodes!(input.into_children();
            [buffer_flag(flags)..] => flags.fold(ir::BufferFlags::new(), |flags, new_flags| (flags.or(& new_flags))),
        ))
    }

    fn buffer_flags(input: Node) -> ParseResult<ir::BufferFlags> {
        Ok(match_nodes!(input.into_children();
            [buffer_flags_elements(flags)] => flags
        ))
    }

    fn quotient(input: Node) -> ParseResult<ast::RemoteNodeId> {
        Ok(match_nodes!(input.into_children();
            [phi_qualifier, meta_name(funclet_id)] => {
                ast::RemoteNodeId {
                    funclet: Hole::Filled(ast::MetaId(funclet_id)),
                    node: None
                }
            },
            [meta_name(funclet_id)] => {
                ast::RemoteNodeId {
                    funclet: ast::MetaId(funclet_id),
                    node: None
                }
            },
            [phi_qualifier, meta_name(funclet_id), name(node_id)] => {
                ast::RemoteNodeId {
                    funclet: Hole::Filled(ast::MetaId(funclet_id)),
                    node: Some(Hole::Filled(ast::NodeId(PHI_QUALIFIER.to_owned() + &node_id)))
                }
            },
            [meta_name(funclet_id), name(node_id)] => {
                ast::RemoteNodeId {
                    funclet: ast::MetaId(funclet_id),
                    node: Some(Hole::Filled(ast::NodeId(node_id)))
                }
            },
        ))
    }

    fn quotient_hole(input: Node) -> ParseResult<Hole<ast::RemoteNodeId>> {
        Ok(match_nodes!(input.into_children();
            [phi_qualifier, meta_name(funclet_id)] => {
                Hole::Filled(ast::RemoteNodeId {
                    funclet: funclet_id.opt().map(ast::MetaId).into(),
                    node: None
                })
            },
            [meta_name(funclet_id)] => {
                Hole::Filled(ast::RemoteNodeId {
                    funclet: funclet_id.opt().map(ast::MetaId).into(),
                    node: None
                })
            },
            [phi_qualifier, meta_name(funclet_id), name_hole(node_id)] => {
                Hole::Filled(ast::RemoteNodeId {
                    funclet: funclet_id.opt().map(ast::MetaId).into(),
                    node: Some(node_id.opt().map(|n|
                        ast::NodeId(PHI_QUALIFIER.to_owned() + &n))
                    .into())
                })
            },
            [meta_name(funclet_id), name_hole(node_id)] => {
                Hole::Filled(ast::RemoteNodeId {
                    funclet: funclet_id.opt().map(ast::MetaId).into(),
                    node: Some(node_id.opt().map(ast::NodeId).into())
                })
            },
            [hole] => {
                Hole::Empty
            }
        ))
    }

    fn flow(input: Node) -> ParseResult<Hole<ir::Flow>> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "dead" => Ok(Some(ir::Flow::Dead)),
                "usable" => Ok(Some(ir::Flow::Usable)),
                "saved" => Ok(Some(ir::Flow::Saved)),
                "need" => Ok(Some(ir::Flow::Need)),
                "?" => Ok(None),
                _ => unreachable!(),
            })
    }

    fn tag(input: Node) -> ParseResult<ast::Tag> {
        Ok(match_nodes!(input.into_children();
            [quotient(quot), flow(flow)] => ast::Tag { quot: Hole::Filled(quot), flow }
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
            [effect(x)] => ast::Declaration::Effect(x),
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

    fn ref_buffer(input: Node) -> ParseResult<ir::BufferFlags> {
        Ok(match_nodes!(input.into_children();
            [buffer_flags(buffer_flags)] => buffer_flags
        ))
    }

    fn ref_type(input: Node) -> ParseResult<(ast::TypeId, ir::Place)> {
        Ok(match_nodes!(input.into_children();
            [typ(typ), place(place)] => (typ, place)
        ))
    }

    fn ref_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), ref_type((storage_type, storage_place)), ref_buffer(buffer_flags)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Ref {
                        storage_type,
                        storage_place,
                        buffer_flags
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
            [name(name), place(storage_place), buffer_flags(flags), n(alignment_bits), n(byte_size)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Buffer {
                        storage_place,
                        static_layout_opt: Some(ir::StaticBufferLayout {
                            alignment_bits,
                            byte_size
                        }),
                        flags
                    }
                })
        ))
    }

    fn buffer_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [buffer_alignment_decl(decl)] => decl
        ))
    }

    fn encoder_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [encoder_sep, name(name), place(queue_place)] => ast::TypeDecl::Local(ast::LocalType {
                name: name,
                data: ast::LocalTypeInfo::Encoder { queue_place }
            })
        ))
    }

    fn event_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => ast::TypeDecl::Local(ast::LocalType {
                name: name,
                data: ast::LocalTypeInfo::Event {}
            })
        ))
    }

    fn buffer_space_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => ast::TypeDecl::Local(ast::LocalType {
                name: name,
                data: ast::LocalTypeInfo::BufferSpace {}
            })
        ))
    }

    fn type_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [ffi_type_decl(t)] => t,
            [native_value_decl(t)] => t,
            [ref_decl(t)] => t,
            [fence_decl(t)] => t,
            [buffer_decl(t)] => t,
            [encoder_decl(t)] => t,
            [event_decl(t)] => t,
            [buffer_space_decl(t)] => t
        ))
    }

    fn name_elements(input: Node) -> ParseResult<Vec<NodeId>> {
        Ok(match_nodes!(input.into_children();
               [name(names)..] => names.map(|name| NodeId(name)).collect()
        ))
    }

    fn name_list(input: Node) -> ParseResult<Vec<NodeId>> {
        Ok(match_nodes!(input.into_children();
            [name_elements(names)] => names,
            [name(name)] => vec![NodeId(name)]
        ))
    }

    fn name_hole_elements(input: Node) -> ParseResult<Vec<Hole<NodeId>>> {
        Ok(match_nodes!(input.into_children();
               [name_hole(names)..] => names.map(|name| name.opt().map(|s| NodeId(s)).into()).collect()
        ))
    }

    fn name_box(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole_elements(lst)] => Hole::Filled(lst),
            [hole] => Hole::Empty
        ))
    }

    fn name_box_single(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_box(b)] => b,
            [name_hole(name)] => Hole::Filled(vec![name.opt().map(|s| NodeId(s)).into()])
        ))
    }

    fn name_call(input: Node) -> ParseResult<Hole<Vec<Hole<NodeId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole_elements(lst)] => Hole::Filled(lst),
            [hole] => Hole::Empty
        ))
    }

    fn assign(input: Node) -> ParseResult<ast::NodeId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => NodeId(name)
        ))
    }

    fn n_elements(input: Node) -> ParseResult<Vec<Hole<usize>>> {
        Ok(match_nodes!(input.into_children();
            [n(values)..] => values.map(|v| Hole::Filled(v)).collect(),
        ))
    }

    fn n_list(input: Node) -> ParseResult<Hole<Vec<Hole<usize>>>> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => Hole::Filled(vec![Hole::Filled(n)]),
            [n_elements(values)] => Hole::Filled(values)
        ))
    }

    fn impl_box(input: Node) -> ParseResult<ast::FunctionClassBinding> {
        Ok(match_nodes!(input.into_children();
            [impl_sep, function_class_name(name)] => ast::FunctionClassBinding {
                default: false,
                function_class: FunctionClassId(name)
            },
            [impl_sep, default, function_class_name(name)] => ast::FunctionClassBinding {
                default: true,
                function_class: FunctionClassId(name)
            }
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

    fn external_dimensionality(input: Node) -> ParseResult<usize> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => n
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
                external_dimensionality(dimensionality),
                external_resource(resources)..] =>
                Some(ast::ExternalGPUInfo {
                    shader_module,
                    entry_point,
                    dimensionality,
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
                impl_box(value_function_binding),
                name(name),
                external_args(input_args),
                external_ret(output_types),
                external_body(body)] => {
                    let kind_result = match loc {
                        (ir::Place::Cpu, true) => Ok(ast::ExternalFunctionKind::CPUPure),
                        (ir::Place::Cpu, false) => Ok(ast::ExternalFunctionKind::CPUEffect),
                        (ir::Place::Gpu, false) => {
                            reject_hole(body.into(), error.clone())
                            .map(|v| ast::ExternalFunctionKind::GPU(v))
                        }
                        _ => Err(error.clone()),
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
            [function_class_sep, function_class_name(name),
            function_class_args(input_types), function_class_ret(output_types)] =>
                Ok(ast::FunctionClass {
                    name: FunctionClassId(name),
                    input_types,
                    output_types
                })
        )
    }

    fn effect_args(input: Node) -> ParseResult<Vec<ast::ExternalFunctionId>> {
        Ok(match_nodes!(input.into_children();
            [name(args)..] => args.map(ast::ExternalFunctionId).collect()
        ))
    }

    fn effect(input: Node) -> ParseResult<ast::EffectDeclaration> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => ast::EffectDeclaration {
                name: ast::EffectId(name),
                effect: ast::Effect::Unrestricted
            },
            [effect_args(effectful_function_ids), name(name)] => ast::EffectDeclaration {
                name: ast::EffectId(name),
                effect: ast::Effect::FullyConnected {
                    effectful_function_ids
                }
            }
        ))
    }

    // Hole::Filled duplication, but it's annoying to fix...
    fn schedule_box_value(input: Node) -> ParseResult<(MetaId, FuncletId)> {
        // the type is a bit of a lie here, but it reflects the AST better
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] =>
            (MetaId(meta_name), FuncletId(name))
        ))
    }

    fn schedule_box_timeline(input: Node) -> ParseResult<(MetaId, FuncletId)> {
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] =>
            (MetaId(meta_name), FuncletId(name))
        ))
    }

    fn schedule_box_spatial(input: Node) -> ParseResult<(MetaId, FuncletId)> {
        Ok(match_nodes!(input.into_children();
            [value_sep, meta_name(meta_name), name(name)] =>
            (MetaId(meta_name), FuncletId(name))
        ))
    }

    fn schedule_box(input: Node) -> ParseResult<ast::MetaMapping> {
        Ok(match_nodes!(input.into_children();
            [schedule_box_value(value), schedule_box_timeline(timeline),
            schedule_box_spatial(spatial)] => ast::MetaMapping {
                value,
                timeline,
                spatial
            }
        ))
    }

    fn funclet(input: Node) -> ParseResult<ast::Funclet> {
        fn add_phi_nodes(mut funclet: ast::Funclet) -> ast::Funclet {
            let mut index = 0;
            for input in funclet.header.args.iter() {
                funclet.commands.insert(
                    index,
                    Hole::Filled(ast::Command::Node(ast::NamedNode {
                        name: input
                            .name
                            .as_ref()
                            .map(|n| ast::NodeId(PHI_QUALIFIER.to_owned() + &n.0)),
                        node: ast::Node::Phi {
                            index: Hole::Filled(index),
                        },
                    })),
                );
                index += 1;
            }
            funclet
        }
        Ok(match_nodes!(input.into_children();
            [value_funclet(funclet)] => add_phi_nodes(funclet),
            [timeline_funclet(funclet)] => add_phi_nodes(funclet),
            [spatial_sep, spatial_funclet(funclet)] => add_phi_nodes(funclet),
            [schedule_funclet(funclet)] => add_phi_nodes(funclet),
        ))
    }

    fn funclet_arg(input: Node) -> ParseResult<ast::FuncletArgument> {
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
                },
            [impl_box(binding), name(name), funclet_args(args), funclet_return(ret)] =>
                ast::FuncletHeader {
                    name: FuncletId(name),
                    args,
                    ret,
                    binding: ast::FuncletBinding::SpecBinding(binding)
                }
        ))
    }

    fn value_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [value_node(node)] => Hole::Filled(ast::Command::Node(node)),
            [tail_edge(tail_edge)] => Hole::Filled(ast::Command::TailEdge(tail_edge))
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
            [timeline_node(node)] => Hole::Filled(ast::Command::Node(node)),
            [tail_edge(tail_edge)] => Hole::Filled(ast::Command::TailEdge(tail_edge)),
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
            [spatial_node(node)] => Hole::Filled(ast::Command::Node(node)),
            [tail_edge(tail_edge)] => Hole::Filled(ast::Command::TailEdge(tail_edge)),
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
        Ok(match_nodes!(input.into_children();
            [schedule_box(meta_map), name(name), tag(itag),
            tag(otag), schedule_args(args), schedule_return(ret)] =>
                {
                    ast::FuncletHeader {
                        name: FuncletId(name),
                        args,
                        ret,
                        binding: ast::FuncletBinding::ScheduleBinding(ast::ScheduleBinding {
                            implicit_tags: (itag, otag),
                            meta_map
                        })
                    }
                }
        ))
    }

    fn schedule_command(input: Node) -> ParseResult<Hole<ast::Command>> {
        Ok(match_nodes!(input.into_children();
            [schedule_node(node)] => Hole::Filled(ast::Command::Node(node)),
            [tail_edge(tail_edge)] => Hole::Filled(ast::Command::TailEdge(tail_edge)),
            [node_hole] => Hole::Empty
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

    fn spec_mapping(input: Node) -> ParseResult<(Hole<Vec<Hole<ast::RemoteNodeId>>>)> {
        Ok(match_nodes!(input.into_children();
            [hole] => Hole::Empty,
            [quotient_hole(operations)..] => Hole::Filled(operations.collect())
        ))
    }

    fn debug_hole_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [name_list(inputs)] => ast::TailEdge::DebugHole {
                inputs
            }
        ))
    }

    fn return_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [return_sep, name_box_single(return_values)] => ast::TailEdge::Return {
                return_values
            }
        ))
    }

    fn jump_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [join_sep, name_hole(join), name_box_single(arguments)] =>
                ast::TailEdge::Jump {
                    join: join.opt().map(|s| NodeId(s)).into(),
                    arguments
                }
        ))
    }

    fn schedule_call_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_call_sep, name_hole(callee_funclet_id),
                spec_mapping(operations),
                name_call(callee_arguments), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleCall {
                    operations,
                    callee_funclet_id: callee_funclet_id.opt().map(|s| FuncletId(s)).into(),
                    callee_arguments,
                    continuation_join: continuation_join.opt().map(|s| NodeId(s)).into()
                }
        ))
    }

    fn schedule_select_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_call_sep, name_hole(condition), name_box(callee_funclet_ids),
                spec_mapping(operations),
                name_call(callee_arguments), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleSelect {
                    operations,
                    condition: condition.opt().map(|s| NodeId(s)).into(),
                    callee_funclet_ids: callee_funclet_ids.opt().map(
                        |v| v.into_iter().map(
                            |name| name.opt().map(
                                |s| FuncletId(s.0)
                            ).into()
                        ).collect()
                    ).into(),
                    callee_arguments,
                    continuation_join: continuation_join.opt().map(|s| NodeId(s)).into()
                }
        ))
    }

    fn schedule_yield_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_yield_sep, name_hole(external_function_id),
                spec_mapping(operations),
                name_call(yielded_nodes), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleCallYield {
                    operations,
                    external_function_id: external_function_id.opt().map(|s| ExternalFunctionId(s)).into(),
                    yielded_nodes,
                    continuation_join: continuation_join.opt().map(|s| NodeId(s)).into()
                }
        ))
    }

    fn tail_edge(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [debug_hole_node(t)] => t,
            [return_node(t)] => t,
            [jump_node(t)] => t,
            [schedule_call_node(t)] => t,
            [schedule_select_node(t)] => t,
            [schedule_yield_node(t)] => t,
        ))
    }

    fn constant_value(input: Node) -> ParseResult<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }

    fn constant_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), name_sep(type_id), constant_value(value)] => ast::NamedNode {
                name: Some(name),
                node: ast::Node::Constant {
                    value: Hole::Filled(value),
                    type_id: Hole::Filled(ast::TypeId::Local(type_id))
                }
            }
        ))
    }

    fn extract_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), extract_sep, name(node_id), n(index)] => ast::NamedNode {
                name: Some(name),
                node: ast::Node::ExtractResult {
                    node_id: Hole::Filled(NodeId(node_id)),
                    index: Hole::Filled(index)
                }
            }
        ))
    }

    fn call_node(input: Node) -> ParseResult<ast::NamedNode> {
        // will split apart later
        Ok(match_nodes!(input.into_children();
            [assign(name), call_sep, function_class_name(external_function_id),
                name_call(arguments)] => ast::NamedNode {
                name: Some(name),
                    node: ast::Node::CallFunctionClass {
                        function_id: Hole::Filled(FunctionClassId(external_function_id)),
                        arguments
                    }
                }
        ))
    }

    fn select_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), select_sep, name_sep(condition),
                name_sep(true_case), name(false_case)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::Select {
                        condition: Hole::Filled(NodeId(condition)),
                        true_case: Hole::Filled(NodeId(true_case)),
                        false_case: Hole::Filled(NodeId(false_case))
                    }
                }
        ))
    }

    fn encoding_event_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), encoding_event_sep, name_sep(local_past),
                name_box(remote_local_pasts)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::EncodingEvent {
                        local_past: Hole::Filled(NodeId(local_past)),
                        remote_local_pasts
                    }
                }
        ))
    }

    fn submission_event_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submission_event_sep, name(local_past)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::SubmissionEvent {
                        local_past: Hole::Filled(NodeId(local_past))
                    }
                }
        ))
    }

    fn synchronization_event_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submission_event_sep, name_sep(local_past),
                name(remote_local_past)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::SynchronizationEvent {
                        local_past: Hole::Filled(NodeId(local_past)),
                        remote_local_past: Hole::Filled(NodeId(remote_local_past))
                    }
                }
        ))
    }

    fn separated_buffer_space_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), separated_buffer_space_sep,
                n_sep(count), name(space)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::SeparatedBufferSpaces {
                        count: Hole::Filled(count),
                        space: Hole::Filled(NodeId(space))
                    }
                }
        ))
    }

    fn alloc_temporary_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), alloc_temporary_sep, place_hole_sep(place),
                buffer_flags(buffer_flags),
                type_hole(storage_type)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::AllocTemporary {
                        place,
                        buffer_flags: Hole::Filled(buffer_flags),
                        storage_type,
                    }
                }
        ))
    }

    fn drop_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [name_hole(node)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::Drop {
                        node: node.opt().map(|s| NodeId(s)).into()
                    }
                }
        ))
    }

    fn static_sub_alloc_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_sub_alloc_sep, place_hole_sep(place),
                type_hole_sep(storage_type), name_hole(node)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::StaticSubAlloc {
                        node: node.opt().map(|s| NodeId(s)).into(),
                        place,
                        storage_type
                    }
                }
        ))
    }

    fn static_split_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_split_sep, place_hole_sep(place),
                name_hole(node), n_list(sizes), quotient_hole(spatial_operation)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::StaticSplit {
                        node: node.opt().map(|s| NodeId(s)).into(),
                        place,
                        sizes,
                        spatial_operation
                    }
                }
        ))
    }

    fn static_merge_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_sub_alloc_sep, place_hole_sep(place),
                quotient_hole(spatial_operation), name_box(nodes)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::StaticMerge {
                        nodes,
                        place,
                        spatial_operation
                    }
                }
        ))
    }

    fn read_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), read_sep, ffi_type_sep(storage_type),
                name_hole(source)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::ReadRef {
                        source: source.opt().map(|s| NodeId(s)).into(),
                        storage_type: Hole::Filled(ast::TypeId::FFI(storage_type))
                    }
                }
        ))
    }

    fn borrow_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), borrow_sep, type_hole_sep(storage_type),
                name_hole(source)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::BorrowRef {
                        source: source.opt().map(|s| NodeId(s)).into(),
                        storage_type
                    }
                }
        ))
    }

    fn write_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [write_sep, type_hole_sep(storage_type),
                name_hole(source), name_hole(destination)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::WriteRef {
                        storage_type,
                        source: source.opt().map(|s| NodeId(s)).into(),
                        destination: destination.opt().map(|s| NodeId(s)).into()
                    }
                }
        ))
    }

    fn local_do_builtin_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [local_do_builtin_sep, quotient_hole(operation),
                name_call(inputs), name_box_single(outputs)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::LocalDoBuiltin {
                        operation,
                        inputs,
                        outputs
                    }
                }
        ))
    }

    fn local_do_external_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [local_do_external_sep, name_hole_sep(external_function_id),
                quotient_hole(operation), name_call(inputs),
                name_box_single(outputs)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::LocalDoExternal {
                        external_function_id: external_function_id.opt().map(|s| ExternalFunctionId(s)).into(),
                        operation,
                        inputs,
                        outputs
                    }
                }
        ))
    }

    fn local_copy_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [local_copy_sep, name_hole(input), name_hole(output)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::LocalCopy {
                        input: input.opt().map(|s| NodeId(s)).into(),
                        output: output.opt().map(|s| NodeId(s)).into()
                    }
                }
        ))
    }

    fn begin_encoding_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), begin_encoding_sep, place_hole_sep(place),
                quotient_hole(event), name_box(encoded),
                name_box_single(fences)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::BeginEncoding {
                        place,
                        event,
                        encoded,
                        fences
                    }
                }
        ))
    }

    fn encode_do_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [encode_do_sep, name_hole_sep(encoder), name_hole_sep(external_function_id),
                quotient_hole(operation), name_call(inputs),
                name_box_single(outputs)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::EncodeDoExternal {
                        encoder: encoder.opt().map(|s| NodeId(s)).into(),
                        external_function_id: external_function_id.opt().map(|s| ExternalFunctionId(s)).into(),
                        operation,
                        inputs,
                        outputs
                    }
                }
        ))
    }

    fn encode_copy_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [encode_copy_sep, name_hole_sep(encoder), name_hole(input),
                name_hole(output)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::EncodeCopy {
                        encoder: encoder.opt().map(|s| NodeId(s)).into(),
                        input: input.opt().map(|s| NodeId(s)).into(),
                        output: output.opt().map(|s| NodeId(s)).into()
                    }
                }
        ))
    }

    fn submit_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submit_sep, name_hole_sep(encoder),
                quotient_hole(event)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::Submit {
                        encoder: encoder.opt().map(|s| NodeId(s)).into(),
                        event
                    }
                }
        ))
    }

    fn sync_fence_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [sync_fence_sep, name_hole_sep(fence), quotient_hole(event)] => ast::NamedNode {
                    name: None,
                    node: ast::Node::SyncFence {
                        fence: fence.opt().map(|s| NodeId(s)).into(),
                        event
                    }
                }
        ))
    }

    fn inline_join_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), inline_join_sep, name_hole_sep(funclet), name_box(captures),
                name_hole(continuation)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::InlineJoin {
                        funclet: funclet.opt().map(|s| FuncletId(s)).into(),
                        captures,
                        continuation: continuation.opt().map(|s| NodeId(s)).into(),
                    }
                }
        ))
    }

    fn serialized_join_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), serialized_join_sep, name_hole_sep(funclet),
                name_box(captures), name_hole(continuation)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::SerializedJoin {
                        funclet: funclet.opt().map(|s| FuncletId(s)).into(),
                        captures,
                        continuation: continuation.opt().map(|s| NodeId(s)).into(),
                    }
                }
        ))
    }

    fn default_join_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::DefaultJoin{}
                }
        ))
    }

    fn promise_captures_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), promise_captures_sep, n(count),
                name_hole(continuation)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::PromiseCaptures {
                        count: Hole::Filled(count),
                        continuation: continuation.opt().map(|s| NodeId(s)).into()
                    }
                }
        ))
    }

    fn fulfill_captures_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [assign(name), fulfill_captures_sep, name_hole_sep(continuation),
                name_box(haves), name_box(needs)] => ast::NamedNode {
                    name: Some(name),
                    node: ast::Node::FulfillCaptures {
                        continuation: continuation.opt().map(|s| NodeId(s)).into(),
                        haves,
                        needs
                    }
                }
        ))
    }

    fn value_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [extract_node(n)] => n,
            [call_node(n)] => n,
            [constant_node(n)] => n,
            [select_node(n)] => n
        ))
    }

    fn timeline_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [extract_node(n)] => n,
            [call_node(n)] => n,
            [encoding_event_node(n)] => n,
            [submission_event_node(n)] => n,
            [synchronization_event_node(n)] => n

        ))
    }

    fn spatial_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [extract_node(n)] => n,
            [call_node(n)] => n,
            [separated_buffer_space_node(n)] => n
        ))
    }

    fn schedule_node(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
            [alloc_temporary_node(n)] => n,
            [drop_node(n)] => n,
            [static_sub_alloc_node(n)] => n,
            [static_split_node(n)] => n,
            [static_merge_node(n)] => n,
            [read_node(n)] => n,
            [borrow_node(n)] => n,
            [write_node(n)] => n,
            [local_do_builtin_node(n)] => n,
            [local_do_external_node(n)] => n,
            [local_copy_node(n)] => n,
            [begin_encoding_node(n)] => n,
            [encode_do_node(n)] => n,
            [encode_copy_node(n)] => n,
            [submit_node(n)] => n,
            [sync_fence_node(n)] => n,
            [inline_join_node(n)] => n,
            [serialized_join_node(n)] => n,
            [default_join_node(n)] => n,
            [promise_captures_node(n)] => n,
            [fulfill_captures_node(n)] => n
        ))
    }

    fn pipeline_effect(input: Node) -> ParseResult<ast::EffectId> {
        Ok(match_nodes!(input.into_children();
            [pipeline_effect_sep, name(name)] => ast::EffectId(name)
        ))
    }

    fn pipeline(input: Node) -> ParseResult<ast::Pipeline> {
        Ok(match_nodes!(input.into_children();
            [pipeline_sep, str(name), name(funclet)] => ast::Pipeline{
                name, funclet: FuncletId(funclet), effect: None
            },
            [pipeline_sep, str(name), name(funclet), pipeline_effect(effect)] =>
            ast::Pipeline{
                name, funclet: FuncletId(funclet), effect: Some(effect)
            }
        ))
    }

    fn program(input: Node) -> ParseResult<ast::Program> {
        Ok(match_nodes!(input.into_children();
            [version(version), declaration(declarations).., EOI] => ast::Program {
                path: "".to_string(),
                version,
                declarations: declarations.collect()
            }
        ))
    }
}

pub fn parse(path: &str, code: &str) -> ParseResult<ast::Program> {
    // necessary to have an empty user data for checking stuff
    let user_data = UserData {};
    // CaimanAssemblyParser::parse(Rule::program, code);
    let parsed = CaimanAssemblyParser::parse_with_userdata(Rule::program, code, user_data)?;
    let mut result = CaimanAssemblyParser::program(parsed.single()?);
    match &mut result {
        Ok(ref mut program) => {
            program.path = path.to_string();
        }
        _ => {}
    };
    result
}
