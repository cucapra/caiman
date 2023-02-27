use super::ast;
use super::typing;
use super::typing::{Context, InferredType, Type};
use crate::error::{ErrorKind, ErrorLocation, HasInfo, Info, LocalError};

#[derive(Debug, Clone)]
pub enum SemanticError
{
    NameCollision(String),
    TypeMismatch(Type, InferredType),
    UnboundVariable(String),
    Incompatible(InferredType, InferredType),
}

pub type SemanticInfoError = (Info, SemanticError);

fn to_local_error(e: SemanticInfoError) -> LocalError
{
    LocalError { kind: ErrorKind::Semantic(e.1), location: ErrorLocation::Double(e.0.location) }
}

pub fn check_program<S: HasInfo, E: HasInfo>(
    program: &ast::Program<S, E>,
) -> Result<Context, LocalError>
{
    let mut ctx = Context::new();
    for (metadata, stmt_kind) in program.iter()
    {
        let info = metadata.info();
        let to_local_with_info = |e| to_local_error((info, e));
        use ast::StmtKind::*;
        match stmt_kind
        {
            Let((x, x_type), e) =>
            {
                let e_inferred_type = typing::type_of_expr(e, &ctx).map_err(to_local_error)?;
                typing::expect_type(*x_type, e_inferred_type).map_err(to_local_with_info)?;
                ctx.add(x, *x_type).map_err(to_local_with_info)?;
            },
            _ => todo!(),
        }
    }
    Ok(ctx)
}
