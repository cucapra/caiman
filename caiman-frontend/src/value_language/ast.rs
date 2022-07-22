// AST
use crate::value_language::typing;

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
pub enum Exp
{
    Var(Var),
    Num(String),
    Bool(bool),
    Input(),
    Binop(Binop, Box<Exp>, Box<Exp>),
    Unop(Unop, Box<Exp>),
    Call(Var, Vec<Exp>),
    Labeled(Var, Box<Exp>),
}

// The two types of statement are so that one can contain ordinary
// expresions (Statement) while the other (AnnotatedStatement) contains the 
// types of its underlying expressions as well

#[derive(Debug, Clone)]
pub enum Statement<E>
{
    If(E, Vec<Statement<E>>),
    While(E, Vec<Statement<E>>),
    Print(E),
    Let(bool, Var, E),
    Assign(Var, E),
    Function(Var, Vec<Var>, Vec<Statement<E>>),
    Call(Var, Vec<E>),
    Return(E),
}

pub type ParsedStatement = Statement<Exp>;

pub type CheckedStatement = Statement<(typing::Type, Exp)>;

pub type Program = Vec<ParsedStatement>;

pub type CheckedProgram = Vec<CheckedStatement>;

// Factory
pub fn make_binop(e1: Exp, b: Binop, e2: Exp) -> Exp
{
    Exp::Binop(b, Box::new(e1), Box::new(e2))
}
