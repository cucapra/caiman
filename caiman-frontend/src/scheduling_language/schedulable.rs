// This module holds the "schedulable" types, which represent
// each expression and subexpression type possible in the value
// language.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intermediate
{
    // If
    IfGuard,
    IfTrue,
    IfFalse,
    // TODO: others
}

#[derive(Debug, Clone)]
pub enum Final
{
    Primitive,
    Var,
    IfComplete,
}
