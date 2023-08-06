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
use ast::Hole;
use ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, CommandId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use ir::ffi;
use std::cell::RefCell;
use std::collections::HashMap;

// data structs

#[derive(Clone, Debug)]
struct ParseBindingInfo {
    pub meta_map: HashMap<String, FuncletId>,
}

#[derive(Clone, Debug)]
struct UserData {
    pub binding_info: RefCell<Option<ParseBindingInfo>>,
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
    fn static_alloc_sep(_input: Node) -> ParseResult<()> {
        unreachable!()
    }
    fn static_dealloc_sep(_input: Node) -> ParseResult<()> {
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

    fn meta_name_hole(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [meta_name(name)] => Some(name),
            [hole] => None
        ))
    }

    fn meta_name_hole_sep(input: Node) -> ParseResult<Hole<String>> {
        Ok(match_nodes!(input.into_children();
            [meta_name_hole(name)] => name
        ))
    }

    fn throwaway(input: Node) -> ParseResult<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }

    fn meta_remote(input: Node) -> ParseResult<RemoteNodeId> {
        let error = input.error("Unknown meta name");
        let meta_map = input
            .user_data()
            .binding_info
            .borrow()
            .clone()
            .unwrap()
            .meta_map;
        match_nodes!(input.into_children();
            [meta_name(meta_name), name(node)] =>
                match meta_map.get(&meta_name) {
                        Some(funclet) => Ok(ast::RemoteNodeId {
                            funclet: Some(funclet.clone()),
                            node: Some(CommandId(node))
                        }),
                        None => Err(error)
                }
        )
    }

    fn meta_remote_hole(input: Node) -> ParseResult<RemoteNodeId> {
        let error = input.error("Unknown meta name");
        let meta_map = input
            .user_data()
            .binding_info
            .borrow()
            .clone()
            .unwrap()
            .meta_map;
        match_nodes!(input.into_children();
            // there's a way to make this pretty, but I'm stupid
            [meta_name_hole(meta_name_hole), name_hole(name_hole)] =>
                match meta_name_hole {
                    None => Ok(ast::RemoteNodeId {
                                funclet: None,
                                node: name_hole.map(|s| CommandId(s))
                            }),
                    Some(meta_name) =>
                        match meta_map.get(&meta_name) {
                                Some(funclet) => Ok(ast::RemoteNodeId {
                                    funclet: Some(funclet.clone()),
                                    node: name_hole.map(|s| CommandId(s))
                                }),
                                None => Err(error)
                        }
                },
            [hole] => Ok(ast::RemoteNodeId {
                funclet: None,
                node: None
            })
        )
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

    fn ffi_type_hole_sep(input: Node) -> ParseResult<Hole<ast::FFIType>> {
        Ok(match_nodes!(input.into_children();
            [ffi_type_hole(f)] => f
        ))
    }

    fn ffi_array_parameters(input: Node) -> ParseResult<ast::FFIType> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(element_type)] => ast::FFIType::ErasedLengthArray(
                Box::new(element_type)
            ),
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

    fn ffi_type_hole(input: Node) -> ParseResult<Hole<ast::FFIType>> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(f)] => Some(f),
            [hole] => None
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
            [place(place)] => Some(place),
            [hole] => None
        ))
    }

    fn place_hole_sep(input: Node) -> ParseResult<Hole<ir::Place>> {
        Ok(match_nodes!(input.into_children(); [place_hole(t)] => t))
    }

    // weirdly, this seems like the best way to do this with pest_consume for now?
    fn quotient_name(
        input: Node,
    ) -> ParseResult<Box<dyn Fn(Hole<ast::RemoteNodeId>) -> ast::Quotient>> {
        fn box_up<F>(f: &'static F) -> Box<dyn Fn(Hole<ast::RemoteNodeId>) -> ast::Quotient>
        where
            F: Fn(Hole<ast::RemoteNodeId>) -> ast::Quotient,
        {
            Box::new(move |x| f(x))
        }

        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "node" => Ok(box_up(&ast::Quotient::Node)),
                "input" => Ok(box_up(&ast::Quotient::Input)),
                "output" => Ok(box_up(&ast::Quotient::Output)),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn quotient(input: Node) -> ParseResult<ast::Quotient> {
        Ok(match_nodes!(input.into_children();
            [none] => ast::Quotient::None,
            [quotient_name(quot), meta_remote(remote)] => {
                quot(Some(remote))
            },
        ))
    }

    fn quotient_hole(input: Node) -> ParseResult<Hole<ast::Quotient>> {
        Ok(match_nodes!(input.into_children();
            [hole(hole)] => None,
            [none(none)] => Some(ast::Quotient::None),
            [quotient_name(quot), meta_remote_hole(remote)] => {
                Some(quot(Some(remote)))
            },
        ))
    }

    fn flow(input: Node) -> ParseResult<ir::Flow> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "none" => Ok(ir::Flow::None),
                "have" => Ok(ir::Flow::Have),
                "met" => Ok(ir::Flow::Met),
                "need" => Ok(ir::Flow::Need),
                _ => unreachable!(),
            })
    }

    fn tag(input: Node) -> ParseResult<ast::Tag> {
        Ok(match_nodes!(input.into_children();
            [none] => ast::Tag { quot : ast::Quotient::None, flow : ir::Flow::None},
            [quotient(quot), flow(flow)] => ast::Tag { quot, flow }
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

    fn ref_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
            [name_type_separator(name), typ(storage_type), place(storage_place)] =>
                ast::TypeDecl::Local(ast::LocalType {
                    name,
                    data: ast::LocalTypeInfo::Ref {
                        storage_type,
                        storage_place
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

    fn name_elements(input: Node) -> ParseResult<Vec<CommandId>> {
        Ok(match_nodes!(input.into_children();
               [name(names)..] => names.map(|name| CommandId(name)).collect()
        ))
    }

    fn name_list(input: Node) -> ParseResult<Vec<CommandId>> {
        Ok(match_nodes!(input.into_children();
            [name_elements(names)] => names,
            [name(name)] => vec![CommandId(name)]
        ))
    }

    fn name_hole_elements(input: Node) -> ParseResult<Vec<Hole<CommandId>>> {
        Ok(match_nodes!(input.into_children();
               [name_hole(names)..] => names.map(|name| name.map(|s| CommandId(s))).collect()
        ))
    }

    fn name_box(input: Node) -> ParseResult<Hole<Vec<Hole<CommandId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole_elements(lst)] => Some(lst),
            [hole] => None
        ))
    }

    fn name_box_single(input: Node) -> ParseResult<Hole<Vec<Hole<CommandId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_box(b)] => b,
            [name_hole(name)] => Some(vec![name.map(|s| CommandId(s))])
        ))
    }

    fn name_call(input: Node) -> ParseResult<Hole<Vec<Hole<CommandId>>>> {
        Ok(match_nodes!(input.into_children();
            [name_hole_elements(lst)] => Some(lst),
            [hole] => None
        ))
    }

    fn assign(input: Node) -> ParseResult<ast::CommandId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => CommandId(name)
        ))
    }

    fn n_elements(input: Node) -> ParseResult<Vec<Hole<usize>>> {
        Ok(match_nodes!(input.into_children();
            [n(values)..] => values.map(|v| Some(v)).collect(),
        ))
    }

    fn n_list(input: Node) -> ParseResult<Hole<Vec<Hole<usize>>>> {
        Ok(match_nodes!(input.into_children();
            [n(n)] => Some(vec![Some(n)]),
            [n_elements(values)] => Some(values)
        ))
    }

    fn impl_box(input: Node) -> ParseResult<(bool, FunctionClassId)> {
        Ok(match_nodes!(input.into_children();
            [impl_sep, function_class_name(name)] => (false, FunctionClassId(name)),
            [impl_sep, default, function_class_name(name)] => (true, FunctionClassId(name))
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

    fn external_input(input: Node) -> ParseResult<CommandId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => CommandId(name)
        ))
    }

    fn external_output(input: Node) -> ParseResult<CommandId> {
        Ok(match_nodes!(input.into_children();
            [name(name)] => CommandId(name)
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
                ast::ExternalArgument { name: Some(CommandId(name)), ffi_type },
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
            [function_class_sep, function_class_name(name),
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

    fn schedule_box(
        input: Node,
    ) -> ParseResult<(
        ParseBindingInfo,
        Option<FuncletId>,
        Option<FuncletId>,
        Option<FuncletId>,
    )> {
        fn build_parse_info(
            val: Option<(String, String)>,
            time: Option<(String, String)>,
            space: Option<(String, String)>,
        ) -> (
            ParseBindingInfo,
            Option<FuncletId>,
            Option<FuncletId>,
            Option<FuncletId>,
        ) {
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
            (ParseBindingInfo { meta_map }, value, timeline, spatial)
        }
        Ok(match_nodes!(input.into_children();
            [schedule_box_value(val), schedule_box_timeline(time),
            schedule_box_spatial(space)] => build_parse_info(val, time, space)
        ))
    }

    fn schedule_implicit(input: Node) -> ParseResult<(ast::Tag, ast::Tag)> {
        Ok(match_nodes!(input.into_children();
            [tag(itag), tag(otag)] => (itag, otag)
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
            [schedule_funclet(funclet)] => Ok(funclet),
            [timeline_sep, timeline_funclet(funclet)] => Ok(funclet),
            [spatial_sep, spatial_funclet(funclet)] => Ok(funclet),
        )
    }

    fn funclet_arg(input: Node) -> ParseResult<ast::FuncletArgument> {
        let error = error_hole(&input);
        Ok(match_nodes!(input.into_children();
            [name(name), typ(typ)] =>  ast::FuncletArgument {
                    name: Some(CommandId(name)),
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

    fn value_command(input: Node) -> ParseResult<ast::NamedCommand> {
        Ok(match_nodes!(input.into_children();
            [value_node((name, node))] => ast::NamedCommand {
                name,
                command: ast::Command::Node(node)
            },
            [tail_edge(tail_edge)] => ast::NamedCommand {
                name: None,
                command: ast::Command::TailEdge(tail_edge)
            }
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

    fn timeline_command(input: Node) -> ParseResult<ast::NamedCommand> {
        Ok(match_nodes!(input.into_children();
            [timeline_node((name, node))] => ast::NamedCommand {
                name,
                command: ast::Command::Node(node)
            },
            [tail_edge(tail_edge)] => ast::NamedCommand {
                name: None,
                command: ast::Command::TailEdge(tail_edge)
            }
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

    fn spatial_command(input: Node) -> ParseResult<ast::NamedCommand> {
        Ok(match_nodes!(input.into_children();
            [spatial_node((name, node))] => ast::NamedCommand {
                name,
                command: ast::Command::Node(node)
            },
            [tail_edge(tail_edge)] => ast::NamedCommand {
                name: None,
                command: ast::Command::TailEdge(tail_edge)
            }
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
                name: Some(CommandId(name)),
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

    fn schedule_header_parameterized(
        input: Node,
    ) -> ParseResult<(
        (ast::Tag, ast::Tag),
        String,
        Vec<ast::FuncletArgument>,
        Vec<ast::FuncletArgument>,
    )> {
        Ok(match_nodes!(input.into_children();
            [schedule_implicit(tags), name(name), schedule_args(args), schedule_return(ret)] =>
                (tags, name, args, ret)
        ))
    }

    fn schedule_header(input: Node) -> ParseResult<(ParseBindingInfo, ast::FuncletHeader)> {
        // requires that UserData be setup properly
        // unwrap with a panic cause this is an internal error if it happens
        match_nodes!(input.into_children();
            [schedule_box((binding, value, timeline, spatial)), mut schedule_header_parameterized] => {
                *schedule_header_parameterized.user_data().binding_info.borrow_mut() =
                    Some(binding.clone());
                let parameterized = CaimanAssemblyParser::schedule_header_parameterized
                    (schedule_header_parameterized);
                parameterized.map(|((itag, otag), name, args, ret)|
                    (binding, ast::FuncletHeader {
                        name: FuncletId(name),
                        args,
                        ret,
                        binding: ast::FuncletBinding::ScheduleBinding(ast::ScheduleBinding {
                            implicit_tags: Some((itag, otag)),
                            value,
                            timeline,
                            spatial
                        })
                    })
                )
            }
        )
    }

    fn schedule_command(input: Node) -> ParseResult<ast::NamedCommand> {
        Ok(match_nodes!(input.into_children();
            [schedule_node((name, node))] => ast::NamedCommand {
                name,
                command: ast::Command::Node(node)
            },
            [tail_edge(tail_edge)] => ast::NamedCommand {
                name: None,
                command: ast::Command::TailEdge(tail_edge)
            },
            [node_hole] => ast::NamedCommand {
                name: None,
                command: ast::Command::Hole
            }
        ))
    }

    fn schedule_commands(input: Node) -> ParseResult<Vec<ast::NamedCommand>> {
        Ok(match_nodes!(input.into_children();
            [schedule_command(commands)..] => commands.collect()
        ))
    }

    fn schedule_funclet(input: Node) -> ParseResult<ast::Funclet> {
        match_nodes!(input.into_children();
            [schedule_header((binding, header)), schedule_commands] => {
                *schedule_commands.user_data().binding_info.borrow_mut() = Some(binding);
                let commands_res = CaimanAssemblyParser::schedule_commands(schedule_commands);
                commands_res.map(|commands|
                    ast::Funclet {
                        kind: ir::FuncletKind::ScheduleExplicit,
                        header,
                        commands,
                    }
                )
            }
        )
    }

    fn triple_box(
        input: Node,
    ) -> ParseResult<(
        Hole<ast::Quotient>,
        Hole<ast::Quotient>,
        Hole<ast::Quotient>,
    )> {
        Ok(match_nodes!(input.into_children();
            [value_sep, quotient_hole(vq),
                timeline_sep, quotient_hole(tq),
                spatial_sep, quotient_hole(sq)] => (vq, tq, sq)
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
                    join: join.map(|s| CommandId(s)),
                    arguments
                }
        ))
    }

    fn schedule_call_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_call_sep, name_hole(callee_funclet_id),
                triple_box((value_operation, timeline_operation, spatial_operation)),
                name_call(callee_arguments), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleCall {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    callee_funclet_id: callee_funclet_id.map(|s| FuncletId(s)),
                    callee_arguments,
                    continuation_join: continuation_join.map(|s| CommandId(s))
                }
        ))
    }

    fn schedule_select_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_call_sep, name_hole(condition), name_box(callee_funclet_ids),
                triple_box((value_operation, timeline_operation, spatial_operation)),
                name_call(callee_arguments), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleSelect {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    condition: condition.map(|s| CommandId(s)),
                    callee_funclet_ids: callee_funclet_ids.map(
                        |v| v.into_iter().map(
                            |name| name.map(
                                |s| FuncletId(s.0)
                            )
                        ).collect()
                    ),
                    callee_arguments,
                    continuation_join: continuation_join.map(|s| CommandId(s))
                }
        ))
    }

    fn schedule_yield_node(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
            [schedule_yield_sep, name_hole(external_function_id),
                triple_box((value_operation, timeline_operation, spatial_operation)),
                name_call(yielded_nodes), name_hole(continuation_join)] =>
                ast::TailEdge::ScheduleCallYield {
                    value_operation,
                    timeline_operation,
                    spatial_operation,
                    external_function_id: external_function_id.map(|s| ExternalFunctionId(s)),
                    yielded_nodes,
                    continuation_join: continuation_join.map(|s| CommandId(s))
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

    fn constant_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), name_sep(type_id), constant_value(value)] => (Some(name),
                ast::Node::Constant {
                    value: Some(value),
                    type_id: Some(ast::TypeId::Local(type_id))
                }
            )
        ))
    }

    fn extract_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), extract_sep, name(node_id), n(index)] => (Some(name),
                ast::Node::ExtractResult {
                    node_id: Some(CommandId(node_id)),
                    index: Some(index)
                }
            )
        ))
    }

    fn call_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        // will split apart later
        Ok(match_nodes!(input.into_children();
            [assign(name), call_sep, function_class_name(external_function_id),
                name_call(arguments)] => (Some(name),
                    ast::Node::CallFunctionClass {
                        function_id: Some(FunctionClassId(external_function_id)),
                        arguments
                    }
                )
        ))
    }

    fn select_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), select_sep, name_sep(condition),
                name_sep(true_case), name(false_case)] => (Some(name),
                    ast::Node::Select {
                        condition: Some(CommandId(condition)),
                        true_case: Some(CommandId(true_case)),
                        false_case: Some(CommandId(false_case))

                    }
                )
        ))
    }

    fn encoding_event_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), encoding_event_sep, name_sep(local_past),
                name_box(remote_local_pasts)] => (Some(name),
                    ast::Node::EncodingEvent {
                        local_past: Some(CommandId(local_past)),
                        remote_local_pasts

                    }
                )
        ))
    }

    fn submission_event_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submission_event_sep, name(local_past)] => (Some(name),
                    ast::Node::SubmissionEvent {
                            local_past: Some(CommandId(local_past))
                    }
                )
        ))
    }

    fn synchronization_event_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submission_event_sep, name_sep(local_past),
                name(remote_local_past)] => (Some(name),
                    ast::Node::SynchronizationEvent {
                            local_past: Some(CommandId(local_past)),
                            remote_local_past: Some(CommandId(remote_local_past))
                    }
                )
        ))
    }

    fn separated_buffer_space_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), separated_buffer_space_sep,
                n_sep(count), name(space)] => (Some(name),
                    ast::Node::SeparatedBufferSpaces {
                            count: Some(count),
                            space: Some(CommandId(space))
                    }
                )
        ))
    }

    fn alloc_temporary_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), alloc_temporary_sep, place_hole_sep(place),
                ffi_type_hole(storage_type)] => (Some(name),
                    ast::Node::AllocTemporary {
                        place,
                        storage_type
                    }
                )
        ))
    }

    fn drop_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [name_hole(node)] => (None,
                ast::Node::Drop {
                    node: node.map(|s| CommandId(s))
                }
            )
        ))
    }

    fn static_sub_alloc_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_sub_alloc_sep, place_hole_sep(place),
                ffi_type_hole_sep(storage_type), name_hole(node)] => (Some(name),
                    ast::Node::StaticSubAlloc {
                        node: node.map(|s| CommandId(s)),
                        place,
                        storage_type
                    }
                )
        ))
    }

    fn static_alloc_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_alloc_sep, place_hole_sep(place),
                name_hole(node), n_list(sizes), quotient_hole(spatial_operation)] => (Some(name),
                    ast::Node::StaticAlloc {
                        node: node.map(|s| CommandId(s)),
                        place,
                        sizes,
                        spatial_operation
                    }
                )
        ))
    }

    fn static_dealloc_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), static_sub_alloc_sep, place_hole_sep(place),
                quotient_hole(spatial_operation), name_box(nodes)] => (Some(name),
                ast::Node::StaticDealloc {
                        nodes,
                        place,
                        spatial_operation
                    }
                )
        ))
    }

    fn read_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), read_sep, ffi_type_hole_sep(storage_type),
                name_hole(source)] => (Some(name),
                    ast::Node::ReadRef {
                        source: source.map(|s| CommandId(s)),
                        storage_type
                    }
                )
        ))
    }

    fn borrow_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), borrow_sep, ffi_type_hole_sep(storage_type),
                name_hole(source)] => (Some(name),
                    ast::Node::BorrowRef {
                        source: source.map(|s| CommandId(s)),
                        storage_type
                    }
                )
        ))
    }

    fn write_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [write_sep, ffi_type_hole_sep(storage_type),
                name_hole(source), name_hole(destination)] => (None,
                    ast::Node::WriteRef {
                        storage_type,
                        source: source.map(|s| CommandId(s)),
                        destination: destination.map(|s| CommandId(s))
                    }
                )
        ))
    }

    fn local_do_builtin_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [local_do_builtin_sep, quotient_hole(operation),
                name_call(inputs), name_box_single(outputs)] => (None,
                    ast::Node::LocalDoBuiltin {
                        operation,
                        inputs,
                        outputs
                    }
                )
        ))
    }

    fn local_do_external_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [local_do_external_sep, name_hole_sep(external_function_id),
                quotient_hole(operation), name_call(inputs),
                name_box_single(outputs)] => (None,
                    ast::Node::LocalDoExternal {
                        external_function_id: external_function_id.map(|s| ExternalFunctionId(s)),
                        operation,
                        inputs,
                        outputs
                    }
                )
        ))
    }

    fn local_copy_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [local_copy_sep, name_hole(input), name_hole(output)] => (None,
                    ast::Node::LocalCopy {
                        input: input.map(|s| CommandId(s)),
                        output: output.map(|s| CommandId(s))
                    }
                )
        ))
    }

    fn begin_encoding_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), begin_encoding_sep, place_hole_sep(place),
                quotient_hole(event), name_box(encoded),
                name_box_single(fences)] => (Some(name),
                    ast::Node::BeginEncoding {
                        place,
                        event,
                        encoded,
                        fences
                    }
                )
        ))
    }

    fn encode_do_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [encode_do_sep, name_hole_sep(encoder), name_hole_sep(external_function_id),
                quotient_hole(operation), name_call(inputs),
                name_box_single(outputs)] => (None,
                    ast::Node::EncodeDoExternal {
                        encoder: encoder.map(|s| CommandId(s)),
                        external_function_id: external_function_id.map(|s| ExternalFunctionId(s)),
                        operation,
                        inputs,
                        outputs
                    }
                )
        ))
    }

    fn encode_copy_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [encode_copy_sep, name_hole_sep(encoder), name_hole(input),
                name_hole(output)] => (None,
                    ast::Node::EncodeCopy {
                        encoder: encoder.map(|s| CommandId(s)),
                        input: input.map(|s| CommandId(s)),
                        output: output.map(|s| CommandId(s))
                    }
                )
        ))
    }

    fn submit_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), submit_sep, name_hole_sep(encoder),
                quotient_hole(event)] => (Some(name),
                    ast::Node::Submit {
                        encoder: encoder.map(|s| CommandId(s)),
                        event
                    }
                )
        ))
    }

    fn sync_fence_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [sync_fence_sep, name_hole_sep(fence), quotient_hole(event)] => (None,
                    ast::Node::SyncFence {
                        fence: fence.map(|s| CommandId(s)),
                        event
                    }
                )
        ))
    }

    fn inline_join_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), inline_join_sep, name_hole_sep(funclet), name_box(captures),
                name_hole(continuation)] => (Some(name),
                    ast::Node::InlineJoin {
                        funclet: funclet.map(|s| FuncletId(s)),
                        captures,
                        continuation: continuation.map(|s| CommandId(s)),
                    }
                )
        ))
    }

    fn serialized_join_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), serialized_join_sep, name_hole_sep(funclet),
                name_box(captures), name_hole(continuation)] => (None,
                    ast::Node::SerializedJoin {
                        funclet: funclet.map(|s| FuncletId(s)),
                        captures,
                        continuation: continuation.map(|s| CommandId(s)),
                    }
                )
        ))
    }

    fn default_join_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name)] => (Some(name), ast::Node::DefaultJoin{})
        ))
    }

    fn promise_captures_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), promise_captures_sep, n(count),
                name_hole(continuation)] => (Some(name),
                    ast::Node::PromiseCaptures {
                        count: Some(count),
                        continuation: continuation.map(|s| CommandId(s))
                    }
                )
        ))
    }

    fn fulfill_captures_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [assign(name), fulfill_captures_sep, name_hole_sep(continuation),
                name_box(haves), name_box(needs)] => (Some(name),
                    ast::Node::FulfillCaptures {
                        continuation: continuation.map(|s| CommandId(s)),
                        haves,
                        needs
                    }
                )
        ))
    }

    fn value_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [constant_node(n)] => n,
            [extract_node(n)] => n,
            [call_node(n)] => n,
            [select_node(n)] => n
        ))
    }

    fn timeline_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [extract_node(n)] => n,
            [encoding_event_node(n)] => n,
            [submission_event_node(n)] => n,
            [synchronization_event_node(n)] => n

        ))
    }

    fn spatial_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [extract_node(n)] => n,
            [separated_buffer_space_node(n)] => n
        ))
    }

    fn schedule_node(input: Node) -> ParseResult<(Option<CommandId>, ast::Node)> {
        Ok(match_nodes!(input.into_children();
            [alloc_temporary_node(n)] => n,
            [drop_node(n)] => n,
            [static_sub_alloc_node(n)] => n,
            [static_alloc_node(n)] => n,
            [static_dealloc_node(n)] => n,
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
                path: "".to_string(),
                version,
                declarations: declarations.collect()
            }
        ))
    }
}

pub fn parse(path: &str, code: &str) -> ParseResult<ast::Program> {
    // necessary to have an empty user data for checking stuff
    let user_data = UserData {
        binding_info: RefCell::new(None),
    };
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
