// Does type checking, but not elaboration. There will be a
// separate elaboration file that assumes tree has been
// checked beforehand.

use crate::error;
use crate::error::{HasInfo, Info};
use crate::value_language::ast::*;
use crate::value_language::calc_expr_typing::{
    context_add_vwt, expr_type, /*call_type, context_get, */
};
use crate::value_language::typing::{Context, ExprType, FunctionContext, Type};

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
    if b
    {
        Ok(())
    }
    else
    {
        Err((i, e))
    }
}

pub fn check_program<S: HasInfo, E: HasInfo>(
    program: &Program<S, E>,
) -> Result<(), error::LocalError>
{
    let mut st =
        CheckingState { context: Context::new(), function_context: FunctionContext::new() };
    check_block(program, &mut st).map_err(|(info, e)| error::LocalError {
        kind: error::ErrorKind::Semantic(e),
        location: error::ErrorLocation::Double(info.location),
    })
}

fn check_block<S: HasInfo, E: HasInfo>(
    block: &Vec<Stmt<S, E>>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
{
    for stmt in block.iter()
    {
        check_stmt(stmt, st)?;
    }
    Ok(())
}

fn check_stmt<S: HasInfo, E: HasInfo>(
    stmt: &Stmt<S, E>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
{
    let (metadata, stmt_kind) = stmt;
    let info = metadata.info();
    match stmt_kind
    {
        StmtKind::Let((x, t), expr) =>
        {
            context_add_vwt(info, &mut st.context, (x, t))?;
            let expr_t = expr_type(expr, &st.context, &st.function_context)?;
            check(expr_t.is_subtype_of(*t), info, SemanticError::TypeMismatch(*t, expr_t))
        },
        StmtKind::LetFunction(f, params, ret_t, block, ret_expr) =>
        {
            // XXX is a clone too slow?
            let mut func_st = st.clone();
            for (x, t) in params.iter()
            {
                context_add_vwt(info, &mut func_st.context, (x, t))?;
            }

            check_block(block, &mut func_st)?;

            let ret_expr_t = expr_type(ret_expr, &func_st.context, &func_st.function_context)?;
            check(
                ret_expr_t.is_subtype_of(*ret_t),
                info,
                SemanticError::ReturnTypeMismatch(*ret_t, ret_expr_t),
            )?;

            // XXX if we ever want to add recursion, then
            // this addition should be moved to the *beginning*
            // of this block, right before the clone
            let param_ts = params.iter().map(|(_, t)| *t).collect();
            st.function_context.add(f.to_string(), param_ts, *ret_t).map_err(|e| (info, e))
        },
        /*StmtKind::If(guard, block)
        | StmtKind::While(guard, block) => {
            check_guard_and_block(guard, block, st)
        },
        StmtKind::Assign(x, e) => {
            let var_t = context_get(info, &st.context, x)?;
            let expr_t = expr_type(e, &st.context, &st.function_context)?;
            check(
                expr_t.is_subtype_of(var_t),
                info,
                SemanticError::TypeMismatch(var_t, expr_t),
            )
        },
        StmtKind::Call(f, args) =>
            call_type(info, &st.context, &st.function_context, f, args)
            .map(|_| ()),
        StmtKind::Print(_) => panic!("No printing sorry :("),*/
    }
}

// For if and while
/*fn check_guard_and_block<S: HasInfo, E: HasInfo>(
    guard: &Expr<E>,
    block: &Vec<Stmt<S, E>>,
    st: &mut CheckingState,
) -> Result<(), InfoError>
{
    let (meta, _) = guard;
    let info = meta.info();
    let guard_t = expr_type(guard, &st.context, &st.function_context)?;
    check(
        guard_t.is_bool(),
        info,
        SemanticError::TypeMismatch(Type::Bool, guard_t),
    )?;
    // XXX is clone too slow?
    let mut block_st = st.clone();
    check_block(block, &mut block_st)
}*/
