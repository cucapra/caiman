use crate::ir;

use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;

struct PartialFunclet {
    pub kind : Option<ir::FuncletKind>,
	pub input_types : Option<Box<[ir::TypeId]>>,
	pub output_types : Option<Box<[ir::TypeId]>>,
	pub nodes : Option<Box<[ir::Node]>>,
	pub tail_edge : Option<ir::TailEdge>,
}

struct SchedulingContext {
    program : ir::Program,
    new_funclet : PartialFunclet
}

fn new_partial_funclet() -> PartialFunclet {
    PartialFunclet { kind: None, input_types: None, 
        output_types: None, nodes: None, tail_edge: None }
}

fn add_new_funclet(context : &mut SchedulingContext) {
    let new_funclet = ir::Funclet {
        kind : context.new_funclet.kind.as_ref().unwrap().to_owned(),
        input_types : context.new_funclet.input_types.as_ref().unwrap().to_owned(),
        output_types : context.new_funclet.output_types.as_ref().unwrap().to_owned(),
        nodes : context.new_funclet.nodes.as_ref().unwrap().to_owned(),
        tail_edge : context.new_funclet.tail_edge.as_ref().unwrap().to_owned(),
    };
    context.program.funclets.create(new_funclet);
    context.new_funclet = new_partial_funclet();
}

fn explicate_phi(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_extract_result(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_constant(type_id : &usize, context : &mut SchedulingContext) {
    new_node = ir::Node::AllocTemporary { 
        place: (), storage_type: type_id, operation: () }
}

fn explicate_value_function(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_select(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_external(node : &ir::Node, context : &mut SchedulingContext) {
    ()
}

fn explicate_node(node : &ir::Node, context : &mut SchedulingContext) {
    match node {
        ir::Node::Phi { index } => explicate_phi(node, context),
        ir::Node::ExtractResult { node_id, 
            index } => explicate_extract_result(node, context),
        ir::Node::ConstantInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::ConstantUnsignedInteger { value, 
            type_id } => explicate_constant(type_id, context),
        ir::Node::CallValueFunction { function_id, 
            arguments } => 
            explicate_value_function(node, context),
        ir::Node::Select { condition, true_case, 
            false_case } => explicate_select(node, context),
        ir::Node::CallExternalCpu { external_function_id, 
            arguments } => explicate_external(node, context),
        ir::Node::CallExternalGpuCompute { external_function_id, 
            dimensions, arguments } => 
            explicate_external(node, context),
        _ => ()
    }
}

fn explicate_funclet(funclet : &ir::Funclet, context : &mut SchedulingContext) {
    // Calculates the new funclets to add (if any)
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
    let mut initial_context = 
        SchedulingContext{program : program, new_funclet : 
            new_partial_funclet()};
    for funclet in original.iter() {
        explicate_funclet(funclet.1, &mut initial_context)
    }
}