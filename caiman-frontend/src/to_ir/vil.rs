// The "Value Intermediate Language" is an intermediate representation
// of the value language whose statements' subexpressions are broken up into
// individualized, labelable pieces. The intention behind this is to make it very
// easily compatible with the scheduling language.

use crate::scheduling_language::schedulable;
use crate::value_language::typing::Type;
use crate::error;

#[derive(Debug, Clone)]
pub enum Value
{
    // These are the only values currently available in
    // caiman's IR (as of my writing this)
    I64(i64),
    U64(u64),
    I32(i32),
}

// A bit of a clone of value language expr
#[derive(Debug, Clone)]
pub enum Expr<NodeIndex>
{
    Value(Value),
    //Var(String),
    If(NodeIndex, NodeIndex, NodeIndex),
}

#[derive(Debug, Clone)]
pub struct Stmt<NodeIndex>
{
    pub expr: Expr<NodeIndex>,
    pub expr_type: Type,
    pub path_from_root: Vec<schedulable::SubExpr>,
    pub root_var: String,
    pub info: error::Info,
}

pub struct Program
{
    pub stmts: Vec<Stmt<usize>>,
}

impl std::fmt::Debug for Program
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        for (i, stmt) in self.stmts.iter().enumerate()
        {
            write!(f, "{}: {}", i, stmt.root_var)?;
            for node in stmt.path_from_root.iter()
            {
                write!(f, ".{:?}", node)?;
            }
            write!(f, " = {:?} : {:?}\n", stmt.expr, stmt.expr_type)?;
        }
        Ok(())
    }
}

pub fn schedulable_of_vil_expr<T>(e: &Expr<T>) -> schedulable::FullExpr
{
    match e 
    {
        Expr::Value(_) => schedulable::FullExpr::Primitive,
        Expr::If(_, _, _) => schedulable::FullExpr::IfComplete,
    }
}

