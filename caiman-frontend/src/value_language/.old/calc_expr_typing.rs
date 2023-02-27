// Factors out code between type-checking and elaboration

use crate::error::{HasInfo, Info};
use crate::value_language::ast::*;
use crate::value_language::check::SemanticError;
use crate::value_language::typing;
use crate::value_language::typing::{Context, ExprType, FunctionContext, Type};

fn check(b: bool, i: Info, e: SemanticError) -> Result<(), (Info, SemanticError)>
{
    if b
    {
        Ok(())
    }
    else
    {
        Err((i, e))
    }
}

pub fn context_add_vwt(
    info: Info,
    context: &mut Context,
    (x, t): (&String, &Type),
) -> Result<(), (Info, SemanticError)>
{
    context.add(x.to_string(), *t).map_err(|e| (info, e))
}

pub fn context_get(info: Info, context: &Context, x: &Var) -> Result<Type, (Info, SemanticError)>
{
    match context.get_type(x)
    {
        Ok(t) => Ok(t),
        Err(e) => Err((info, e)),
    }
}

pub fn expr_type<E: HasInfo>(
    expr: &Expr<E>,
    context: &Context,
    function_context: &FunctionContext,
) -> Result<ExprType, (Info, SemanticError)>
{
    let (meta, expr_kind) = expr;
    let info = meta.info();
    match expr_kind
    {
        ExprKind::Num(n) => Ok(expr_type_of_num(n)),
        ExprKind::Bool(_) => Ok(ExprType::Ordinary(Type::Bool)),
        ExprKind::Var(x) => context_get(info, context, x).map(|t| ExprType::Ordinary(t)),
        ExprKind::Unit => Ok(ExprType::Ordinary(Type::I32)),
        ExprKind::Binop(bop, e1, e2) =>
        {
            let t1 = expr_type(e1, context, function_context)?;
            let t2 = expr_type(e2, context, function_context)?;
            binop_type(info, bop, t1, t2)
        },
        ExprKind::Unop(unop, e) => unop_type(info, unop, expr_type(e, context, function_context)?),
        ExprKind::If(e1, e2, e3) =>
        {
            panic!("TODO: expr kind if type")
        },
        ExprKind::Call(f, args) => call_type(info, context, function_context, f, args),
        ExprKind::Labeled(_, e) => expr_type(e, context, function_context),
        ExprKind::Tuple(es) => panic!("TODO tuple checking"),
        ExprKind::IRNode(_n, _es) => panic!("TODO ir node check"),
    }
}

pub fn call_type<E: HasInfo>(
    info: Info,
    context: &Context,
    function_context: &FunctionContext,
    f: &str,
    args: &Vec<Expr<E>>,
) -> Result<ExprType, (Info, SemanticError)>
{
    let (param_ts, ret_r) = function_context.get(f.to_string()).map_err(|e| (info, e))?;
    check(
        param_ts.len() == args.len(),
        info,
        SemanticError::WrongNumberArgs(param_ts.len(), args.len()),
    )?;
    for (arg, t) in args.iter().zip(param_ts.iter())
    {
        let arg_expr_t = expr_type(arg, context, function_context)?;
        check(arg_expr_t.is_subtype_of(*t), info, SemanticError::TypeMismatch(*t, arg_expr_t))?;
    }
    Ok(ExprType::Ordinary(ret_r))
}

fn expr_type_of_num(num: &str) -> ExprType
{
    if num.contains(".")
    {
        ExprType::Float
    }
    else if num.contains("-")
    {
        ExprType::NegativeInteger
    }
    else
    {
        ExprType::PositiveInteger
    }
}

fn binop_type(
    info: Info,
    bop: &Binop,
    t1: ExprType,
    t2: ExprType,
) -> Result<ExprType, (Info, SemanticError)>
{
    let check_num =
        |t: ExprType| check(t.is_number(), info.clone(), SemanticError::WrongBinop(t, *bop));
    let check_bool =
        |t: ExprType| check(t.is_bool(), info.clone(), SemanticError::WrongBinop(t, *bop));
    let check_compatible = || check(t1.compatible(t2), info, SemanticError::Incompatible(t1, t2));
    let bop_body = |f: &dyn Fn(ExprType) -> Result<(), (Info, SemanticError)>, ret: ExprType| {
        f(t1)?;
        f(t2)?;
        check_compatible()?;
        Ok(ret)
    };
    match bop
    {
        Binop::Plus | Binop::Minus | Binop::Mult | Binop::Div =>
        {
            bop_body(&check_num, typing::min_expr_type(t1, t2))
        },
        Binop::Equals => bop_body(&check_num, ExprType::Ordinary(Type::Bool)),
        Binop::And | Binop::Or => bop_body(&check_bool, typing::min_expr_type(t1, t2)),
    }
}

fn unop_type(info: Info, unop: &Unop, t: ExprType) -> Result<ExprType, (Info, SemanticError)>
{
    match unop
    {
        Unop::Not =>
        {
            check(t.is_bool(), info, SemanticError::WrongUnop(t, *unop))?;
            Ok(ExprType::Ordinary(Type::Bool))
        },
    }
}
