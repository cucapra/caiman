use super::vil;
use crate::scheduling_language::schedulable;
use crate::value_language::ast::{self, ExprKind};
use crate::value_language::typing::Type;
use std::collections::HashMap;

#[derive(Clone)]
struct ExprPath
{
    pub path: Vec<schedulable::SubExpr>,
}

impl ExprPath
{
    fn new() -> Self { ExprPath { path: Vec::new() } }

    fn add(&self, new_elt: schedulable::SubExpr) -> Self
    {
        let mut copy = self.clone();
        copy.path.push(new_elt);
        copy
    }
}

struct IndexContext
{
    var_index: HashMap<String, usize>,
}

impl IndexContext
{
    fn new() -> Self { IndexContext { var_index: HashMap::new() } }

    fn add(&mut self, x: &str, u: usize)
    {
        if self.var_index.insert(x.to_string(), u).is_some()
        {
            panic!("Variable shadowing of {} bypassed typechecker", x);
        }
    }

    fn get(&self, x: &str) -> usize
    {
        self.var_index[x]
    }
}

fn add_expr_stmt(
    expr: &ast::TypedExpr,
    stmts: &mut Vec<vil::Stmt<usize>>,
    path: ExprPath,
    root_var: &str,
    ctx: &IndexContext,
) -> usize
{
    use schedulable::SubExpr::*;
    let ((info, expr_type), expr_kind) = expr;
    let vil_expr = match expr_kind
    {
        ExprKind::Var(x) =>
        {
            // Use of a variable doesn't really represent an operation. You just use
            // a previous operation represented by the variable. We should just point
            // to that variable here; no need to continue on making a new statement.
            return ctx.get(x);
        },
        ExprKind::Num(n) =>
        {
            let value = match expr_type
            {
                Type::I32 => vil::Value::I32(n.parse::<i32>().unwrap()),
                _ => panic!("Invalid number type bypassed typechecking at {:?}", info),
            };
            vil::Expr::Value(value)
        },
        ExprKind::Bool(b) => vil::Expr::Value(vil::Value::U64(if *b { 1 } else { 0 })),
        ExprKind::If(guard, e_true, e_false) =>
        {
            let guard_idx = add_expr_stmt(guard, stmts, path.add(IfGuard), root_var, ctx);
            let true_idx = add_expr_stmt(e_true, stmts, path.add(IfTrue), root_var, ctx);
            let false_idx = add_expr_stmt(e_false, stmts, path.add(IfFalse), root_var, ctx);
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
    let mut ctx = IndexContext::new();
    for (_, stmt_kind) in value_ast.iter()
    {
        match stmt_kind
        {
            ast::StmtKind::Let((x, _), e) =>
            {
                let e_index = add_expr_stmt(e, &mut vil_stmts, ExprPath::new(), x, &ctx);
                ctx.add(x, e_index);
            },
            _ => todo!(),
        }
    }
    vil::Program { stmts: vil_stmts }
}
