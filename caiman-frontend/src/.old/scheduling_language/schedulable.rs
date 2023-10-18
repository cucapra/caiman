// This module holds the "schedulable" types, which represent
// each expression and subexpression type possible in the value
// language.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubExpr
{
    // If
    IfGuard,
    IfTrue,
    IfFalse,
    // TODO: others
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FullExpr
{
    Primitive,
    IfComplete,
}
