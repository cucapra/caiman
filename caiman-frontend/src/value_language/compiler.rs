use crate::error;
use super::ast::ParsedProgram;
use super::ast::TypedProgram;
use super::run_parser;
use super::check;
use super::type_elab;

#[derive(PartialEq, Eq)]
pub enum Stage
{
    Parse,
    TypeElaborate,
}

enum StageOutput
{
    Parse(ParsedProgram),
    TypeElaborate(TypedProgram),
}

fn run_with_result(filename: &str, stage: Stage) -> Result<StageOutput, error::LocalError>
{
    let parsed_ast = run_parser::parse_file(filename)?;
    if stage == Stage::Parse { return Ok(StageOutput::Parse(parsed_ast)); }
    let typed_ast = type_elab::elaborate_program(&parsed_ast)?;
    Ok(StageOutput::TypeElaborate(typed_ast))
    // Uncomment when another stage is added
    //if stage == Stage::TypeElaborate { return Ok(StageOutput::TypeElaborate(typed_ast)); }
}

fn globalize_error(filename: &str, e: error::LocalError) -> error::Error
{
    error::Error { kind: e.kind, location: e.location, filename: filename.to_string() }
}

fn handle_output_by_stage(s: StageOutput)
{
    use StageOutput::*;
    match s
    {
        Parse(ast) => println!("{:?}", ast),
        TypeElaborate(ast) => println!("{:?}", ast),
    }
}

pub fn run_until_stage(filename: &str, stage: Stage)
{
    match run_with_result(filename, stage)
    {
        Ok(s) => handle_output_by_stage(s),
        Err(e) => println!("{}", globalize_error(filename, e)),
    }
}

pub fn parse(filename: &str) -> Result<ParsedProgram, error::Error>
{
    match run_with_result(filename, Stage::Parse)
    {
        Ok(StageOutput::Parse(ast)) => Ok(ast),
        Err(local_e) => Err(globalize_error(filename, local_e)),
        _ => panic!("Value language compiler \"parse\" function is bugged"), 
    }
}

pub fn parse_and_elaborate(filename: &str) -> Result<TypedProgram, error::Error>
{
    match run_with_result(filename, Stage::TypeElaborate)
    {
        Ok(StageOutput::TypeElaborate(ast)) => Ok(ast),
        Err(local_e) => Err(globalize_error(filename, local_e)),
        _ => panic!("Value language compiler \"parse_and_elaborate\" function is bugged"), 
    }
}
