// Elaborate the parsed AST by giving each expression
// or statement type metadata
//
// Expression tree is assumed to have already been checked!
use crate::value_language::ast::*;
use crate::value_language::typing::{Type, ExprType, Context, FunctionContext};

/*pub struct TypeMetadata 
{
    info: Info,
}

pub type TypedExpr = Expr<Info>;
pub type TypedStmt = Stmt<Info, Info>;
pub type TypedProgram = Program<Info, Info>;*/
