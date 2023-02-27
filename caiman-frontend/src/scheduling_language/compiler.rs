use crate::scheduling_language::ast_factory::ASTFactory;
use crate::scheduling_language::parser;
use crate::scheduling_language::ast;
use crate::error;
use std::path::Path;
use std::fs::File;

pub enum Stage
{
    Parse,
}

fn parse_string(buf: String) -> ast::ParsedProgram
{
    let parser = parser::ProgramParser::new();
    let ast_factory = ASTFactory::new(&buf);
    parser.parse(&ast_factory, &buf).unwrap()
}

fn parse_read<R: std::io::Read>(
    mut input: R,
) -> ast::ParsedProgram
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    parse_string(buf)
}

fn parse_file(filename: &str) -> ast::ParsedProgram
{
    let input_path = Path::new(filename);
    let input_file = match File::open(&input_path) {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };
    parse_read(input_file)
}

pub fn parse(filename: &str) -> Result<ast::ParsedProgram, error::Error>
{
    Ok(parse_file(filename))
}

pub fn run_until_stage(filename: &str, stage: Stage) 
{
    let parsed_ast = parse_file(filename);
    match stage
    {
        Stage::Parse => println!("{:?}", parsed_ast),
    }
}
