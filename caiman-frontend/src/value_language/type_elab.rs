use super::ast;
use crate::error::Info;
use super::typing;
use super::typing::Context;
use typing::Type;

/// Precondition: parsed_prog has been type-checked already.
pub fn elaborate_program(prog: &ast::ParsedProgram, ctx: &Context) -> ast::TypedProgram
{
    prog.iter().map(|stmt| elaborate_stmt(stmt, ctx)).collect()
}

fn elaborate_stmt(parsed_stmt: &ast::ParsedStmt, ctx: &Context) -> ast::TypedStmt
{
    let (info, kind) = parsed_stmt;
    use ast::StmtKind::*;
    match kind
    {
        Let((x, t), e) =>
        {
            let e_typed = elaborate_expr(e, ctx, *t);
            let stmt_kind_typed = Let((x.to_string(), *t), e_typed);
            (*info, stmt_kind_typed)
        },
        _ => todo!(),
    }
}

fn elaborate_expr(
    parsed_expr: &ast::ParsedExpr,
    ctx: &Context,
    outer_type: typing::Type,
) -> ast::TypedExpr
{
    let (info, kind) = parsed_expr;
    let inferred_type = typing::type_of_expr(parsed_expr, ctx).unwrap();
    let t = match inferred_type
    {
        typing::InferredType::Ordinary(t) => t,
        _ => outer_type,
    };
    let annot = |ek: ast::ExprKind<(Info, Type)>| ((*info, t), ek);
    // Extremely boilerplate
    use ast::ExprKind::*;
    match kind
    {
        Var(x) => annot(Var(x.clone())),
        Num(n) => annot(Num(n.clone())),
        Bool(b) => annot(Bool(*b)),
        If(e1, e2, e3) => {
            let e1_typed = Box::new(elaborate_expr(e1, ctx, outer_type));
            let e2_typed = Box::new(elaborate_expr(e2, ctx, outer_type));
            let e3_typed = Box::new(elaborate_expr(e3, ctx, outer_type));
            annot(If(e1_typed, e2_typed, e3_typed))
        },
        _ => todo!(),
    }
}
