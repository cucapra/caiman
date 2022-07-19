// AST

pub type Var = String;

// Negative numbers are parsed as negative at a later stage
// because we store all numbers as Strings here
#[derive(Debug)]
pub enum Unop 
{
    Not,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Statement
{
    If(Exp, Vec<Statement>),
    While(Exp, Vec<Statement>),
    Print(Exp),
    Let(bool, Var, Exp),
    Assign(Var, Exp),
    Function(Var, Vec<Var>, Vec<Statement>),
    Call(Var, Vec<Exp>),
    Return(Exp),
}

pub type Program = Vec<Statement>;

// Factory
pub fn make_binop(e1: Exp, b: Binop, e2: Exp) -> Exp
{
    Exp::Binop(b, Box::new(e1), Box::new(e2))
}
