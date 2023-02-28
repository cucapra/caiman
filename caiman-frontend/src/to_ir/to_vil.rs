use super::vil;
use crate::scheduling_language::schedulable;
use crate::value_language::ast::{self, ExprKind};
use crate::value_language::typing::Type;

#[derive(Clone)]
struct ExprPath
{
    pub path: Vec<schedulable::Intermediate>,
}

impl ExprPath
{
    fn new() -> Self { ExprPath { path: Vec::new() } }

    fn add(&self, new_elt: schedulable::Intermediate) -> Self
    {
        let mut copy = self.clone();
        copy.path.push(new_elt);
        copy
    }
}

fn add_expr_stmt(
    expr: &ast::TypedExpr,
    stmts: &mut Vec<vil::Stmt<usize>>,
    path: ExprPath,
    root_var: &str,
) -> usize
{
    use schedulable::Intermediate::*;
    let ((info, expr_type), expr_kind) = expr;
    let vil_expr = match expr_kind
    {
        ExprKind::Var(x) => vil::Expr::Var(x.to_string()),
        ExprKind::Num(n) =>
        {
            let value = match expr_type
            {
                Type::I32 => vil::Value::I64(n.parse::<i64>().unwrap()),
                _ => panic!("Invalid number type bypassed typechecking at {:?}", info),
            };
            vil::Expr::Value(value)
        },
        ExprKind::Bool(b) => vil::Expr::Value(vil::Value::U64(if *b { 1 } else { 0 })),
        ExprKind::If(guard, e_true, e_false) =>
        {
            let guard_idx = add_expr_stmt(guard, stmts, path.add(IfGuard), root_var);
            let true_idx = add_expr_stmt(e_true, stmts, path.add(IfTrue), root_var);
            let false_idx = add_expr_stmt(e_false, stmts, path.add(IfFalse), root_var);
            vil::Expr::If(guard_idx, true_idx, false_idx)
        },
        _ => todo!(),
    };
    let vil_stmt = vil::Stmt {
        expr: vil_expr,
        expr_type: *expr_type,
        path_from_root: path.path,
        root_var: root_var.to_string(),
        info: *info,
    };
    stmts.push(vil_stmt);
    stmts.len() - 1
}

pub fn value_ast_to_vil(value_ast: &ast::TypedProgram) -> vil::Program
{
    let mut vil_stmts: Vec<vil::Stmt<usize>> = Vec::new();
    for (_, stmt_kind) in value_ast.iter()
    {
        match stmt_kind
        {
            ast::StmtKind::Let((x, _), e) =>
            {
                add_expr_stmt(e, &mut vil_stmts, ExprPath::new(), x);
            },
            _ => todo!(),
        }
    }
    vil::Program { stmts: vil_stmts }
}
