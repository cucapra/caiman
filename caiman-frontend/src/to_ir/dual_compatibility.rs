// This module is meant for matching up each scheduling statement with
// its VIL statement's respective index. In doing this, is also checks
// that the value and scheduling language files are compatible.

use super::error::{self, DualLocalError, ToIRError};
use super::vil;
use crate::scheduling_language::schedulable::{FullExpr, SubExpr};
use crate::scheduling_language::{self, ast};
use crate::value_language::typing;

pub struct MatchedScheduleStmt
{
    pub schedule_stmt: ast::ParsedStmt,
    pub vil_index: usize,
    pub vl_type: typing::Type,
}

pub fn match_vil_to_scheduling(
    vil_program: &vil::Program,
    scheduling_program: &ast::ParsedProgram,
) -> Result<Vec<MatchedScheduleStmt>, DualLocalError>
{
    let mut vil_with_index: Vec<(usize, &vil::Stmt<usize>)> =
        vil_program.stmts.iter().enumerate().collect();
    println!("VIL WITH INDEX: {:?}", vil_with_index);
    let mut matched_stmts: Vec<MatchedScheduleStmt> = Vec::new();
    for schedule_stmt in scheduling_program.iter()
    {
        let (info, schedule_stmt_kind) = schedule_stmt;
        match schedule_stmt_kind
        {
            ast::StmtKind::Expr(x, sub_exprs, full_expr) =>
            {
                let (vec_index, (vil_index, vil_stmt)) = vil_with_index
                    .iter()
                    .enumerate()
                    .find(|(_, (_, s))| is_compatible(s, x, sub_exprs, full_expr))
                    .ok_or(err_unknown_scheduling(x, sub_exprs, full_expr, info))?;
                matched_stmts.push(MatchedScheduleStmt {
                    schedule_stmt: schedule_stmt.clone(),
                    vil_index: *vil_index,
                    vl_type: vil_stmt.expr_type,
                });
                vil_with_index.remove(vec_index);
            },
        }
    }
    // Check that we matched every single vil statement (otherwise one is unscheduled!)
    if vil_with_index.len() > 0
    {
        let (_, stmt) = vil_with_index[0];
        let to_ir_error = ToIRError::ForgottenExpr(
            stmt.root_var.to_string(),
            stmt.path_from_root.clone(),
            vil::schedulable_of_vil_expr(&stmt.expr),
        );
        Err(error::make_error(to_ir_error, stmt.info))
    }
    else
    {
        Ok(matched_stmts)
    }
}

fn is_compatible(
    vil_stmt: &vil::Stmt<usize>,
    x: &str,
    sub_exprs: &Vec<SubExpr>,
    full_expr: &FullExpr,
) -> bool
{
    println!("comp: x is {} subexprs={:?} full={:?} and vil {:?}", x, sub_exprs, full_expr,vil_stmt);
    for i in 0..sub_exprs.len()
    {
        if vil_stmt.path_from_root[i] != sub_exprs[i]
        {
            println!("PATHFROMROOT");
            return false;
        }
    }
    let same_var = vil_stmt.root_var == x;
    let same_fullexprs = vil::schedulable_of_vil_expr(&vil_stmt.expr) == full_expr.clone();
    println!("SAMEVAR {}, SAME_FULLEXPRS {}", same_var, same_fullexprs);
    same_var && same_fullexprs
}

fn err_unknown_scheduling(
    x: &str,
    sub_exprs: &Vec<SubExpr>,
    full_expr: &FullExpr,
    info: &error::Info,
) -> DualLocalError
{
    error::make_error(
        ToIRError::UnknownScheduling(x.to_string(), sub_exprs.clone(), full_expr.clone()),
        *info,
    )
}
