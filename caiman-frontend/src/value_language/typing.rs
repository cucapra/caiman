use super::ast::{Expr, ExprKind};
use super::check::{SemanticError, SemanticInfoError};
use crate::error::HasInfo;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type
{
    I32,
    //F32,
    Bool,
    //Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferredType
{
    Ordinary(Type),
    PositiveInteger,
    NegativeInteger,
    Float,
    Any,
}

type Var = String;
pub struct Context
{
    var_map: HashMap<Var, Type>,
}

impl Context
{
    pub fn new() -> Self { Context { var_map: HashMap::new() } }

    pub fn add(&mut self, x: &str, t: Type) -> Result<(), SemanticError>
    {
        match self.var_map.insert(x.to_string(), t)
        {
            Some(_) => Err(SemanticError::NameCollision(x.to_string())),
            None => Ok(()),
        }
    }

    pub fn get(&self, x: &str) -> Result<Type, SemanticError>
    {
        match self.var_map.get(x)
        {
            Some(t_ref) => Ok(*t_ref),
            None => Err(SemanticError::UnboundVariable(x.to_string())),
        }
    }
}

pub fn type_of_expr<E: HasInfo>(
    e: &Expr<E>,
    ctx: &Context,
) -> Result<InferredType, SemanticInfoError>
{
    use ExprKind::*;
    let (metadata, e_kind) = e;
    let info = metadata.info();
    let ord = |t| -> Result<InferredType, SemanticInfoError> { Ok(InferredType::Ordinary(t)) };
    let add_info = |e: SemanticError| (info, e);
    match e_kind
    {
        Num(s) => Ok(infer_num_type(s)),
        Bool(_) => ord(Type::Bool),
        Var(x) => ord(ctx.get(x).map_err(add_info)?),
        If(e_guard, e_true, e_false) =>
        {
            let t_guard = type_of_expr(e_guard, ctx)?;
            let guard_info = e_guard.0.info();
            expect_type(Type::Bool, t_guard).map_err(|e| (guard_info, e))?;
            let t_true = type_of_expr(e_true, ctx)?;
            let t_false = type_of_expr(e_false, ctx)?;
            merge_inferred_types(t_true, t_false).map_err(add_info)
        },
        _ => todo!(),
    }
}

pub fn expect_type(expected: Type, actual: InferredType) -> Result<(), SemanticError>
{
    use InferredType::*;
    use Type::*;
    let err = || Err(SemanticError::TypeMismatch(expected, actual));
    if let Ordinary(t) = actual
    {
        if expected == t
        {
            return Ok(());
        }
        return err();
    }
    match (expected, actual)
    {
        (I32, PositiveInteger) | (I32, NegativeInteger) | (I32, Any) => Ok(()),
        _ => err(),
    }
}

fn merge_inferred_types(
    it1: InferredType,
    it2: InferredType,
) -> Result<InferredType, SemanticError>
{
    let err = || Err(SemanticError::Incompatible(it1, it2));
    if it1 == it2
    {
        return Ok(it1);
    }
    use InferredType::*;
    match (it1, it2)
    {
        (PositiveInteger, NegativeInteger) | (NegativeInteger, PositiveInteger) =>
        {
            Ok(NegativeInteger)
        },
        (Any, it) | (it, Any) => Ok(it),
        (Ordinary(t), it) | (it, Ordinary(t)) =>
        {
            expect_type(t, it)?;
            Ok(Ordinary(t))
        },
        _ => err(),
    }
}

fn infer_num_type(num_str: &str) -> InferredType
{
    if num_str.contains(".")
    {
        InferredType::Float
    }
    else if num_str.contains("-")
    {
        InferredType::NegativeInteger
    }
    else
    {
        InferredType::PositiveInteger
    }
}
