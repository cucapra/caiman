// AST

use crate::error::Info;

pub type Var = String;

#[derive(Debug)]
pub enum StmtKind//<S> 
{
    Var(Var),
}
pub type Stmt<S> = (S, StmtKind);//<S>);

pub type ParsedStmt = Stmt<Info>;
pub type ParsedProgram = Vec<ParsedStmt>;
