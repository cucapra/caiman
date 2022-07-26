// AST
use crate::value_language::typing::Type;

pub type Var = String;

// (Line, Column) and (Beginning, Ending)
pub type Info = ((usize, usize), (usize, usize));

pub trait HasInfo
{
    fn info(&self) -> Info;
}

// Negative numbers are parsed as negative at a later stage
// because we store all numbers as Strings here
#[derive(Debug, Clone, Copy)]
pub enum Unop 
{
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum Binop
{
    Plus,
    Minus,
    Mult,
    Div,
    Equals,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum ExprKind<E>
{
    Var(Var),
    Num(String),
    Bool(bool),
    Input(),
    Binop(Binop, Box<Expr<E>>, Box<Expr<E>>),
    Unop(Unop, Box<Expr<E>>),
    Call(Var, Vec<Expr<E>>),
    Labeled(Var, Box<Expr<E>>),
}
pub type Expr<E> = (E, ExprKind<E>);

// TODO:
// Add type annotations!
// It's a construct that occurs in two places: let and function.
// Will need to expand the types file as well
// Probably don't need reference or array types yet because
//   no way to initialize them

pub type VarWithType = (Var, Type);

#[derive(Debug, Clone)]
pub enum StmtKind<S, E>
{
    If(Expr<E>, Vec<Stmt<S, E>>),
    While(Expr<E>, Vec<Stmt<S, E>>),
    Print(Expr<E>),
    Let(bool, VarWithType, Expr<E>),
    Assign(Var, Expr<E>),
    Function(Var, Vec<VarWithType>, Type, Vec<Stmt<S, E>>),
    Call(Var, Vec<Expr<E>>),
    Return(Expr<E>),
}
pub type Stmt<S, E> = (S, StmtKind<S, E>);

pub type Program<S, E> = Vec<Stmt<S, E>>;

pub type ParsedExpr = Expr<Info>;
pub type ParsedStmt = Stmt<Info, Info>;
pub type ParsedProgram = Program<Info, Info>;

//pub type CheckedExpr = Expr<(Info, Type)>;
//pub type CheckedStmt = Stmt<Info, (Info, Type)>;
//pub type CheckedProgram = Vec<CheckedStmt>;

impl HasInfo for Info
{
    fn info(&self) -> Info { *self }
}

