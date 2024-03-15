use super::*;
use crate::explication::util::*;
use std::collections::hash_map::Entry;
use std::fmt::Debug;

impl FuncletOutState {
    pub fn add_allocation_request(&mut self, typ: StorageTypeId) {
        *self.allocation_requests.entry(typ).or_insert(0) += 1;
    }

    pub fn pop_allocation_request(&mut self, typ: &StorageTypeId) -> bool {
        match self.allocation_requests.get_mut(typ) {
            Some(mut x) => {
                if (*x > 0) {
                    *x -= 1;
                    true
                } else {
                    false
                }
            },
            None => false
        }
    }

    pub fn drain_allocation_requests(&mut self) -> Vec<(expir::StorageTypeId, usize)> {
        self.allocation_requests.drain().collect()
    }

    pub fn add_fill(&mut self, spec_type: Location) {
        let check = self.to_fill.insert(spec_type);
        assert!(check);
    }

    pub fn reqs_fill(&self, spec_type: &Location) -> bool {
        self.to_fill.contains(spec_type)
    }

    pub fn fill(&mut self, spec_type: &Location) {
        let check = self.to_fill.remove(spec_type);
        assert!(check);
    }

    pub fn has_fills_remaining(&self) -> bool {
        self.to_fill.is_empty()
    }

    pub fn push_node(&mut self, command: expir::Node) {
        self.nodes.push_front(command);
    }

    pub fn drain_nodes(&mut self) -> Vec<expir::Node> {
        self.nodes.drain(..).collect()
    }
}