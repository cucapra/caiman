// AST

use crate::error::Info;
use super::schedulable;

pub type Var = String;

#[derive(Clone, Debug)]
pub enum StmtKind//<S> 
{
    Expr(Var, Vec<schedulable::SubExpr>, schedulable::FullExpr),
}
pub type Stmt<S> = (S, StmtKind);//<S>);

pub type ParsedStmt = Stmt<Info>;
pub type ParsedProgram = Vec<ParsedStmt>;
