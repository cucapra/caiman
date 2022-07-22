// This module is for semantic checking and type elaboration. 
// The first version of this will assume that the type of 
// everything is i32, as type annotations have not been 
// added to the language yet
use crate::value_language::ast;
use crate::value_language::error;
use crate::value_language::typing::{Context, Type};

pub enum SemanticError
{
}

fn check_exp(exp: &ast::Exp) -> Result<Type, SemanticError>
{
    Ok(Type::I32)
}

// To factor out if and while code
fn check_guard_and_block(
    context: &mut Context,
    guard: &ast::Exp, 
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

fn check_statement(
    context: &mut Context,
    statement: &ast::ParsedStatement,
) -> Result<ast::CheckedStatement, SemanticError>
{
    match statement
    {
        ast::Statement::If(guard, block) => {
            let (t, checked_block) = check_guard_and_block(
                context, 
                guard, 
                block,
            )?;
            Ok(ast::Statement::If((t, guard.clone()), checked_block))
        },
        ast::Statement::While(guard, block) => {
            let (t, checked_block) = check_guard_and_block(
                context, 
                guard, 
                block,
            )?;
            Ok(ast::Statement::If((t, guard.clone()), checked_block))
        },
        /*ast::Statement::Let(is_mut, var, exp) => {
            // TODO: Types other than i32
            let t = Type::I32;
            // Shadowing is allowed for now
            context.add(var.to_string(), t, is_mut);

        },*/
        _ => panic!("TODO"),
    }
}

pub fn check_program(
    program: &ast::Program
) -> Result<ast::CheckedProgram, error::Error>
{
    let mut context = Context::new();
    let checked : Result<Vec<ast::CheckedStatement>, SemanticError> = program
        .iter()
        .map(|statement| check_statement(&mut context, statement))
        .collect();
    checked.map_err(|e| error::Error::Semantic(e))
}

