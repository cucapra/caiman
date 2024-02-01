use super::*;
use crate::assembly::explication::util::*;
use paste::paste;
use std::collections::hash_map::Entry;
use std::fmt::Debug;

impl FuncletOutState {
    pub fn add_allocation_request(&mut self, typ: StorageTypeId) {
        self.allocation_requests.entry(typ).default(0) += 1;
    }

    pub fn pop_allocation_request(&mut self, typ: StorageTypeId) -> bool {
        match self.allocation_requests.get_mut(typ) {
            Some(mut x) => {
                if (x > 0) {
                    x -= 1;
                    true
                } else {
                    false
                }
            }
            None {
                false
            }
        }
    }

    pub fn add_fill(&mut self, spec_type: Location) {
        let result = self.insert(spec_typ);
        assert!(!result);
    }

    pub fn reqs_fill(&self, spec_type: Location) -> bool {
        self.to_fill.contains(spec_typ)
    }

    pub fn fill(&mut self, spec_type: Location) {
        let result = self.remove(spec_typ);
        assert!(!result);
    }

    pub fn push_comand(&mut self, command: ast::Command) {
        self.commands.push_front(command);
    }
}