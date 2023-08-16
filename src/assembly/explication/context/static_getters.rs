use super::*;
use crate::ir;

// These are all getters designed to work with "original" data, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl<'context> Context<'context> {
    // we need to own the id so that this is static under explication
    // the funclet name should never change, but to be safe
    pub fn static_schedule_funclet_ids(&self) -> Vec<FuncletId> {
        let mut result = Vec::new();
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => match f.kind {
                    ir::FuncletKind::ScheduleExplicit => {
                        result.push(f.header.name.clone());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        result
    }

    // we need to own the name so that this is static
    // the command names should never be changed, but to be safe
    pub fn static_command_ids(&self, funclet: &ast::FuncletId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut tail_edge = None;
        for command in &self.get_funclet(funclet).commands {
            match &command.command {
                ast::Command::Node(_) => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                ast::Command::Hole => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                ast::Command::TailEdge(_) => {
                    if tail_edge.is_some() {
                        panic!("Multiple tail edges found for {:?}", funclet);
                    }
                    tail_edge = Some(command.name.as_ref().unwrap().clone());
                }
                ast::Command::ExplicationHole => unreachable!(),
            }
        }
        match tail_edge {
            None => {} // no error because this might be a hole
            Some(edge) => result.push(edge),
        }
        result
    }
}
