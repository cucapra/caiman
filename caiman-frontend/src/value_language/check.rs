// Does type checking, but not elaboration. There will be a 
// separate elaboration file that assumes tree has been 
// checked beforehand.

use crate::value_language::error;
use crate::value_language::ast::*;
use crate::value_language::typing::{Context, FunctionContext, Type, ExprType};
use crate::value_language::typing;

pub enum SemanticError
{
    FunctionNameCollision(String),
    LetTypeMismatch(Type, ExprType),
    UnboundVariable(String),
    WrongBinop(ExprType, Binop),
    Incompatible(ExprType, ExprType),
}

type InfoError = (Info, SemanticError);

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
    filename: &str,
    program: &Program<S, E>,
) -> Result<(), error::Error>
where
    S: HasInfo,
    E: HasInfo,
{
    let mut context = Context::new();
    let mut function_context = FunctionContext::new();
    let mut st = CheckingState { context, function_context };
    for stmt in program.iter()
    {
        check_stmt(stmt, &mut st).map_err(|(info, e)| error::Error {
            kind: error::ErrorKind::Semantic(e),
            location: error::ErrorLocation::Double(info),
            filename: filename.to_string(),
        })?;
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
        StmtKind::Let(is_mut, (x, t), expr) => {
            st.context.add(x.to_string(), *t, *is_mut);
            let expr_t = expr_type(expr, st)?;
            check(
                expr_t.is_subtype_of(*t),
                info,
                SemanticError::LetTypeMismatch(*t, expr_t),
            )
        },
        _ => panic!("TODO check stmt"),
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
        // TODO: Other number types
        ExprKind::Num(n) => Ok(expr_type_of_num(n)),
        ExprKind::Bool(_) => Ok(ExprType::Ordinary(Type::Bool)),
        ExprKind::Var(x) => context_get(info, &st.context, x),
        ExprKind::Binop(bop, e1, e2) => {
            let t1 = expr_type(e1, st)?;
            let t2 = expr_type(e2, st)?;
            check_binop(info, bop, t1, t2)
        },
        _ => panic!("TODO check expr"),
    }
}

fn check_binop(
    info: Info,
    bop: &Binop, 
    t1: ExprType, 
    t2: ExprType
) -> Result<ExprType, InfoError>
{
    let check_num = |t: ExprType| 
        check(t.is_number(), info, SemanticError::WrongBinop(t, *bop));
    let check_bool = |t: ExprType| 
        check(t.is_bool(), info, SemanticError::WrongBinop(t, *bop));
    let check_compatible = || 
        check(t1.compatible(t2), info, SemanticError::Incompatible(t1, t2));
    let bop_body = 
        |f: &dyn Fn(ExprType) -> Result<(), InfoError>, ret: ExprType| { 
            f(t1)?; f(t2)?; check_compatible()?; Ok(ret)
        };
    match bop
    {
        Binop::Plus | Binop::Minus | Binop::Mult | Binop::Div => {
            bop_body(&check_num, t1)
        },
        Binop::Equals => {
            bop_body(&check_num, ExprType::Ordinary(Type::Bool))
        },
        Binop::And | Binop::Or => {
            bop_body(&check_bool, t1)
        },
    }
}

fn context_get(
    info: Info, 
    context: &Context, 
    x: &Var,
) -> Result<ExprType, InfoError>
{
    match context.get_type(x)
    {
        Ok(t) => Ok(ExprType::Ordinary(t)),
        Err(e) => Err((info, e)),
    }
}

fn expr_type_of_num(num: &str) -> ExprType
{
    if num.contains(".") { ExprType::Float }
    else if num.contains("-") { ExprType::NegativeInteger }
    else { ExprType::PositiveInteger }
}

/*

// To factor out if and while code
fn check_guard_and_block(
    context: &mut Context,
    guard: &ast::ParsedExp, 
    block: &Vec<ast::ParsedStatement>,
) -> Result<(Type, Vec<ast::CheckedStatement>), SemanticError>
{
    let checked_guard = check_exp(guard)?;
    // TODO: Is cloning a hash map too slow? 
    let mut block_context = context.clone();
    let checked_block : Result<Vec<ast::CheckedStatement>, SemanticError> = 
        block.iter()
        .map(|s| check_statement(&mut block_context, s))
        .collect();
    checked_block.map(|b| (checked_guard, b))
}

*/
