use pest::iterators::{Pair, Pairs};
use pest_consume::{match_nodes, Error, Parser};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
struct CaimanAssemblyParser;

use crate::{assembly, frontend, ir};
use assembly::ast;
use ast::Hole;
use ast::{
    ExternalFunctionId, FFIType, FuncletId, OperationId, RemoteNodeId, StorageTypeId, TypeId,
    ValueFunctionId,
};
use ir::ffi;

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

#[derive(Clone, Debug)]
struct UserData {}

fn unexpected(s: String) -> String {
    format!("Unexpected string {}", s)
}

fn reject_hole<T>(input: Node, h: Hole<T>) -> ParseResult<T> {
    match h {
        Some(v) => Ok(v),
        None => Err(input.error("Invalid hole")),
    }
}

// dumb hack
fn create_map<F, T, U>(f: &'static F) -> Box<dyn Fn(T) -> U>
    where
        F: Fn(T) -> U,
{
    Box::new(move |x| f(x))
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

    fn type_name(input: Node) -> ParseResult<Hole<TypeId>> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => Some(TypeId::Local(s)),
            [hole] => None
        ))
    }

    fn type_name_sep(input: Node) -> ParseResult<Hole<TypeId>> {
        Ok(match_nodes!(input.into_children();
            [type_name(t)] => t
        ))
    }

    fn throwaway(input: Node) -> ParseResult<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }

    fn var_name(input: Node) -> ParseResult<Hole<OperationId>> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => Some(OperationId(s)),
            [n] => Some(OperationId(n.as_str().to_string())),
            [throwaway(s)] => Some(OperationId(s)),
            [hole] => None
        ))
    }

    fn var_name_sep(input: Node) -> ParseResult<Hole<OperationId>> {
        Ok(match_nodes!(input.into_children();
            [var_name(t)] => t
        ))
    }

    fn fn_name(input: Node) -> ParseResult<Hole<FuncletId>> {
        Ok(match_nodes!(input.into_children();
            [id(s)] => Some(FuncletId(s)),
            [hole] => None
        ))
    }

    fn fn_name_sep(input: Node) -> ParseResult<Hole<FuncletId>> {
        Ok(match_nodes!(input.into_children();
            [fn_name(t)] => t
        ))
    }

    fn funclet_loc_filled(input: Node) -> ParseResult<RemoteNodeId> {
        Ok(match_nodes!(input.into_children();
            [fn_name(funclet_name), var_name(node_name)] => ast::RemoteNodeId {
                funclet_name,
                node_name
            }
        ))
    }

    fn funclet_loc(input: Node) -> ParseResult<Hole<RemoteNodeId>> {
        Ok(match_nodes!(input.into_children();
            [funclet_loc_filled(f)] => Some(f),
            [hole] => None
        ))
    }

    fn funclet_loc_sep(input: Node) -> ParseResult<Hole<RemoteNodeId>> {
        Ok(match_nodes!(input.into_children();
            [funclet_loc(t)] => t
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

    fn typ(input: Node) -> ParseResult<Hole<ast::Type>> {
        Ok(match_nodes!(input.into_children();
            [ffi_type(t)] => Some(ast::Type::FFI(t)),
            [type_name(v)] => v.map(|s| ast::Type::Local(s)),
        ))
    }

    fn place(input: Node) -> ParseResult<Hole<ir::Place>> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "local" => Ok(Some(ir::Place::Local)),
                "cpu" => Ok(Some(ir::Place::Cpu)),
                "gpu" => Ok(Some(ir::Place::Gpu)),
                "?" => Ok(None),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn stage(input: Node) -> ParseResult<Hole<ir::ResourceQueueStage>> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "unbound" => Ok(Some(ir::ResourceQueueStage::Unbound)),
                "bound" => Ok(Some(ir::ResourceQueueStage::Bound)),
                "encoded" => Ok(Some(ir::ResourceQueueStage::Encoded)),
                "submitted" => Ok(Some(ir::ResourceQueueStage::Submitted)),
                "ready" => Ok(Some(ir::ResourceQueueStage::Ready)),
                "dead" => Ok(Some(ir::ResourceQueueStage::Dead)),
                "?" => Ok(None),
                _ => Err(input.error(unexpected(s))),
            })
    }

    // weirdly, this seems like the best way to do this with pest_consume for now?
    fn tag_core_op(input: Node) -> ParseResult<Box<dyn Fn(ast::RemoteNodeId) -> ast::Tag>> {
        fn box_up<F>(f: &'static F) -> Box<dyn Fn(ast::RemoteNodeId) -> ast::TagCore>
        where
            F: Fn(ast::RemoteNodeId) -> ast::TagCore,
        {
            Box::new(move |x| f(x))
        }

        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "operation" => Ok(create_map(&ast::TagCore::Operation)),
                "input" => Ok(create_map(&ast::TagCore::Input)),
                "output" => Ok(create_map(&ast::TagCore::Output)),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn tag_core(input: Node) -> ParseResult<ast::TagCore> {
        reject_hole(input, match_nodes!(input.into_children();
            [tag_core_op(op), funclet_loc(f)] => f.map(|loc| op(loc)),
            [none] => Some(ast::TagCore::None)
        ))
    }

    fn tag_halt(input: Node) -> ParseResult<ast::OperationId> {
        match_nodes!(input.into_children();
            [var_name(v)] => reject_hole(input, v)
        )
    }

    fn value_tag_op(input: Node) -> ParseResult<Box<dyn Fn(ast::RemoteNodeId) -> ast::ValueTag>> {
        input
            .as_str()
            .parse::<String>()
            .map_err(|e| input.error(e))
            .and_then(|s| match s.as_str() {
                "function_input" => Ok(create_map(&ast::ValueTag::FunctionInput)),
                "function_output" => Ok(create_map(&ast::ValueTag::FunctionOutput)),
                _ => Err(input.error(unexpected(s))),
            })
    }

    fn value_tag_loc(input: Node) -> ParseResult<ast::ValueTag> {
        reject_hole(input, match_nodes!(input.into_children();
            [value_tag_op(op), funclet_loc(f)] => f.map(|loc| op(loc))
        ))
    }

    fn value_tag_data(input: Node) -> ParseResult<ast::ValueTag> {
        Ok(match_nodes!(input.into_children();
            [tag_core(c)] => ast::ValueTag::Core(c),
            [value_tag_loc(t)] => t,
            [tag_halt(h)] => ast::ValueTag::Halt(h)
        ))
    }

    fn value_tag(input: Node) -> ParseResult<ast::ValueTag> {
        Ok(match_nodes!(input.into_children();
            [value_tag_data(d)] => d
        ))
    }

    fn timeline_tag(input: Node) -> ParseResult<ast::TimelineTag> {
        Ok(match_nodes!(input.into_children();
             [tag_core(c)] => ast::TimelineTag::Core(c)
        ))
    }

    fn spatial_tag(input: Node) -> ParseResult<ast::SpatialTag> {
        Ok(match_nodes!(input.into_children();
             [tag_core(c)] => ast::SpatialTag::Core(c)
        ))
    }

    fn tag(input: Node) -> ParseResult<ast::Tag> {
        Ok(match_nodes!(input.into_children();
            [value_tag(t)] => ast::Tag::ValueTag(t),
            [timeline_tag(t)] => ast::Tag::TimelineTag(t),
            [spatial_tag(t)] => ast::Tag::SpatialTag(t)
        ))
    }

    fn slot_info(input: Node) -> ParseResult<ast::SlotInfo> {
        let tags : Vec<ast::Tag> = match_nodes!(input.into_children();
            [tag(t)..] => t.collect()
        );
        let mut value_tag = ast::ValueTag::Core(ast::TagCore::None);
        let mut timeline_tag = ast::TimelineTag::Core(ast::TagCore::None);
        let mut spatial_tag = ast::SpatialTag::Core(ast::TagCore::None);
        for tag in tags.iter() {
            match tag {
                // duplicates are whatever
                ast::Tag::ValueTag(t) => value_tag = t.clone(),
                ast::Tag::TimelineTag(t) => timeline_tag = t.clone(),
                ast::Tag::SpatialTag(t) => spatial_tag = t.clone(),
            }
        }
        Ok(ast::SlotInfo {
            value_tag,
            timeline_tag,
            spatial_tag,
        })
    }

    fn fence_info(input: Node) -> ParseResult<ast::FenceInfo> {
        let timeline_tag = match_nodes!(input.into_children();
            [] => ast::TimelineTag::Core(ast::TagCore::None),
            [timeline_tag(t)] => t
        );
        Ok(ast::FenceInfo { timeline_tag })
    }

    fn buffer_info(input: Node) -> ParseResult<ast::BufferInfo> {
        let spatial_tag = match_nodes!(input.into_children();
            [] => ast::SpatialTag::Core(ast::TagCore::None),
            [spatial_tag(t)] => t
        );
        Ok(ast::BufferInfo { spatial_tag })
    }

    fn value(input: Node) -> ParseResult<ast::Value> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn list(input: Node) -> ParseResult<Vec<ast::DictValue>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn dict_value(input: Node) -> ParseResult<ast::DictValue> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn dict_key(input: Node) -> ParseResult<ast::Value> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn dict_element(input: Node) -> ParseResult<(String, ast::DictValue)> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn version(input: Node) -> ParseResult<ast::Version> {
        dbg!(&input);
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn ir_type_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn ffi_type_decl(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn type_def(input: Node) -> ParseResult<ast::TypeDecl> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn types(input: Node) -> ParseResult<ast::Types> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn node_list(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn node_box_raw(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn node_box(input: Node) -> ParseResult<Hole<Vec<Hole<String>>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn return_args(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn return_command(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn yield_command(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn jump_command(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_nodes(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_box_raw(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_fn_box(input: Node) -> ParseResult<Hole<Vec<Hole<String>>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_option_node(input: Node) -> ParseResult<Option<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn tail_edge(input: Node) -> ParseResult<ast::TailEdge> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn phi_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn constant_raw(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn constant(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn constant_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn extract_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn call_args(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn call_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn select_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn value_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn value_assign(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_args(input: Node) -> ParseResult<Vec<Hole<String>>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_params(input: Node) -> ParseResult<Box<[Hole<String>]>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_do_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn create_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn drop_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn alloc_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn encode_copy_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn submit_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_fence_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn inline_join_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn default_join_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn sync_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn submission_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn timeline_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn spatial_command(input: Node) -> ParseResult<ast::Node> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn spatial_assign(input: Node) -> ParseResult<ast::NamedNode> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn value_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn schedule_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn timeline_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn spatial_funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn funclet(input: Node) -> ParseResult<ast::Funclet> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn funclet_def(input: Node) -> ParseResult<ast::FuncletDef> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn value_function_funclets(input: Node) -> ParseResult<Vec<String>> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn funclets(input: Node) -> ParseResult<ast::FuncletDefs> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn extras(input: Node) -> ParseResult<ast::Extras> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn pipelines(input: Node) -> ParseResult<ast::Pipelines> {
        Ok(match_nodes!(input.into_children();
        ))
    }

    fn program(input: Node) -> ParseResult<ast::Program> {
        let (version, types, funclets, extras, pipelines) = match_nodes!(input.into_children();
            [version(v), types(t), funclets(f), extras(e), pipelines(p), EOI(_)] => (v, t, f, e, p)
        );

        Ok(ast::Program {
            version,
            types,
            funclets,
            extras,
            pipelines,
        })
    }
}

pub fn parse(code: &str) -> ParseResult<ast::Program> {
    // necessary to have an empty user data for checking stuff
    let user_data = UserData {};
    let parsed = CaimanAssemblyParser::parse_with_userdata(Rule::program, code, user_data)?;
    CaimanAssemblyParser::program(parsed.single()?)
}
