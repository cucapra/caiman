use super::ir_funclets::{InnerFunclet, ValueFunclet};
use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
use super::context;
use super::label;
use super::vil::{self, Expr, Value};
use caiman::assembly_ast as asm;
use caiman::assembly_context as asm_ctx;
//use super::error::ToIRResult;

pub fn vil_to_value_funclets(
    vil_program: &vil::Program,
    context: &mut context::Context
) -> Vec<ValueFunclet>
{
    // XXX For now, we just have a big global funclet. Here's its name
    let global_vf_name = "my_great_valuefunclet".to_string();
    context.begin_local_funclet(global_vf_name.clone());

    let mut commands: Vec<Option<asm::Node>> = Vec::new();
    // Used for returning the last node, which should eventually be undone
    // The below value is just a default, essentially
    let mut last_type = asm::Type::FFI(asm::FFIType::I32);
    for (i, stmt) in vil_program.stmts.iter().enumerate()
    {
        match &stmt.expr
        {
            Expr::Value(val) =>
            {
                let (value_str, t) = match val
                {
                    Value::I32(i) => (i.to_string(), asm::FFIType::I32),
                    Value::I64(i) => (i.to_string(), asm::FFIType::I64),
                    Value::U64(u) => (u.to_string(), asm::FFIType::U64),
                };
                context.add_ffi_type(t.clone());
                last_type = asm::Type::FFI(t.clone());
                commands.push(Some(asm::Node::Constant {
                    value: Some(value_str),
                    type_id: Some(asm::Type::FFI(t)),
                }));
            },
            Expr::If(guard, e1, e2) =>
            {
                todo!()
            },
        }
        context.add_node();
    }

    // TODO actually calculate the header and tail edge
    let dummy_header =
        asm::FuncletHeader { ret: vec![(None, last_type)], name: global_vf_name, args: vec![] };

    // TODO don't simply return the last node like below
    let last_id = label::label_node(commands.len() - 1);
    let tail_edge =
        Some(asm::TailEdge::Return { return_values: Some(vec![Some(last_id.clone())]) });

    context.end_local_funclet();
    vec![ValueFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge },
    }]
}
