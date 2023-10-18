use super::ast;
use super::typing;
use super::check;
use super::typing::Context;
use crate::error::LocalError;

/// Precondition: parsed_prog has been type-checked already.
pub fn elaborate_program(prog: &ast::ParsedProgram) -> Result<ast::TypedProgram, LocalError>
{
    let ctx = build_context(prog)?;
    check::check_program(prog, &ctx)?;
    Ok(prog.iter().map(|stmt| elaborate_stmt(stmt, &ctx)).collect())
}

fn build_context(prog: &ast::ParsedProgram) -> Result<Context, LocalError>
{
    let mut ctx = Context::new();
    use ast::StmtKind::*;
    for (info, kind) in prog
    {
        match kind
        {
            Let((x, t), _) =>
            {
                ctx.add(x, *t).map_err(|e| check::error_semantic_to_local((*info, e)))?
            },
            _ => (),
        }
    }
    Ok(ctx)
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
    let type_annotation = match inferred_type
    {
        typing::InferredType::Ordinary(t) => t,
        _ => outer_type,
    };
    let elab = |e| Box::new(elaborate_expr(e, ctx, outer_type));
    use ast::ExprKind::*;
    let typed_kind = match kind
    {
        Var(x) => Var(x.clone()),
        Num(n) => Num(n.clone()),
        Bool(b) => Bool(*b),
        If(e1, e2, e3) => If(elab(e1), elab(e2), elab(e3)),
        _ => todo!(),
    };
    ((*info, type_annotation), typed_kind)
}
