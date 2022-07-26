use crate::value_language::run_parser;
use crate::value_language::check;
use crate::value_language::error;

fn run_with_result(filename: &str) -> Result<(), error::Error>
{
    let parsed_ast = run_parser::parse_file(filename)?;
    check::check_program(filename, &parsed_ast)?;
    Ok(())
}

pub fn run(filename: &str)
{
    match run_with_result(filename)
    {
        Ok(()) => println!("Program OK."),
        Err(e) => println!("{}", e),
    }
}
