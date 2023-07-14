use crate::ir;
use super::*;

// These are all getters designed to work with "original" data, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl<'context> Context<'context> {
    pub fn static_schedule_funclet_ids(&self) -> Vec<ast::FuncletId> {
        let mut result = Vec::new();
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    match f.kind {
                        ir::FuncletKind::ScheduleExplicit => {
                            result.push(f.header.name.clone());
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        };
        result
    }

    pub fn static_node_ids(&self, funclet : &ast::FuncletId) -> Vec<ast::NodeId> {
        let mut result = Vec::new();
        for command in &self.get_funclet(funclet).unwrap().commands {
            match command {
                Some(ast::Command::Node(named)) => {
                    match &named.name {
                        None => {},
                        Some(name) => { result.push(name.clone()); }
                    }
                },
                _ => {}
            }
        };
        result
    }

    pub fn location_funclet(&self) -> &FuncletId {
        self.location.funclet.as_ref().unwrap()
    }

    pub fn location_node(&self) -> &NodeId {
        self.location.node.as_ref().unwrap()
    }
}