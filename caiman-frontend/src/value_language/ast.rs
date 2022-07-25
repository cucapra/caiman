// AST
use crate::value_language::typing::Type;

pub type Var = String;

// Negative numbers are parsed as negative at a later stage
// because we store all numbers as Strings here
#[derive(Debug, Clone)]
pub enum Unop 
{
    Not,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum StmtKind<S, E>
{
    If(Expr<E>, Vec<Stmt<S, E>>),
    While(Expr<E>, Vec<Stmt<S, E>>),
    Print(Expr<E>),
    Let(bool, Var, Expr<E>),
    Assign(Var, Expr<E>),
    Function(Var, Vec<Var>, Vec<Stmt<S, E>>),
    Call(Var, Vec<Expr<E>>),
    Return(Expr<E>),
}
pub type Stmt<S, E> = (S, StmtKind<S, E>);

// (Line, Column) and (Beginning, Ending)
pub type Info = ((usize, usize), (usize, usize));

pub type ParsedExpr = Expr<Info>;
pub type ParsedStmt = Stmt<Info, Info>;
pub type ParsedProgram = Vec<ParsedStmt>;

//pub type CheckedExp = Exp<Type>;
//pub type CheckedStmt = Stmt<(), Type>;
//pub type CheckedProgram = Vec<CheckedStmt>;

