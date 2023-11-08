use super::funclet_util::make_asm_funclet;
use super::typing::TypingContext;
use crate::parse::ast;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct SpatialFunclet(pub asm::Funclet);

pub fn lower_spatial_funclets(
    typing_ctx: &mut TypingContext,
    program: &ast::Program,
) -> Vec<SpatialFunclet> {
    program
        .iter()
        .filter_map(|(_info, decl)| match decl {
            ast::DeclKind::SpatialFunclet {
                name,
                input,
                output,
                statements,
            } => Some(lower_spatial_funclet(
                typing_ctx, name, input, output, statements,
            )),
            _ => None,
        })
        .collect()
}

fn lower_spatial_funclet(
    typing_ctx: &mut TypingContext,
    name: &String,
    input: &Vec<ast::Arg<ast::spatial::Type>>,
    output: &ast::spatial::Type,
    statements: &Vec<ast::spatial::Stmt>,
) -> SpatialFunclet {
    let mut returned_variable = None;
    let /*mut*/ nodes: Vec<Option<asm::NamedNode>> = Vec::new();
    for (_stmt_info, stmt_kind) in statements.iter() {
        match stmt_kind {
            ast::spatial::StmtKind::Return(x) => returned_variable = Some(asm::NodeId(x.clone())),
        }
    }

    let tail_edge = asm::TailEdge::Return {
        return_values: Some(vec![returned_variable]),
    };

    let mut convert_type = |t: &ast::spatial::Type| {
        asm::TypeId::Local(typing_ctx.convert_and_add_spatial_type(t.clone()))
    };
    let header = asm::FuncletHeader {
        args: input
            .iter()
            .map(|(x, t)| asm::FuncletArgument {
                name: Some(asm::NodeId(x.clone())),
                typ: convert_type(t),
                tags: Vec::new(),
            })
            .collect(),
        ret: vec![asm::FuncletArgument {
            name: None,
            typ: convert_type(output),
            tags: Vec::new(),
        }],
        name: asm::FuncletId(name.clone()),
        binding: asm::FuncletBinding::None,
    };

    SpatialFunclet(make_asm_funclet(
        ir::FuncletKind::Spatial,
        header,
        nodes,
        tail_edge,
    ))
}
