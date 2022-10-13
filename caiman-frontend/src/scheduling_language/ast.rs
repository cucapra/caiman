// AST

// Copied from value language:

pub type Var = String;

#[derive(Clone, Copy, Debug)]
pub struct Info {
    // (Line, Column) and (Beginning, Ending)
    pub location: ((usize, usize), (usize, usize)),
}

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
