use crate::value_language::run_parser;
use crate::value_language::check;
use crate::error;
use crate::value_language::ast::ParsedProgram;
use crate::stage::Stage;

enum StageOutput
{
    Parse(ParsedProgram),
    Check(ParsedProgram),
    //TypeElaborate
}

fn run_with_result(
    filename: &str, 
    stage: Stage,
) -> Result<StageOutput, error::LocalError>
{
    let parsed_ast = run_parser::parse_file(filename)?;
    //check::check_program(&parsed_ast)?;
    Ok(match stage
    {
        Stage::Parse => StageOutput::Parse(parsed_ast),
        Stage::Check => StageOutput::Check(parsed_ast),
    })
}

fn handle_output(s: StageOutput)
{
    use StageOutput::*;
    match s
    {
        Parse(ast) => println!("{:?}", ast),
        Check(_) => println!("Program OK."),
    }
}

pub fn run_output(filename: &str) -> Result<ParsedProgram, error::Error>
{
    match run_with_result(filename, Stage::Check)
    {
        Ok(s) => match s {
            StageOutput::Parse(ast) => Ok(ast),
            StageOutput::Check(ast) => Ok(ast),
        },
        Err(e) => {
            let e_global = error::Error {
                kind: e.kind,
                location: e.location,
                filename: filename.to_string(),
            };
            Err(e_global)
        },
    }
}

pub fn run(filename: &str, stage: Stage)
{
    match run_with_result(filename, stage)
    {
        Ok(s) => handle_output(s),
        Err(e) => {
            let e_global = error::Error {
                kind: e.kind,
                location: e.location,
                filename: filename.to_string(),
            };
            println!("{}", e_global)
        },
    }
}
