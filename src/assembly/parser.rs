use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
pub struct IRParser;
use crate::{ir, frontend};

pub fn parse(code : &str) ->
Result<frontend::Definition, ron::de::Error> {
    let parse = IRParser::parse(Rule::program, code);
    dbg!(parse);
    std::process::exit(0)
}