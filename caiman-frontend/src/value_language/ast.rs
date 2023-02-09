// AST
use crate::value_language::typing::Type;
use crate::error::{Info, HasInfo};
use crate::spec;

pub type Var = String;

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
    Unit,
    Binop(Binop, Box<Expr<E>>, Box<Expr<E>>),
    Unop(Unop, Box<Expr<E>>),
    If(Box<Expr<E>>, Box<Expr<E>>, Box<Expr<E>>),
    Call(Var, Vec<Expr<E>>),
    Labeled(Var, Box<Expr<E>>),
    Tuple(Vec<Expr<E>>),
    IRNode(spec::nodes::FunctionalExprNodeKind, Vec<Expr<E>>),
}
pub type Expr<E> = (E, ExprKind<E>);

pub type VarWithType = (Var, Type);

#[derive(Debug, Clone)]
pub enum StmtKind<S, E>
{
    Let(VarWithType, Expr<E>),
    LetFunction(Var, Vec<VarWithType>, Type, Vec<Stmt<S, E>>, Expr<E>),
    //LetMulti(Vec<VarWithType>, Vec<Expr<E>>),
}
pub type Stmt<S, E> = (S, StmtKind<S, E>);

pub type Program<S, E> = Vec<Stmt<S, E>>;

pub type ParsedExpr = Expr<Info>;
pub type ParsedStmt = Stmt<Info, Info>;
pub type ParsedProgram = Program<Info, Info>;

impl HasInfo for Info
{
    fn info(&self) -> Info { *self }
}


