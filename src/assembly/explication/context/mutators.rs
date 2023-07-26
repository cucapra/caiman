use super::*;
use crate::assembly::explication::util::*;

impl<'context> Context<'context> {
    pub fn update_node(&mut self, node: NodeId) {
        self.location.node = Some(node);
    }

    pub fn enter_funclet(&mut self, funclet: FuncletId) {
        // updates the location and the scope vec
    }

    pub fn add_instantiation(&mut self, schedule_node: NodeId,
}