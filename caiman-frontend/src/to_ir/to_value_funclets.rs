use super::ir_funclets::{InnerFunclet, ValueFunclet};
use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
use super::vil::{self, Expr, Value};
use caiman::assembly_ast as asm;
//use super::error::ToIRResult;

pub fn vil_to_value_funclets(
    vil_program: &vil::Program,
) -> Vec<ValueFunclet>
{
    let mut commands: Vec<Option<asm::Node>> = vil_program
        .stmts
        .iter()
        .map(|stmt| match &stmt.expr
        {
            Expr::Value(val) => { 
                let (value_str, t) = match val
                {
                Value::I32(i) => (i.to_string(), asm::FFIType::I32),
                Value::I64(i) => (i.to_string(), asm::FFIType::I64),
                Value::U64(u) => (u.to_string(), asm::FFIType::U64),
                };
                Some(asm::Node::Constant {
                    value: Some(value_str),
                    type_id: Some(asm::Type::FFI(t)),
                })
            },
            Expr::If(guard, e1, e2) =>
            {
                todo!()
            },
        })
        .collect();
    commands.insert(0, Some(asm::Node::Phi { index: Some(0) }));
    // TODO actually calculate the header and tail edge
    let dummy_header = asm::FuncletHeader {
        ret: vec![], 
        name: "my_great_valuefunclet".to_string(),
        args: vec![],
    };
    let dummy_tail_edge = None;
    vec![ValueFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge: dummy_tail_edge },
    }]
}
