use crate::ir;
use crate::ir::Program;
use crate::shadergen;
use crate::arena::Arena;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::hash;
use std::hash::Hash;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;
pub use crate::rust_wgpu_backend::ffi as ffi;

struct PartialFunclet {
    new_id : usize,
    kind : Option<ir::FuncletKind>,
	input_types : Vec<ir::TypeId>,
	output_types : Vec<ir::TypeId>,
	nodes : Vec<ir::Node>,
	tail_edge : Option<ir::TailEdge>,
}

enum ResolvedType {
    Single (ffi::TypeId),
    Multiple (Vec<ffi::TypeId>)
}

struct ResolvedNode {
    place : ir::Place,
    node_id : usize,
    type_info : ResolvedType
}

struct SchedulingContext<'a> {
    program : &'a mut ir::Program,
    new_funclet : PartialFunclet,
    funclet_id : usize,
    node_id : usize,
    // scheduled node map
    // note that nodes can only be referenced in the funclet,
    //   so this should get reset on starting a new funclet
    resolved_map : HashMap<usize, ResolvedNode>, 
    // where future nodes to schedule live
    waiting_map : HashMap<usize, Vec<Box<Fn(ResolvedNode)->ir::Node>>>
}

fn initial_partial_funclet(prog : &Program) -> PartialFunclet {
    let id = prog.funclets.get_next_id();
    PartialFunclet { new_id : id, kind: None, input_types: Vec::new(), 
        output_types: Vec::new(), nodes: Vec::new(), tail_edge: None }
}

fn new_partial_funclet(context : &mut SchedulingContext) -> PartialFunclet {
    initial_partial_funclet(context.program)
}

fn add_new_funclet(context : &mut SchedulingContext) {
    let mut updated_funclet = new_partial_funclet(context);
    std::mem::swap(&mut updated_funclet, &mut context.new_funclet);
    let new_funclet = ir::Funclet {
        kind : updated_funclet.kind.as_ref().unwrap().to_owned(),
        input_types : updated_funclet.input_types.into_boxed_slice(),
        output_types : updated_funclet.output_types.into_boxed_slice(),
        nodes : updated_funclet.nodes.into_boxed_slice(),
        tail_edge : updated_funclet.tail_edge.as_ref().unwrap().to_owned(),
    };
    context.program.funclets.create(new_funclet);
    context.new_funclet = new_partial_funclet(context);
}

fn explicate_extract_result(node_id : &usize, 
    index : &usize, context : &mut SchedulingContext) {
    // The goal here is to maintain the hashmaps to keep track of ids
    // Specifically the funclet and node to extract from (or to)
    fn build_result(resolved : ResolvedNode) -> ir::Node {
        let remote = ir::RemoteNodeId {
            funclet_id : context.funclet_id,
            node_id : *node_id
        };
        let typ = match callInfo.type_info {
            ResolvedType::Single(typ) => 
            {
                assert!(*index == 0);
                typ 
            },
            ResolvedType::Multiple(typs) => { typs[*index] }
        };
        ir::Node { ir::Node::AllocTemporary {
            place: ir::Place::Cpu, // TODO: sus
            storage_type: typ, operation: remote }}
    }
    match context.resolved_map.get(node_id) {
        Some(callInfo) => {
            let new_node = 
            context.new_funclet.nodes.push(build_result(callInfo));
        }
        None => {
            let new_index = ir::RemoteNodeId { 
                funclet_id: context.new_funclet.new_id,
                node_id: context.node_id
            };
            match context.waiting_map.get_mut(node_id) {
                Some(v) => {
                    v.push(new_index);
                },
                None => {
                    context.waiting_map.insert(*node_id, vec![new_index]);
                }
            };
        }
    }
}

fn explicate_constant(type_id : &usize, context : &mut SchedulingContext) {
    let new_id = ir::RemoteNodeId {
        funclet_id: context.funclet_id,
        node_id: context.node_id
    };
    let new_node = ir::Node::StaticAllocFromStaticBuffer { 
        buffer: 0,
        place: ir::Place::Local, storage_type: ffi::TypeId {0: *type_id}, 
        operation: new_id };
    context.new_funclet.nodes.push(new_node);
}

fn explicate_value_function(function_id : &usize, arguments : &Box<[usize]>, 
    context : &mut SchedulingContext) {
    ()
}

fn explicate_select(condition : &usize, true_case : &usize, false_case : &usize, 
    context : &mut SchedulingContext) {
    ()
}

fn explicate_external(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_node(node : &ir::Node, context : &mut SchedulingContext) {
    match node {
        ir::Node::ExtractResult { node_id, 
            index } => explicate_extract_result(node_id, index, context),
        ir::Node::ConstantInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::ConstantUnsignedInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::CallValueFunction { function_id, 
            arguments } => 
            explicate_value_function(function_id, arguments, context),
        ir::Node::Select { condition, true_case, 
            false_case } => 
            explicate_select(condition, true_case, false_case, context),
        ir::Node::CallExternalCpu { external_function_id, 
            arguments } => explicate_external(node, context),
        ir::Node::CallExternalGpuCompute { external_function_id, 
            dimensions, arguments } => 
            explicate_external(node, context),
        _ => ()
    };
    context.node_id += 1;
}

fn explicate_funclet(funclet : &ir::Funclet, context : &mut SchedulingContext) {
    // Calculates the new funclets to add (if any)
    context.resolved_map = HashMap::new();
    let result : HashMap<usize, ir::Funclet> = HashMap::new();
    match funclet.kind {
        ir::FuncletKind::MixedImplicit => todo!(),
        ir::FuncletKind::MixedExplicit => todo!(),
        ir::FuncletKind::Value => todo!(),
        ir::FuncletKind::ScheduleExplicit => (),
        ir::FuncletKind::Inline => todo!(),
        ir::FuncletKind::Timeline => todo!(),
        ir::FuncletKind::Spatial => todo!(),
    }
}

pub fn explicate_scheduling(program : &mut ir::Program)
{
    let original = program.funclets.clone();
    let initial_funclet = initial_partial_funclet(program);
    let mut initial_context = 
        SchedulingContext{program : program, new_funclet : initial_funclet, 
            funclet_id : 0, node_id : 0, value_map : HashMap::new(), 
            schedule_map: HashMap::new()};
    for funclet in original.iter() {
        explicate_funclet(funclet.1, &mut initial_context)
    }
}