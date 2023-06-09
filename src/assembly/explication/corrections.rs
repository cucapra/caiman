use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, StorageTypeId, TypeId,
};
use crate::assembly::context::FuncletLocation;
use crate::assembly::explication::context::Context;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

fn corrections(context: Context) -> Context {
    context
        .program
        .declarations
        .iter_mut()
        .map(|declaration| match declaration {
            ast::Declaration::Funclet(f) => {
                let mut index = 0;
                let mut commands = Vec::new();
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
                                    NodeId(context.next_name())
                                } else {
                                    name
                                },
                                node,
                            }))
                        }
                        _ => command,
                    };
                    commands.push(new_command);
                }
            }
            _ => {}
        });
}
