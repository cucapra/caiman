use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use pest_consume::{Parser, match_nodes, Error};

#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
struct CaimanAssemblyParser;

use crate::{assembly, frontend, ir};
use assembly::ast;
use ast::Hole;
use ast::{
    ExternalFunctionId, FFIType, FuncletId, OperationId, RemoteNodeId, StorageTypeId, TypeId,
    ValueFunctionId,
};
use ir::ffi;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

#[derive(Clone)]
struct UserData {}

#[pest_consume::parser]
impl CaimanAssemblyParser {
    fn id(input: Node) -> Result<String> {
        input.as_str().parse::<String>().map_err(|e| input.error(e))
    }
    fn n(input: Node) -> Result<usize> {
        input.as_str().parse::<usize>().map_err(|e| input.error(e))
    }
    fn program(input: Node) -> Result<ast::Program> {
        todo!()
    }
}

pub fn parse(code: &str) -> Result<assembly::ast::Program> {
    // necessary to have an empty user data for checking stuff
    let user_data = UserData { };
    let parsed = CaimanAssemblyParser::parse_with_userdata(Rule::program, code, user_data)?;
    CaimanAssemblyParser::program(parsed.single()?)
}
