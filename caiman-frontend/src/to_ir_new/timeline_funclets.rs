use super::funclet_util::make_asm_funclet;
use super::typing::TypingContext;
use crate::syntax::ast;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct TimelineFunclet(pub asm::Funclet);

pub fn lower_timeline_funclets(
    typing_ctx: &mut TypingContext,
    program: &ast::Program,
) -> Vec<TimelineFunclet>
{
    program
        .iter()
        .filter_map(|(_info, decl)| match decl {
            ast::DeclKind::TimelineFunclet { name, input, output, statements } => {
                Some(lower_timeline_funclet(typing_ctx, name, input, output, statements))
            },
            _ => None,
        })
        .collect()
}

fn lower_timeline_funclet(
    typing_ctx: &mut TypingContext,
    name: &String,
    input: &Vec<ast::Arg<ast::timeline::Type>>,
    output: &ast::timeline::Type,
    statements: &Vec<ast::timeline::Stmt>,
) -> TimelineFunclet
{
    let mut returned_variable = None;
    let /*mut*/ nodes: Vec<Option<asm::NamedNode>> = Vec::new();
    for (_stmt_info, stmt_kind) in statements.iter() {
        match stmt_kind {
            ast::timeline::StmtKind::Return(x) => returned_variable = Some(asm::NodeId(x.clone())),
        }
    }

    let tail_edge = asm::TailEdge::Return { return_values: Some(vec![returned_variable]) };

    let mut convert_type = |t: &ast::timeline::Type| asm::TypeId::Local(typing_ctx.convert_and_add_timeline_type(t.clone()));
    let header = 
        asm::FuncletHeader {
            args: input.iter().map(|(x, t)| {
                asm::FuncletArgument {
                    name: Some(asm::NodeId(x.clone())),
                    typ: convert_type(t),
                    tags: Vec::new(),
                }
            }).collect(),
            ret: vec![asm::FuncletArgument { name: None, typ: convert_type(output), tags: Vec::new() }],
            name: asm::FuncletId(name.clone()),
            binding: asm::FuncletBinding::None,
        };

    TimelineFunclet(make_asm_funclet(ir::FuncletKind::Timeline, header, nodes, tail_edge))
}
