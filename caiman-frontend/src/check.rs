// Does type checking, but not elaboration. There will be a 
// separate elaboration file that assumes tree has been 
// checked beforehand.

use crate::error;
use crate::ast::*;
use crate::typing::{Context, FunctionContext, Type, ExprType};
use crate::typing;

pub enum SemanticError
{
    FunctionNameCollision(String),
    TypeMismatch(Type, ExprType),
    UnboundVariable(String),
    UnboundFunction(String),
    Incompatible(ExprType, ExprType),
    WrongBinop(ExprType, Binop),
    WrongUnop(ExprType, Unop),
    ReturnTypeMismatch(Type, ExprType),
    WrongNumberArgs(usize, usize),
}

type InfoError = (Info, SemanticError);

#[derive(Clone)]
struct CheckingState
{
    context: Context,
    function_context: FunctionContext,
}

fn check(b: bool, i: Info, e: SemanticError) -> Result<(), InfoError>
{
    if b { Ok(()) } else { Err((i, e)) }
}

pub fn check_program<S, E>(
    program: &Program<S, E>,
) -> Result<(), error::LocalError>
where
    S: HasInfo,
    E: HasInfo,
{
    let mut st = CheckingState { 
        context: Context::new(), 
        function_context: FunctionContext::new(), 
    };
    check_block(program, &mut st)
        .map_err(|(info, e)| error::LocalError {
            kind: error::ErrorKind::Semantic(e),
            location: error::ErrorLocation::Double(info.location),
        })
}

fn check_block<S, E>(
    block: &Vec<Stmt<S, E>>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
where
    S: HasInfo,
    E: HasInfo,
{
    for stmt in block.iter()
    {
        check_stmt(stmt, st)?;
    }
    Ok(())
}


fn check_stmt<S, E>(
    stmt: &Stmt<S, E>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
where
    S: HasInfo,
    E: HasInfo,
{
    let (metadata, stmt_kind) = stmt;
    let info = metadata.info();
    match stmt_kind
    {
        StmtKind::Let((is_mut, x, t), expr) => {
            context_add_vwt(info, &mut st.context, (is_mut, x, t))?;
            let expr_t = expr_type(expr, st)?;
            check(
                expr_t.is_subtype_of(*t),
                info,
                SemanticError::TypeMismatch(*t, expr_t),
            )
        },
        StmtKind::Function(f, params, ret_t, block, ret_expr) => {
            // XXX is a clone too slow?
            let mut func_st = st.clone();
            for (m, x, t) in params.iter() 
            {
                context_add_vwt(info, &mut func_st.context, (m, x, t))?;
            }

            check_block(block, &mut func_st)?;

            let ret_expr_t = expr_type(ret_expr, &mut func_st)?;
            check(
                ret_expr_t.is_subtype_of(*ret_t),
                info,
                SemanticError::ReturnTypeMismatch(*ret_t, ret_expr_t),
            )?;

            // XXX if we ever want to add recursion, then 
            // this addition should be moved to the *beginning*
            // of this block, right before the clone
            let param_ts = params.iter().map(|(_, _, t)| *t).collect();
            st.function_context.add(f.to_string(), param_ts, *ret_t)
                .map_err(|e| (info, e))
        },
        StmtKind::If(guard, block)
        | StmtKind::While(guard, block) => {
            check_guard_and_block(guard, block, st)
        },
        StmtKind::Assign(x, e) => {
            let var_t = context_get(info, &st.context, x)?;
            let expr_t = expr_type(e, st)?;
            check(
                expr_t.is_subtype_of(var_t),
                info,
                SemanticError::TypeMismatch(var_t, expr_t),
            )
        },
        StmtKind::Call(f, args) => call_type(info, st, f, args).map(|_| ()),
        StmtKind::Print(_) => panic!("No printing sorry :("),
    }
}

fn expr_type<E>(
    expr: &Expr<E>,
    st: &mut CheckingState,
) -> Result<ExprType, InfoError>
where
    E: HasInfo,
{
    let (meta, expr_kind) = expr;
    let info = meta.info();
    match expr_kind
    {
        ExprKind::Num(n) => Ok(expr_type_of_num(n)),
        ExprKind::Bool(_) => Ok(ExprType::Ordinary(Type::Bool)),
        ExprKind::Var(x) => {
            context_get(info, &st.context, x).map(|t| ExprType::Ordinary(t))
        },
        ExprKind::Input() => Ok(ExprType::Any),
        ExprKind::Binop(bop, e1, e2) => {
            let t1 = expr_type(e1, st)?;
            let t2 = expr_type(e2, st)?;
            check_binop(info, bop, t1, t2)
        },
        ExprKind::Unop(unop, e) => check_unop(info, unop, expr_type(e, st)?),
        ExprKind::Call(f, args) => call_type(info, st, f, args),
        ExprKind::Labeled(_, e) => expr_type(e, st),
    }
}

fn check_binop(
    info: Info,
    bop: &Binop, 
    t1: ExprType, 
    t2: ExprType
) -> Result<ExprType, InfoError>
{
    let check_num = |t: ExprType| {
        check(t.is_number(), info.clone(), SemanticError::WrongBinop(t, *bop))
    };
    let check_bool = |t: ExprType| {
        check(t.is_bool(), info.clone(), SemanticError::WrongBinop(t, *bop))
    };
    let check_compatible = || {
        check(t1.compatible(t2), info, SemanticError::Incompatible(t1, t2))
    };
    let bop_body = 
        |f: &dyn Fn(ExprType) -> Result<(), InfoError>, ret: ExprType| { 
            f(t1)?; f(t2)?; check_compatible()?; Ok(ret)
        };
    match bop
    {
        Binop::Plus | Binop::Minus | Binop::Mult | Binop::Div => {
            bop_body(&check_num, typing::min_expr_type(t1, t2))
        },
        Binop::Equals => {
            bop_body(&check_num, ExprType::Ordinary(Type::Bool))
        },
        Binop::And | Binop::Or => {
            bop_body(&check_bool, typing::min_expr_type(t1, t2))
        },
    }
}

fn check_unop(
    info: Info,
    unop: &Unop, 
    t: ExprType, 
) -> Result<ExprType, InfoError>
{
    match unop
    {
        Unop::Not => {
            check(t.is_bool(), info, SemanticError::WrongUnop(t, *unop))?;
            Ok(ExprType::Ordinary(Type::Bool))
        },
    }
}

// For if and while
fn check_guard_and_block<S, E>(
    guard: &Expr<E>,
    block: &Vec<Stmt<S, E>>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
where
    S: HasInfo,
    E: HasInfo,
{
    let (meta, _) = guard;
    let info = meta.info();
    let guard_t = expr_type(guard, st)?;
    check(
        guard_t.is_bool(),
        info,
        SemanticError::TypeMismatch(Type::Bool, guard_t),
    )?;
    // XXX is clone too slow?
    let mut block_st = st.clone();
    check_block(block, &mut block_st)
}


fn call_type<E>(
    info: Info, 
    st: &mut CheckingState, 
    f: &str,
    args: &Vec<Expr<E>>,
) -> Result<ExprType, InfoError>
where
    E: HasInfo,
{
    let (param_ts, ret_r) = st.function_context.get(f.to_string())
        .map_err(|e| (info, e))?;
    check(
        param_ts.len() == args.len(), 
        info, 
        SemanticError::WrongNumberArgs(param_ts.len(), args.len()),
    )?;
    for (arg, t) in args.iter().zip(param_ts.iter())
    {
        let arg_expr_t = expr_type(arg, st)?;
        check(
            arg_expr_t.is_subtype_of(*t),
            info,
            SemanticError::TypeMismatch(*t, arg_expr_t),
        )?;
    }
    Ok(ExprType::Ordinary(ret_r))
}


fn context_add_vwt(
    info: Info,
    context: &mut Context,
    (is_mut, x, t): (&bool, &String, &Type),
) -> Result<(), InfoError>
{
    context.add(x.to_string(), *t, *is_mut).map_err(|e| (info, e))
}

fn context_get(
    info: Info, 
    context: &Context, 
    x: &Var,
) -> Result<Type, InfoError>
{
    match context.get_type(x)
    {
        Ok(t) => Ok(t),
        Err(e) => Err((info, e)),
    }
}

fn expr_type_of_num(num: &str) -> ExprType
{
    if num.contains(".") { ExprType::Float }
    else if num.contains("-") { ExprType::NegativeInteger }
    else { ExprType::PositiveInteger }
}

