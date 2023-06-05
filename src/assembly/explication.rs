mod explicator;
mod util;
mod context;

use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, StorageTypeId, TypeId,
};
use context::Context;
use crate::assembly::context::FuncletLocation;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

// always called immediately, turns arguments into phi nodes and renames `_`
// note that phi nodes will precede anything in the function, including other phi nodes
fn corrections(program: ast::Program) -> (usize, ast::Program) {
    let mut number = 0;
    let declarations = program
        .declarations
        .into_iter()
        .map(|declaration| match declaration {
            ast::Declaration::Funclet(f) => {
                let mut commands = Vec::new();
                let mut index = 0;
                for arg in &f.header.args {
                    commands.push(Some(ast::Command::Node(ast::NamedNode {
                        name: arg.name.clone().unwrap_or(NodeId("".to_string())),
                        node: ast::Node::Phi { index: Some(index) },
                    })));
                    index += 1;
                }
                for command in f.commands.into_iter() {
                    let new_command = match command {
                        Some(ast::Command::Node(ast::NamedNode { node, name })) => {
                            Some(ast::Command::Node(ast::NamedNode {
                                name: if name.0 == "_" {
                                    let result = NodeId(format!("~{}", number));
                                    number += 1;
                                    result
                                } else {
                                    name
                                },
                                node
                            }))
                        }
                        _ => command
                    };
                    commands.push(new_command);
                }
                ast::Declaration::Funclet(ast::Funclet {
                    kind: f.kind,
                    header: f.header,
                    commands,
                })
            }
            d => d,
        })
        .collect();
    (number, ast::Program {
        version: program.version,
        declarations,
    })
}

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
// alternatively we could use macros to lift the holes from the ast?
//   seems cool, but probably too much work
// arguably this pass should be on the lowered AST rather than on the frontend
//   but debugging explication is gonna be even harder without names...
pub fn explicate(program: ast::Program) -> ast::Program {
    let (number, program) = corrections(program);
    let context = Context::new(&program, number);
    program
}
