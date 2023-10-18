// AST
use crate::value_language::typing::Type;
use crate::error::{Info, HasInfo};
use crate::spec;

pub type Var = String;

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
    If(Box<Expr<E>>, Box<Expr<E>>, Box<Expr<E>>),
    Unit,
    Binop(Binop, Box<Expr<E>>, Box<Expr<E>>),
    Unop(Unop, Box<Expr<E>>),
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

pub type TypedExpr = Expr<(Info, Type)>;
pub type TypedStmt = Stmt<Info, (Info, Type)>;
pub type TypedProgram = Program<Info, (Info, Type)>;

impl HasInfo for Info
{
    fn info(&self) -> Info { *self }
}


