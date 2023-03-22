use super::ir_funclets::{InnerFunclet, ValueFunclet};
use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
use super::vil::{self, Expr, Value};
use caiman::assembly_ast as asm;
//use super::error::ToIRResult;

pub fn vil_to_value_funclets(
    vil_program: &vil::Program,
) -> Vec<ValueFunclet>
{
    let mut commands: Vec<asm::Node> = vil_program
        .stmts
        .iter()
        .map(|stmt| match &stmt.expr
        {
            Expr::Value(val) => match val
            {
                Value::I32(i) => asm::Node::ConstantI32 {
                    value: *i,
                    type_id: asm::Type::FFI(asm::FFIType::I32),
                },
                Value::I64(i) => asm::Node::ConstantInteger {
                    value: *i,
                    type_id: asm::Type::FFI(asm::FFIType::I64),
                },
                Value::U64(u) => asm::Node::ConstantUnsignedInteger {
                    value: *u,
                    type_id: asm::Type::FFI(asm::FFIType::U64),
                },
            },
            Expr::If(guard, e1, e2) =>
            {
                todo!()
            },
        })
        .collect();
    commands.insert(0, asm::Node::Phi { index: 0 });
    // TODO actually calculate the header and tail edge
    let dummy_header = asm::FuncletHeader {
        ret: asm::Type::FFI(asm::FFIType::I32),
        name: "my_great_valuefunclet".to_string(),
        args: vec![],
    };
    let dummy_tail_edge = asm::TailEdge::Return { return_values: vec![] };
    vec![ValueFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge: dummy_tail_edge },
    }]
}
