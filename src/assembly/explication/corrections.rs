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

// pre-processing function that corrects some stupid details
// gives names to variable holes, `_`
// also puts functlet arguments into the funclet
pub fn correct(context: &mut Context) {
    let mut funclets_to_update = Vec::new();
    context
        .program()
        .declarations
        .iter()
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
                funclets_to_update.push((&f.header.name, commands))
            }
            _ => {}
        });
    for (funclet_name, commands) in funclets_to_update.into_iter() {
        context.forcibly_replace_commands(funclet_name, commands)
    };
}
