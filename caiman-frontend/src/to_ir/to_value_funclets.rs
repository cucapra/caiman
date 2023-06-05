use super::context;
use super::ir_funclets::{InnerFunclet, ValueFunclet};
use super::label;
use super::vil::{self, Expr, Value};
use caiman::assembly::ast as asm;
//use super::error::ToIRResult;

pub fn vil_to_value_funclets(
    vil_program: &vil::Program,
    context: &mut context::Context,
) -> Vec<ValueFunclet>
{
    // XXX For now, we just have a big global funclet. Here's its name
    let global_vf_name = asm::FuncletId("my_great_valuefunclet".to_string());

    let mut node_context = context::NodeContext::new();
    // Used for returning the last node, which should eventually be undone
    // The below value is just a default, essentially
    let mut last_type = asm::TypeId::FFI(asm::FFIType::I32);
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
                last_type = asm::TypeId::FFI(t.clone());
                node_context.add_node(Some(asm::Node::Constant {
                    value: Some(value_str),
                    type_id: Some(asm::TypeId::FFI(t)),
                }));
            },
            Expr::If(guard, e1, e2) =>
            {
                todo!()
            },
        }
    }

    // TODO actually calculate the header and tail edge
    let dummy_header = asm::FuncletHeader {
        ret: vec![asm::FuncletArgument { name: None, typ: last_type, tags: Vec::new() }],
        name: global_vf_name,
        args: vec![],
        binding: asm::FuncletBinding::None,
    };

    let commands = node_context.into_commands();

    // TODO don't simply return the last node like below
    let last_id = label::label_node(commands.len() - 1);
    let tail_edge =
        Some(asm::TailEdge::Return { return_values: Some(vec![Some(last_id.clone())]) });

    vec![ValueFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge },
    }]
}
