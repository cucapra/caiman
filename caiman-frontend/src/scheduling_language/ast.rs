// AST

use crate::error::Info;

pub type Var = String;

/*pub trait HasInfo
{
    fn info(&self) -> Info;
}

impl HasInfo for Info
{
    fn info(&self) -> Info { *self }
}*/

// End of copied section

#[derive(Debug)]
pub enum StmtKind//<S> 
{
    Var(Var),
}
pub type Stmt<S> = (S, StmtKind);//<S>);

pub type ParsedStmt = Stmt<Info>;
pub type ParsedProgram = Vec<ParsedStmt>;
